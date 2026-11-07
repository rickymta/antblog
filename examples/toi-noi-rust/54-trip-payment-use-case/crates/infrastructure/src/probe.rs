//! Consumer đọc topic `ledger.entry-posted` như một service khác (Payment) sẽ làm.

use std::time::Duration;

use anyhow::{Context, Result, anyhow, ensure};
use domain::LedgerEntryPosted;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::{ClientConfig, Message};
use tokio::time::{Instant, timeout_at};

use crate::lab::LabEnv;

/// Người nghe `LedgerEntryPosted`.
pub struct Probe {
    consumer: StreamConsumer,
}

impl Probe {
    /// Subscribe topic ra của ca, đọc từ đầu topic.
    ///
    /// # Errors
    /// Không tạo hoặc subscribe được consumer.
    pub fn new(env: &LabEnv) -> Result<Probe> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &env.kafka)
            .set("group.id", format!("{}-probe", env.group))
            .set("auto.offset.reset", "earliest")
            .create()?;
        consumer.subscribe(&[&env.topic_out()])?;
        Ok(Probe { consumer })
    }

    /// Đọc đúng `count` sự kiện tiếp theo theo thứ tự đến. Có hạn chót, không ngủ cố định.
    ///
    /// # Errors
    /// Hết hạn, payload sai dạng hoặc key Kafka khác `idempotencyKey`.
    pub async fn next_events(
        &self,
        count: usize,
        within: Duration,
    ) -> Result<Vec<LedgerEntryPosted>> {
        let deadline = Instant::now() + within;
        let mut events = Vec::with_capacity(count);
        while events.len() < count {
            let received = timeout_at(deadline, self.consumer.recv())
                .await
                .map_err(|_| {
                    anyhow!(
                        "mới thấy {} trong {count} LedgerEntryPosted sau {within:?}",
                        events.len()
                    )
                })?;
            // Lỗi tạm của broker không làm test đỏ ngay: hạn chót mới là phán quyết.
            let Ok(msg) = received else { continue };
            let event: LedgerEntryPosted =
                serde_json::from_slice(msg.payload().context("payload rỗng")?)?;
            ensure!(
                msg.key() == Some(event.idempotency_key.as_bytes()),
                "key Kafka phải là idempotencyKey"
            );
            events.push(event);
        }
        Ok(events)
    }
}
