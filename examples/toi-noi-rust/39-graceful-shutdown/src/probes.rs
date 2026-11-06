//! `/readyz` kiểm phụ thuộc thật; `/healthz` không kiểm gì ngoài việc tiến trình còn trả lời.

use std::time::Duration;

use rdkafka::producer::{FutureProducer, Producer};
use sqlx::postgres::PgPool;

/// Thời gian chờ mỗi phụ thuộc. Phải nhỏ hơn `timeoutSeconds` của probe bên ngoài.
const PROBE_TIMEOUT: Duration = Duration::from_millis(1000);

/// Kết quả kiểm từng phụ thuộc: `Err` mang lý do.
#[derive(Debug, Clone)]
pub struct Readiness {
    /// PostgreSQL trả lời `SELECT 1`.
    pub postgres: Result<(), String>,
    /// Broker trả metadata của topic phát ra.
    pub kafka: Result<(), String>,
}

impl Readiness {
    /// Sẵn sàng khi cả hai phụ thuộc đều ổn.
    pub fn is_ready(&self) -> bool {
        self.postgres.is_ok() && self.kafka.is_ok()
    }
}

/// Gói các kết nối cần kiểm.
pub struct Probes {
    pool: PgPool,
    producer: FutureProducer,
    topic: String,
}

impl Probes {
    /// Dùng lại pool và producer của service, không mở kết nối riêng.
    pub fn new(pool: PgPool, producer: FutureProducer, topic: String) -> Probes {
        Probes {
            pool,
            producer,
            topic,
        }
    }

    /// Kiểm PostgreSQL và Kafka, mỗi bên tối đa `PROBE_TIMEOUT`.
    pub async fn check(&self) -> Readiness {
        let postgres =
            match tokio::time::timeout(PROBE_TIMEOUT, sqlx::query("SELECT 1").execute(&self.pool))
                .await
            {
                Ok(Ok(_)) => Ok(()),
                Ok(Err(err)) => Err(err.to_string()),
                Err(_) => Err("quá thời gian chờ".to_string()),
            };
        let producer = self.producer.clone();
        let topic = self.topic.clone();
        // `fetch_metadata` là lời gọi chặn: đưa sang luồng blocking, không chặn runtime.
        let kafka = tokio::task::spawn_blocking(move || {
            producer
                .client()
                .fetch_metadata(Some(&topic), PROBE_TIMEOUT)
                .map(|_| ())
                .map_err(|err| err.to_string())
        })
        .await
        .unwrap_or_else(|err| Err(err.to_string()));
        Readiness { postgres, kafka }
    }
}
