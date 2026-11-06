//! Consumer `TripCompleted` có DLQ (bản rút gọn của lab 37): lỗi dữ liệu đi DLQ rồi commit offset,
//! lỗi hạ tầng làm hàm trả lỗi và KHÔNG commit, nên message sẽ đến lại.

use std::time::Duration;

use anyhow::{Context, Result, bail};
use rdkafka::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::{Header, Message, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use sqlx::PgPool;

use crate::clock::Clock;
use crate::store::{ApplyCtx, Outcome, apply_trip_completed};

/// Tham số một lần chạy consumer.
#[derive(Debug, Clone)]
pub struct ConsumerConfig {
    /// Consumer group.
    pub group: String,
    /// Topic nguồn.
    pub topic: String,
    /// Topic DLQ.
    pub dlq_topic: String,
    /// Topic mà dòng outbox gửi tới.
    pub out_topic: String,
    /// `schemaVersion` mà bản consumer này hiểu (đổi giá trị này = «triển khai bản sửa»).
    pub accepted_versions: Vec<u32>,
    /// Dừng sau khi đã xử lý (kể cả đưa DLQ) từng này message.
    pub stop_after: usize,
}

/// Số liệu một lần chạy.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ConsumerStats {
    /// Message tạo bút toán mới.
    pub applied: usize,
    /// Message bị inbox chặn.
    pub duplicates: usize,
    /// Message sang DLQ.
    pub dead_lettered: usize,
}

/// Đưa message hỏng sang DLQ: giữ nguyên byte gốc, ghi lý do và nguồn vào header.
async fn dead_letter(
    producer: &FutureProducer,
    dlq_topic: &str,
    message: &impl Message,
    error: &str,
    failed_at: &str,
) -> Result<()> {
    let partition = message.partition().to_string();
    let offset = message.offset().to_string();
    let headers = OwnedHeaders::new()
        .insert(Header {
            key: "x-error",
            value: Some(error),
        })
        .insert(Header {
            key: "x-source-topic",
            value: Some(message.topic()),
        })
        .insert(Header {
            key: "x-source-partition",
            value: Some(&partition),
        })
        .insert(Header {
            key: "x-source-offset",
            value: Some(&offset),
        })
        .insert(Header {
            key: "x-failed-at",
            value: Some(failed_at),
        });
    let mut record: FutureRecord<'_, [u8], [u8]> = FutureRecord::to(dlq_topic).headers(headers);
    if let Some(key) = message.key() {
        record = record.key(key);
    }
    if let Some(payload) = message.payload() {
        record = record.payload(payload);
    }
    producer
        .send(record, Timeout::Never)
        .await
        .map_err(|(err, _)| err)?;
    Ok(())
}

/// Chạy consumer cho tới khi đã xử lý `stop_after` message. DLQ được xác nhận TRƯỚC khi commit offset.
///
/// # Errors
/// Lỗi Kafka, lỗi hạ tầng PostgreSQL, hoặc 60 giây không có message (hạn chót an toàn).
pub async fn run_consumer(
    pool: &PgPool,
    brokers: &str,
    producer: &FutureProducer,
    clock: &dyn Clock,
    cfg: &ConsumerConfig,
) -> Result<ConsumerStats> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", &cfg.group)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .create()?;
    consumer.subscribe(&[&cfg.topic])?;
    let mut stats = ConsumerStats::default();
    let mut handled = 0;
    while handled < cfg.stop_after {
        let received = tokio::time::timeout(Duration::from_secs(60), consumer.recv())
            .await
            .context("60 giây không có message")?;
        let message = received?;
        let ctx = ApplyCtx {
            accepted: &cfg.accepted_versions,
            at: clock.now(),
            out_topic: &cfg.out_topic,
        };
        match apply_trip_completed(pool, &ctx, message.payload().unwrap_or_default()).await {
            Ok(Outcome::Duplicate) => stats.duplicates += 1,
            Ok(Outcome::Applied | Outcome::EntryAlreadyExists) => stats.applied += 1,
            Err(err) if err.is_permanent() => {
                dead_letter(
                    producer,
                    &cfg.dlq_topic,
                    &message,
                    &err.to_string(),
                    &ctx.at.to_string(),
                )
                .await?;
                stats.dead_lettered += 1;
            }
            Err(err) => bail!("lỗi hạ tầng, không commit offset: {err}"),
        }
        consumer.commit_message(&message, CommitMode::Sync)?;
        handled += 1;
    }
    Ok(stats)
}
