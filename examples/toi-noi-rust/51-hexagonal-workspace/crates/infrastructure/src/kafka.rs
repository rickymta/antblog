//! Adapter Kafka tối thiểu cho `EventPublisher`. Chỉ được biên dịch khi bật feature `kafka`.

use std::time::Duration;

use anyhow::{Context, bail};
use application::{EventPublisher, PublishError};
use domain::EntryPosted;
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::error::RDKafkaErrorCode;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::{ClientConfig, Message};
use uuid::Uuid;

use crate::wire;

/// Phát `LedgerEntryPosted` lên một topic Kafka, khoá partition là `idempotency_key`.
pub struct KafkaPublisher {
    producer: FutureProducer,
    topic: String,
}

impl KafkaPublisher {
    /// Producer an toàn cho sổ cái: `acks=all` và idempotent producer.
    ///
    /// # Errors
    /// Cấu hình client lỗi.
    pub fn new(brokers: &str, topic: &str) -> anyhow::Result<KafkaPublisher> {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("acks", "all")
            .set("enable.idempotence", "true")
            .set("message.timeout.ms", "10000")
            .create()
            .context("tạo producer")?;
        Ok(KafkaPublisher {
            producer,
            topic: topic.to_string(),
        })
    }
}

impl EventPublisher for KafkaPublisher {
    async fn publish(&self, event: &EntryPosted) -> Result<(), PublishError> {
        let payload = wire::to_json(event)?;
        self.producer
            .send(
                FutureRecord::to(&self.topic)
                    .key(&event.idempotency_key)
                    .payload(&payload),
                Duration::from_secs(10),
            )
            .await
            .map_err(|(err, _)| PublishError(Box::new(err)))?;
        Ok(())
    }
}

/// Topic và group riêng của một lần chạy: `lab.51.<uuid>.entry-posted`, `lab-51-<uuid>`.
pub struct KafkaLab {
    brokers: String,
    /// Tên topic của lần chạy.
    pub topic: String,
    group: String,
}

fn admin(brokers: &str) -> anyhow::Result<AdminClient<DefaultClientContext>> {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .create()
        .context("tạo admin client")
}

impl KafkaLab {
    /// Đọc `TOI_NOI_KAFKA`, tạo topic một partition và chờ tới khi có leader.
    ///
    /// # Errors
    /// Thiếu biến, không kết nối được broker, hoặc topic không có leader sau 30 giây.
    pub async fn setup() -> anyhow::Result<KafkaLab> {
        let brokers = std::env::var("TOI_NOI_KAFKA")
            .context("thiếu biến TOI_NOI_KAFKA (xem examples/toi-noi-rust/infra/README.md)")?;
        let id = Uuid::new_v4().simple().to_string();
        let lab = KafkaLab {
            topic: format!("lab.51.{id}.entry-posted"),
            group: format!("lab-51-{id}"),
            brokers,
        };
        let topic = NewTopic::new(&lab.topic, 1, TopicReplication::Fixed(1));
        let results = admin(&lab.brokers)?
            .create_topics(
                &[topic],
                &AdminOptions::new().request_timeout(Some(Duration::from_secs(10))),
            )
            .await
            .context("create_topics: không kết nối được Kafka?")?;
        for result in results {
            if let Err((name, code)) = result {
                bail!("không tạo được topic {name}: {code}");
            }
        }
        if let Err(e) = lab.wait_for_leader().await {
            let _ = lab.cleanup().await;
            return Err(e);
        }
        Ok(lab)
    }

    /// Địa chỉ broker (để dựng `KafkaPublisher`).
    pub fn brokers(&self) -> &str {
        &self.brokers
    }

    // `create_topics` trả về trước khi metadata lan hết: chờ tới khi topic có leader, có hạn chót.
    async fn wait_for_leader(&self) -> anyhow::Result<()> {
        let client = admin(&self.brokers)?;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
        loop {
            let metadata = tokio::task::block_in_place(|| {
                client
                    .inner()
                    .fetch_metadata(Some(&self.topic), Duration::from_secs(5))
            });
            let ready = metadata.is_ok_and(|m| {
                m.topics().iter().any(|t| {
                    t.name() == self.topic
                        && t.error().is_none()
                        && !t.partitions().is_empty()
                        && t.partitions()
                            .iter()
                            .all(|p| p.error().is_none() && p.leader() >= 0)
                })
            });
            if ready {
                return Ok(());
            }
            if tokio::time::Instant::now() >= deadline {
                bail!("topic {} chưa có leader sau 30 giây", self.topic);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Đọc đúng `count` message từ đầu topic (khoá, JSON). Quá 30 giây là lỗi, không phải assert thời gian.
    ///
    /// # Errors
    /// Không subscribe được, lỗi nhận, hoặc không đủ message trước hạn chót.
    pub async fn read(&self, count: usize) -> anyhow::Result<Vec<(String, String)>> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &self.brokers)
            .set("group.id", &self.group)
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .set("session.timeout.ms", "10000")
            .create()
            .context("tạo consumer")?;
        consumer.subscribe(&[&self.topic]).context("subscribe")?;
        let mut out = Vec::with_capacity(count);
        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
        while out.len() < count {
            let msg = tokio::time::timeout_at(deadline, consumer.recv())
                .await
                .with_context(|| format!("mới nhận {}/{count} message", out.len()))?
                .context("recv")?;
            let key = msg
                .key()
                .map(|k| String::from_utf8_lossy(k).into_owned())
                .unwrap_or_default();
            let payload = msg
                .payload()
                .map(|p| String::from_utf8_lossy(p).into_owned())
                .unwrap_or_default();
            out.push((key, payload));
        }
        Ok(out)
    }

    /// Xoá topic và consumer group của lần chạy.
    ///
    /// # Errors
    /// Không kết nối được broker.
    pub async fn cleanup(&self) -> anyhow::Result<()> {
        let admin = admin(&self.brokers)?;
        let opts = AdminOptions::new().request_timeout(Some(Duration::from_secs(10)));
        admin
            .delete_topics(&[self.topic.as_str()], &opts)
            .await
            .context("delete_topics")?;
        // Group chỉ xoá được khi đã rỗng; consumer vừa đóng có thể chưa kịp rời nên thử lại vài lần.
        for attempt in 0..10 {
            let results = admin
                .delete_groups(&[self.group.as_str()], &opts)
                .await
                .context("delete_groups")?;
            let done = results.iter().all(|result| {
                !matches!(result, Err((_, code)) if *code != RDKafkaErrorCode::GroupIdNotFound)
            });
            if done {
                break;
            }
            if attempt < 9 {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
        Ok(())
    }
}
