//! Lắp consumer và relay thành một service chạy trong tiến trình (test) hoặc làm `serve` (tiến trình con).

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio_util::sync::{CancellationToken, DropGuard};

use crate::consumer::{self, ConsumerSettings};
use crate::pg::PgStore;
use crate::progress::Progress;
use crate::{Fault, kafka, relay};

/// Mọi thứ service cần biết lúc chạy.
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// URL PostgreSQL (`TOI_NOI_PG_URL`).
    pub pg_url: String,
    /// Schema dùng làm `search_path`; `None` là mặc định của role.
    pub schema: Option<String>,
    /// Broker (`TOI_NOI_KAFKA`).
    pub kafka: String,
    /// Topic `TripCompleted`.
    pub topic_trips: String,
    /// Topic `PaymentCaptured`.
    pub topic_captures: String,
    /// Topic `LedgerEntryPosted`.
    pub topic_out: String,
    /// Consumer group.
    pub group: String,
    /// Điểm chết giả lập (chỉ tiến trình con của test đặt).
    pub fault: Fault,
    /// Có chạy relay không (tiến trình con của ca «chết» tắt nó để outbox còn nguyên lúc chết).
    pub run_relay: bool,
}

impl ServiceConfig {
    /// Đọc từ biến môi trường thật của tiến trình.
    ///
    /// # Errors
    /// Thiếu `TOI_NOI_PG_URL` hoặc `TOI_NOI_KAFKA`, hoặc `PAYMENT_FAULT` lạ.
    pub fn from_env() -> Result<ServiceConfig> {
        let get = |name: &str| std::env::var(name).ok().filter(|v| !v.trim().is_empty());
        let need = |name: &str| get(name).ok_or_else(|| anyhow!("thiếu biến môi trường {name}"));
        let prefix = get("PAYMENT_TOPIC_PREFIX").unwrap_or_else(|| "payment".to_string());
        let fault = match get("PAYMENT_FAULT") {
            None => Fault::None,
            Some(raw) => Fault::parse(&raw).ok_or_else(|| anyhow!("PAYMENT_FAULT lạ: {raw}"))?,
        };
        Ok(ServiceConfig {
            pg_url: need("TOI_NOI_PG_URL")?,
            schema: get("PAYMENT_PG_SCHEMA"),
            kafka: need("TOI_NOI_KAFKA")?,
            topic_trips: format!("{prefix}.trip.completed"),
            topic_captures: format!("{prefix}.payment.captured"),
            topic_out: format!("{prefix}.ledger.entry-posted"),
            group: get("PAYMENT_GROUP").unwrap_or_else(|| "payment-use-case".to_string()),
            fault,
            run_relay: get("PAYMENT_RELAY").is_none_or(|v| v != "0"),
        })
    }
}

/// Service đang chạy: một consumer và (tuỳ chọn) một relay.
pub struct Service {
    /// Cổng vào PostgreSQL (test đọc sổ qua đây).
    pub store: PgStore,
    /// Bộ đếm tiến độ để test chờ theo điều kiện.
    pub progress: Arc<Progress>,
    consumer_token: CancellationToken,
    consumer: JoinHandle<Result<()>>,
    relay_token: CancellationToken,
    relay: Option<JoinHandle<()>>,
    // Kịch bản lỗi giữa chừng làm rơi `Service` mà không gọi `stop`: vẫn hủy task để consumer rời group.
    _cancel_on_drop: [DropGuard; 2],
}

impl Service {
    /// Kết nối PostgreSQL, rồi chạy consumer và relay.
    ///
    /// # Errors
    /// Không kết nối được PostgreSQL hoặc không tạo được producer.
    pub async fn start(cfg: &ServiceConfig) -> Result<Service> {
        let store = PgStore::connect(
            &cfg.pg_url,
            cfg.schema.as_deref(),
            &cfg.topic_out,
            cfg.fault,
        )
        .await
        .context("kết nối PostgreSQL")?;
        let progress = Arc::new(Progress::new());
        let wake = Arc::new(Notify::new());
        // Hai token riêng: consumer dừng trước, relay dừng sau để phát nốt outbox.
        let consumer_token = CancellationToken::new();
        let relay_token = CancellationToken::new();
        let relay = if cfg.run_relay {
            Some(tokio::spawn(relay::run(
                store.clone(),
                kafka::producer(&cfg.kafka)?,
                Arc::clone(&progress),
                Arc::clone(&wake),
                Duration::from_millis(500),
                relay_token.clone(),
            )))
        } else {
            None
        };
        let settings = ConsumerSettings {
            kafka: cfg.kafka.clone(),
            group: cfg.group.clone(),
            topic_trips: cfg.topic_trips.clone(),
            topic_captures: cfg.topic_captures.clone(),
        };
        let consumer = {
            let (store, progress, token, fault) = (
                store.clone(),
                Arc::clone(&progress),
                consumer_token.clone(),
                cfg.fault,
            );
            tokio::spawn(async move {
                consumer::run(&settings, &store, &progress, &wake, &token, fault).await
            })
        };
        let cancel_on_drop = [
            consumer_token.clone().drop_guard(),
            relay_token.clone().drop_guard(),
        ];
        Ok(Service {
            store,
            progress,
            consumer_token,
            consumer,
            relay_token,
            relay,
            _cancel_on_drop: cancel_on_drop,
        })
    }

    /// Tắt có thứ tự: dừng consumer (message đang làm được làm xong), rồi hủy relay, vòng relay
    /// phát nốt outbox trước khi thoát.
    ///
    /// # Errors
    /// Consumer lỗi hoặc task bị panic.
    pub async fn stop(self) -> Result<()> {
        self.consumer_token.cancel();
        let consumed = self.consumer.await.context("task consumer panic")?;
        self.relay_token.cancel();
        if let Some(relay) = self.relay {
            relay.await.context("task relay panic")?;
        }
        consumed
    }
}
