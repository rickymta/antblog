//! Dựng service và trình tự tắt: readiness, ngừng nhận việc, drain, commit offset, flush, đóng pool.

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use rdkafka::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::error::{KafkaError, RDKafkaErrorCode};
use rdkafka::producer::{FutureProducer, Producer};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::task::{AbortHandle, JoinHandle};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{info, warn};

use crate::api::{AppState, router};
use crate::consumer::intake_loop;
use crate::metrics::Metrics;
use crate::probes::Probes;
use crate::relay::run_relay;
use crate::worker::run_worker;

/// Cấu hình của một lần chạy. Mọi mốc thời gian là tham số, không ghi cứng trong logic.
#[derive(Debug, Clone)]
pub struct Config {
    /// URL PostgreSQL (biến `TOI_NOI_PG_URL`).
    pub pg_url: String,
    /// Schema `lab_<uuid>` đặt qua `search_path`.
    pub schema: String,
    /// Địa chỉ broker (biến `TOI_NOI_KAFKA`).
    pub kafka: String,
    /// Topic `trip.completed` của ca.
    pub topic_in: String,
    /// Topic `ledger.entry-posted` của ca.
    pub topic_out: String,
    /// Consumer group của ca.
    pub group: String,
    /// Sức chứa hàng đợi giữa consumer và worker.
    pub queue_capacity: usize,
    /// Consumer đang pause thì resume khi hàng đợi còn tối đa chừng này tin.
    pub resume_depth: usize,
    /// Bước chậm mô phỏng trong transaction.
    pub work_delay: Duration,
    /// Thời gian giữ `/readyz` = 503 mà vẫn phục vụ, trước khi ngừng nhận việc.
    pub readiness_grace: Duration,
    /// Thời gian tối đa cho drain (và riêng cho relay).
    pub drain_timeout: Duration,
    /// Chu kỳ auto-commit offset đã lưu.
    pub commit_interval_ms: u32,
    /// `session.timeout.ms`: broker chờ bao lâu mới coi thành viên đã chết.
    pub session_timeout_ms: u32,
    /// `max.poll.interval.ms`.
    pub max_poll_interval_ms: u32,
}

impl Config {
    /// Đọc từ biến môi trường `LAB_*` (tiến trình con của lab dùng cách này).
    ///
    /// # Errors
    /// Thiếu biến bắt buộc hoặc số không hợp lệ.
    pub fn from_env() -> Result<Config> {
        let text = |name: &str| std::env::var(name).with_context(|| format!("thiếu biến {name}"));
        let num = |name: &str, default: u64| -> Result<u64> {
            match std::env::var(name) {
                Ok(raw) => raw.parse().with_context(|| format!("{name} phải là số")),
                Err(_) => Ok(default),
            }
        };
        Ok(Config {
            pg_url: text("TOI_NOI_PG_URL")?,
            kafka: text("TOI_NOI_KAFKA")?,
            schema: text("LAB_SCHEMA")?,
            topic_in: text("LAB_TOPIC_IN")?,
            topic_out: text("LAB_TOPIC_OUT")?,
            group: text("LAB_GROUP")?,
            queue_capacity: usize::try_from(num("LAB_CAPACITY", 32)?)?,
            resume_depth: usize::try_from(num("LAB_RESUME_DEPTH", 24)?)?,
            work_delay: Duration::from_millis(num("LAB_WORK_DELAY_MS", 0)?),
            readiness_grace: Duration::from_millis(num("LAB_GRACE_MS", 200)?),
            drain_timeout: Duration::from_millis(num("LAB_DRAIN_TIMEOUT_MS", 5000)?),
            commit_interval_ms: u32::try_from(num("LAB_COMMIT_INTERVAL_MS", 200)?)?,
            session_timeout_ms: u32::try_from(num("LAB_SESSION_TIMEOUT_MS", 6000)?)?,
            max_poll_interval_ms: u32::try_from(num("LAB_MAX_POLL_MS", 300_000)?)?,
        })
    }

    /// Biến môi trường tương ứng, để truyền cho tiến trình con. Không có credential: URL được thừa kế.
    pub fn to_env(&self) -> Vec<(&'static str, String)> {
        vec![
            ("LAB_SCHEMA", self.schema.clone()),
            ("LAB_TOPIC_IN", self.topic_in.clone()),
            ("LAB_TOPIC_OUT", self.topic_out.clone()),
            ("LAB_GROUP", self.group.clone()),
            ("LAB_CAPACITY", self.queue_capacity.to_string()),
            ("LAB_RESUME_DEPTH", self.resume_depth.to_string()),
            ("LAB_WORK_DELAY_MS", self.work_delay.as_millis().to_string()),
            ("LAB_GRACE_MS", self.readiness_grace.as_millis().to_string()),
            (
                "LAB_DRAIN_TIMEOUT_MS",
                self.drain_timeout.as_millis().to_string(),
            ),
            (
                "LAB_COMMIT_INTERVAL_MS",
                self.commit_interval_ms.to_string(),
            ),
            (
                "LAB_SESSION_TIMEOUT_MS",
                self.session_timeout_ms.to_string(),
            ),
            ("LAB_MAX_POLL_MS", self.max_poll_interval_ms.to_string()),
        ]
    }
}

/// Consumer group: offset chỉ do worker lưu, sau khi commit transaction.
pub fn build_consumer(cfg: &Config) -> Result<StreamConsumer> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &cfg.kafka)
        .set("group.id", &cfg.group)
        .set("enable.auto.commit", "true")
        .set("enable.auto.offset.store", "false")
        .set(
            "auto.commit.interval.ms",
            cfg.commit_interval_ms.to_string(),
        )
        .set("auto.offset.reset", "earliest")
        .set("session.timeout.ms", cfg.session_timeout_ms.to_string())
        .set("max.poll.interval.ms", cfg.max_poll_interval_ms.to_string())
        .create()
        .context("tạo consumer Kafka")?;
    Ok(consumer)
}

/// Producer cho relay.
pub fn build_producer(kafka: &str) -> Result<FutureProducer> {
    ClientConfig::new()
        .set("bootstrap.servers", kafka)
        .set("message.timeout.ms", "10000")
        .set("enable.idempotence", "true")
        .create()
        .context("tạo producer Kafka")
}

/// Pool trỏ vào schema của ca qua `search_path`.
///
/// # Errors
/// URL sai hoặc không kết nối được.
pub async fn build_pool(url: &str, schema: &str) -> Result<PgPool> {
    let options = PgConnectOptions::from_str(url)?.options([("search_path", schema)]);
    PgPoolOptions::new()
        .max_connections(6)
        .connect_with(options)
        .await
        .context("không kết nối được PostgreSQL ở TOI_NOI_PG_URL")
}

/// Service đang chạy. `shutdown` là cách duy nhất để kết thúc đúng thứ tự.
pub struct Running {
    /// Địa chỉ HTTP thật (cổng do hệ điều hành cấp).
    pub addr: SocketAddr,
    /// Bộ đếm dùng chung.
    pub metrics: Arc<Metrics>,
    cfg: Config,
    pool: PgPool,
    consumer: Arc<StreamConsumer>,
    producer: FutureProducer,
    draining: Arc<AtomicBool>,
    intake: CancellationToken,
    relay_stop: CancellationToken,
    tracker: TaskTracker,
    worker_abort: AbortHandle,
    http_abort: AbortHandle,
    relay: JoinHandle<()>,
}

/// Khởi động HTTP, consumer, worker và relay.
///
/// # Errors
/// Không kết nối được PostgreSQL hoặc Kafka, không bind được cổng.
pub async fn start(cfg: Config) -> Result<Running> {
    let pool = build_pool(&cfg.pg_url, &cfg.schema).await?;
    let consumer = Arc::new(build_consumer(&cfg)?);
    consumer.subscribe(&[cfg.topic_in.as_str()])?;
    let producer = build_producer(&cfg.kafka)?;
    let metrics = Arc::new(Metrics::default());
    let draining = Arc::new(AtomicBool::new(false));

    // `relay_stop` là cha của `intake`: huỷ `intake` chỉ dừng nhận việc; huỷ `relay_stop` dừng tất cả.
    let relay_stop = CancellationToken::new();
    let intake = relay_stop.child_token();
    let tracker = TaskTracker::new();

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let state = AppState {
        pool: pool.clone(),
        probes: Arc::new(Probes::new(
            pool.clone(),
            producer.clone(),
            cfg.topic_out.clone(),
        )),
        draining: draining.clone(),
        topic_out: Arc::from(cfg.topic_out.as_str()),
        hold: cfg.work_delay,
    };
    let http_token = intake.clone();
    let http = tracker.spawn(async move {
        let served = axum::serve(listener, router(state))
            .with_graceful_shutdown(http_token.cancelled_owned())
            .await;
        if let Err(err) = served {
            warn!("HTTP dừng vì lỗi: {err}");
        }
    });

    let (tx, rx) = mpsc::channel(cfg.queue_capacity);
    let worker = tracker.spawn(run_worker(
        rx,
        pool.clone(),
        consumer.clone(),
        metrics.clone(),
        cfg.topic_out.clone(),
        cfg.work_delay,
    ));
    tracker.spawn(intake_loop(
        consumer.clone(),
        tx,
        intake.clone(),
        metrics.clone(),
        cfg.resume_depth,
    ));
    let relay = tokio::spawn(run_relay(
        pool.clone(),
        producer.clone(),
        relay_stop.clone(),
        metrics.clone(),
    ));

    Ok(Running {
        addr,
        metrics,
        worker_abort: worker.abort_handle(),
        http_abort: http.abort_handle(),
        cfg,
        pool,
        consumer,
        producer,
        draining,
        intake,
        relay_stop,
        tracker,
        relay,
    })
}

/// Báo cáo một lần tắt.
#[derive(Debug, Clone)]
pub struct ShutdownReport {
    /// Số tin xử lý xong trong thời gian giữ readiness 503.
    pub done_in_grace: u64,
    /// Số tin xử lý xong trong lúc drain (sau khi ngừng nhận việc).
    pub done_in_drain: u64,
    /// Thời gian từ lúc bắt đầu tắt đến khi xong, mili giây.
    pub total_ms: u128,
    /// Drain vượt `drain_timeout`: task bị huỷ.
    pub drain_timed_out: bool,
    /// Relay không kịp dọn outbox trong thời hạn.
    pub relay_timed_out: bool,
    /// Kết quả commit offset cuối.
    pub commit: Result<(), String>,
    /// Số dòng outbox chưa phát khi tắt xong (`None` nếu không đếm được).
    pub outbox_pending: Option<i64>,
}

impl ShutdownReport {
    /// 0 sạch, 3 nếu phải cắt drain hoặc relay.
    pub fn exit_code(&self) -> u8 {
        if self.drain_timed_out || self.relay_timed_out {
            3
        } else {
            0
        }
    }
}

impl Running {
    /// Bản sao pool, để scenario hỏi DB.
    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }

    /// Trình tự tắt. Mỗi bước có log để đọc lại thứ tự.
    pub async fn shutdown(self) -> ShutdownReport {
        let started = Instant::now();
        let metrics = self.metrics.clone();

        // 1. Báo "đừng gửi thêm cho tôi": /readyz = 503, vẫn phục vụ trong thời gian grace.
        self.draining.store(true, Ordering::Release);
        info!(
            "shutdown phase=1 readyz=503 grace_ms={}",
            self.cfg.readiness_grace.as_millis()
        );
        let done_before_grace = metrics.done();
        tokio::time::sleep(self.cfg.readiness_grace).await;
        let done_after_grace = metrics.done();

        // 2. Ngừng nhận việc mới: HTTP ngừng accept, consumer ngừng poll, worker còn hàng đợi.
        self.intake.cancel();
        self.tracker.close();
        info!("shutdown phase=2 intake=stopped http=no-new-connections consumer=no-new-messages");

        // 3. Drain có thời hạn: chờ HTTP đang bay, hàng đợi và transaction đang mở.
        let drain_timed_out = tokio::time::timeout(self.cfg.drain_timeout, self.tracker.wait())
            .await
            .is_err();
        if drain_timed_out {
            // Huỷ task: transaction đang mở bị drop nên rollback, tin đó chưa lưu offset nên sẽ đến lại.
            self.worker_abort.abort();
            self.http_abort.abort();
            warn!(
                "shutdown phase=3 drain=timeout after_ms={} action=abort",
                self.cfg.drain_timeout.as_millis()
            );
        } else {
            info!("shutdown phase=3 drained");
        }
        let done_after_drain = metrics.done();

        // 4. Commit offset cuối: chỉ gồm tin worker đã làm xong và lưu.
        let commit = match self.consumer.commit_consumer_state(CommitMode::Sync) {
            Ok(()) | Err(KafkaError::ConsumerCommit(RDKafkaErrorCode::NoOffset)) => Ok(()),
            Err(err) => Err(err.to_string()),
        };
        info!("shutdown phase=4 offsets=committed result={commit:?}");

        // 5. Relay dọn nốt outbox (worker đã xong nên không còn dòng mới), rồi flush producer.
        self.relay_stop.cancel();
        let mut relay = self.relay;
        let relay_timed_out = tokio::time::timeout(self.cfg.drain_timeout, &mut relay)
            .await
            .is_err();
        if relay_timed_out {
            relay.abort();
            warn!("shutdown phase=5 relay=timeout action=abort");
        }
        let flush = self.producer.flush(Duration::from_secs(5));
        info!("shutdown phase=5 relay=stopped producer_flush={flush:?}");

        // 6. Đóng pool sau cùng, vì relay và HTTP còn dùng nó ở trên.
        let outbox_pending =
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM outbox WHERE published_at IS NULL")
                .fetch_one(&self.pool)
                .await
                .ok();
        self.consumer.unsubscribe();
        self.pool.close().await;
        info!("shutdown phase=6 pool=closed");

        ShutdownReport {
            done_in_grace: done_after_grace - done_before_grace,
            done_in_drain: done_after_drain - done_after_grace,
            total_ms: started.elapsed().as_millis(),
            drain_timed_out,
            relay_timed_out,
            commit,
            outbox_pending,
        }
    }
}
