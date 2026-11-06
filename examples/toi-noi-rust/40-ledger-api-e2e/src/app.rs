//! Lắp ráp service: bốn task (consumer, relay, HTTP, gRPC) và quy trình tắt có thứ tự.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use tokio::net::TcpListener;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::consumer::{self, ConsumerExit};
use crate::grpc::{GrpcService, pb::ledger_server::LedgerServer};
use crate::http::{self, HttpState};
use crate::kafka;
use crate::relay;
use crate::state::{Readiness, Stats};
use crate::store::Store;

/// Service đang chạy. Cổng thật đọc từ `http_addr`/`grpc_addr` (bind cổng 0).
pub struct App {
    /// Địa chỉ HTTP đang nghe.
    pub http_addr: SocketAddr,
    /// Địa chỉ gRPC đang nghe.
    pub grpc_addr: SocketAddr,
    /// Bộ đếm hoạt động.
    pub stats: Arc<Stats>,
    /// Cờ sẵn sàng.
    pub readiness: Arc<Readiness>,
    store: Store,
    timeout: Duration,
    tokens: Tokens,
    consumer: Option<JoinHandle<Result<ConsumerExit>>>,
    relay: JoinHandle<()>,
    http: JoinHandle<()>,
    grpc: JoinHandle<()>,
}

// Mỗi task một token riêng để tắt theo thứ tự, không phải một token cho tất cả.
struct Tokens {
    consumer: CancellationToken,
    relay: CancellationToken,
    servers: CancellationToken,
}

/// Kết quả quy trình tắt.
#[derive(Debug, Clone, Copy)]
pub struct ShutdownReport {
    /// Có task nào phải bị hủy cứng vì quá hạn không.
    pub timed_out: bool,
    /// Tổng thời gian tắt.
    pub elapsed: Duration,
}

impl App {
    /// Khởi động: PostgreSQL, producer, hai cổng nghe, rồi bốn task.
    ///
    /// # Errors
    /// Không kết nối được PostgreSQL, không bind được cổng, hoặc cấu hình Kafka lỗi.
    pub async fn start(cfg: Config) -> Result<App> {
        let store = Store::connect(&cfg).await.context("kết nối PostgreSQL")?;
        let producer = kafka::producer(&cfg.kafka)?;
        let http_listener = TcpListener::bind(cfg.http_addr)
            .await
            .context("bind HTTP")?;
        let grpc_listener = TcpListener::bind(cfg.grpc_addr)
            .await
            .context("bind gRPC")?;
        let http_addr = http_listener.local_addr()?;
        let grpc_addr = grpc_listener.local_addr()?;

        let stats = Arc::new(Stats::default());
        let readiness = Arc::new(Readiness::default());
        let wake = Arc::new(Notify::new());
        let tokens = Tokens {
            consumer: CancellationToken::new(),
            relay: CancellationToken::new(),
            servers: CancellationToken::new(),
        };

        let relay = tokio::spawn(relay::run(
            store.clone(),
            producer,
            stats.clone(),
            wake.clone(),
            cfg.relay_poll,
            tokens.relay.clone(),
        ));

        let consumer = tokio::spawn({
            let (store, stats, ready, token) = (
                store.clone(),
                stats.clone(),
                readiness.clone(),
                tokens.consumer.clone(),
            );
            let cfg = cfg.clone();
            async move { consumer::run(&cfg, store, stats, ready, wake, token).await }
        });

        let router = http::router(HttpState {
            store: store.clone(),
            ready: readiness.clone(),
        });
        let http = tokio::spawn({
            let token = tokens.servers.clone();
            async move {
                let served = axum::serve(http_listener, router)
                    .with_graceful_shutdown(token.cancelled_owned())
                    .await;
                if let Err(err) = served {
                    error!(%err, "HTTP dừng vì lỗi");
                }
            }
        });
        let grpc = tokio::spawn({
            let token = tokens.servers.clone();
            let service = LedgerServer::new(GrpcService::new(store.clone()));
            async move {
                let served = tonic::transport::Server::builder()
                    .add_service(service)
                    .serve_with_incoming_shutdown(
                        TcpListenerStream::new(grpc_listener),
                        token.cancelled_owned(),
                    )
                    .await;
                if let Err(err) = served {
                    error!(%err, "gRPC dừng vì lỗi");
                }
            }
        });

        Ok(App {
            http_addr,
            grpc_addr,
            stats,
            readiness,
            store,
            timeout: cfg.shutdown_timeout,
            tokens,
            consumer: Some(consumer),
            relay,
            http,
            grpc,
        })
    }

    /// Chờ consumer dừng tự nhiên (chết giả lập hoặc lỗi). Chỉ dùng được một lần.
    ///
    /// # Errors
    /// Consumer báo lỗi, bị panic, hoặc đã được lấy đi.
    pub async fn wait_consumer(&mut self) -> Result<ConsumerExit> {
        let handle = self.consumer.take().context("consumer đã được chờ rồi")?;
        handle.await.context("task consumer")?
    }

    /// Tắt có trật tự: ngừng nhận việc, xả nốt việc dở, rồi mới đóng cổng và pool.
    /// Mọi bước chia nhau một hạn chung `shutdown_timeout`; quá hạn thì hủy cứng phần còn lại.
    pub async fn shutdown(mut self) -> ShutdownReport {
        let start = Instant::now();
        let deadline = tokio::time::Instant::now() + self.timeout;
        info!("bắt đầu tắt");
        // 1. /readyz trả 503 ngay; cổng vẫn mở để orchestrator kịp thấy tín hiệu.
        self.readiness.begin_shutdown();
        // 2. Consumer: xong message đang xử lý, commit offset, rồi dừng nhận mới.
        self.tokens.consumer.cancel();
        let mut clean = true;
        if let Some(consumer) = self.consumer.take() {
            clean &= join_before("consumer", consumer, deadline).await;
        }
        // 3. Relay: phát nốt outbox. Chạy sau consumer để không còn dòng mới xuất hiện.
        self.tokens.relay.cancel();
        clean &= join_before("relay", self.relay, deadline).await;
        // 4. Hai server đóng sau cùng: chúng còn trả lời /readyz trong suốt các bước trên.
        self.tokens.servers.cancel();
        clean &= join_before("http", self.http, deadline).await;
        clean &= join_before("grpc", self.grpc, deadline).await;
        // 5. Đóng pool.
        self.store.pool().close().await;
        let report = ShutdownReport {
            timed_out: !clean,
            elapsed: start.elapsed(),
        };
        info!(
            elapsed_ms = report.elapsed.as_millis() as u64,
            timed_out = report.timed_out,
            "shutdown complete"
        );
        report
    }

    /// Giả lập process bị giết: hủy cứng mọi task, không xả gì cả.
    pub fn kill(self) {
        if let Some(consumer) = &self.consumer {
            consumer.abort();
        }
        self.relay.abort();
        self.http.abort();
        self.grpc.abort();
    }
}

// Chờ một task tới `deadline`; quá hạn thì hủy cứng. Trả `true` nếu task tự kết thúc kịp.
async fn join_before<T>(
    name: &str,
    mut handle: JoinHandle<T>,
    deadline: tokio::time::Instant,
) -> bool {
    match tokio::time::timeout_at(deadline, &mut handle).await {
        Ok(Ok(_)) => true,
        Ok(Err(err)) => {
            warn!(task = name, %err, "task kết thúc bất thường");
            false
        }
        Err(_) => {
            warn!(task = name, "quá hạn tắt, hủy cứng");
            handle.abort();
            false
        }
    }
}
