//! Lắp ráp service: chọn adapter cho từng port, chạy bốn task và tắt có thứ tự (bài 39).

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use ledger_api::auth::AuthConfig;
use ledger_api::http::{self, ApiState};
use ledger_application::Ledger;
use ledger_application::readiness::Readiness;
use ledger_domain::clock::Clock;
use ledger_domain::default_policy;
use ledger_infrastructure::consumer::{self, ConsumerConfig};
use ledger_infrastructure::kafka;
use ledger_infrastructure::pg::PgStore;
use ledger_infrastructure::relay;
use ledger_infrastructure::stats::Stats;
use tokio::net::TcpListener;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::config::Config;

/// Service đang chạy. Cổng thật đọc từ `http_addr`/`grpc_addr` (bind cổng 0).
pub struct App {
    /// Địa chỉ HTTP đang nghe.
    pub http_addr: SocketAddr,
    /// Địa chỉ gRPC đang nghe.
    pub grpc_addr: SocketAddr,
    /// Bộ đếm hoạt động của consumer và relay.
    pub stats: Arc<Stats>,
    /// Cờ sẵn sàng.
    pub readiness: Arc<Readiness>,
    store: PgStore,
    timeout: Duration,
    tokens: Tokens,
    consumer: Option<JoinHandle<Result<()>>>,
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
    /// Khởi động: PostgreSQL, producer, hai cổng nghe, rồi bốn task. `clock` là đồng hồ của use case.
    ///
    /// # Errors
    /// Không kết nối được PostgreSQL, không bind được cổng, khóa JWT sai hoặc cấu hình Kafka lỗi.
    pub async fn start(cfg: Config, clock: Arc<dyn Clock>) -> Result<App> {
        let store = PgStore::connect(&cfg.pg_url, cfg.pg_schema.as_deref(), &cfg.topic_out)
            .await
            .context("kết nối PostgreSQL")?;
        let producer = kafka::producer(&cfg.kafka)?;
        let auth =
            Arc::new(AuthConfig::with_secret(&cfg.jwt_secret.0).map_err(anyhow::Error::msg)?);
        let http_listener = TcpListener::bind(cfg.http_addr)
            .await
            .context("bind HTTP")?;
        let grpc_listener = TcpListener::bind(cfg.grpc_addr)
            .await
            .context("bind gRPC")?;
        let http_addr = http_listener.local_addr()?;
        let grpc_addr = grpc_listener.local_addr()?;

        // Điểm lắp duy nhất: PgStore cắm vào port LedgerStore, đồng hồ và chính sách phí cắm vào use case.
        let ledger = Arc::new(Ledger::new(
            store.clone(),
            clock,
            Arc::new(default_policy()),
        ));
        let stats = Arc::new(Stats::default());
        let readiness = Arc::new(Readiness::default());
        let wake = Arc::new(Notify::new());
        let tokens = Tokens {
            consumer: CancellationToken::new(),
            relay: CancellationToken::new(),
            servers: CancellationToken::new(),
        };

        let relay = tokio::spawn(relay::run(
            store.pool().clone(),
            producer,
            stats.clone(),
            wake.clone(),
            cfg.relay_poll,
            tokens.relay.clone(),
        ));

        let consumer = tokio::spawn({
            let ccfg = ConsumerConfig {
                brokers: cfg.kafka.clone(),
                topic: cfg.topic_in.clone(),
                group: cfg.group.clone(),
            };
            let (ledger, stats, ready, token) = (
                ledger.clone(),
                stats.clone(),
                readiness.clone(),
                tokens.consumer.clone(),
            );
            async move { consumer::run(&ccfg, ledger, stats, ready, wake, token).await }
        });

        let router = http::router(ApiState {
            ledger: ledger.clone(),
            auth: auth.clone(),
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
            let service = ledger_api::grpc::service(ledger, auth);
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
            elapsed_ms = u64::try_from(report.elapsed.as_millis()).unwrap_or(u64::MAX),
            timed_out = report.timed_out,
            "shutdown complete"
        );
        report
    }
}

// Chờ một task tới `deadline`; quá hạn thì hủy cứng. Trả `true` nếu task tự kết thúc kịp và không lỗi.
async fn join_before<T: JoinOutcome>(
    name: &str,
    mut handle: JoinHandle<T>,
    deadline: tokio::time::Instant,
) -> bool {
    match tokio::time::timeout_at(deadline, &mut handle).await {
        Ok(Ok(outcome)) => outcome.report(name),
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

// Task trả `()` luôn sạch; task trả `Result` thì `Err` là tắt không sạch.
trait JoinOutcome {
    fn report(self, name: &str) -> bool;
}

impl JoinOutcome for () {
    fn report(self, _name: &str) -> bool {
        true
    }
}

impl JoinOutcome for Result<()> {
    fn report(self, name: &str) -> bool {
        match self {
            Ok(()) => true,
            Err(err) => {
                warn!(task = name, error = %err, "task kết thúc với lỗi");
                false
            }
        }
    }
}
