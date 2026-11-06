//! Consumer idempotent: inbox + hiệu ứng trong một transaction, thử lại có giới hạn, DLQ.

use std::time::Duration;

use anyhow::Result;
use ledger_core::LedgerError;
use rdkafka::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::{Header, Headers, Message, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use sqlx::PgPool;

use crate::events::{EVENT_ID_HEADER, EventError, parse_entry_posted, parse_trip_completed};
use crate::store::{entry_from_trip, post_with_outbox};

/// Việc consumer làm với mỗi thông điệp.
#[derive(Debug, Clone)]
pub enum Handler {
    /// `TripCompleted` → bút toán + dòng outbox `LedgerEntryPosted` gửi tới `out_topic`.
    TripCompleted {
        /// Topic vật lý mà dòng outbox sẽ gửi tới.
        out_topic: String,
    },
    /// `LedgerEntryPosted` → cộng số tiền có dấu vào `account_balances`.
    EntryPosted,
}

/// Kết quả xử lý một thông điệp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Hiệu ứng mới đã ghi.
    Applied,
    /// Inbox đã có `event_id`: không làm gì.
    Duplicate,
    /// `event_id` mới nhưng `idempotency_key` của bút toán đã có: không ghi bút toán thứ hai.
    EntryAlreadyExists,
}

/// Lỗi xử lý một thông điệp.
#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    /// Thiếu header `event-id`.
    #[error("thiếu header event-id")]
    MissingEventId,
    /// Payload sai.
    #[error(transparent)]
    Event(#[from] EventError),
    /// Dữ liệu sự kiện không dựng được bút toán hợp lệ.
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    /// Lỗi PostgreSQL.
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

/// Xử lý MỘT thông điệp trong MỘT transaction.
///
/// Thứ tự: chèn `event_id` vào inbox (`ON CONFLICT DO NOTHING`) rồi mới làm hiệu ứng, cùng
/// transaction. Đã có trong inbox thì dừng. Hiệu ứng lỗi thì cả dòng inbox cũng rollback,
/// nên lần giao lại sau được xử lý chứ không bị coi là đã xong. `use_inbox = false` chỉ để
/// chứng minh điều gì xảy ra khi thiếu inbox.
///
/// # Errors
/// [`ProcessError`]; transaction bị drop nên tự rollback.
pub async fn process_message(
    pool: &PgPool,
    handler: &Handler,
    use_inbox: bool,
    event_id: &str,
    payload: &[u8],
) -> Result<Outcome, ProcessError> {
    let mut tx = pool.begin().await?;
    if use_inbox {
        let inserted = sqlx::query(
            "INSERT INTO inbox (event_id) VALUES ($1) ON CONFLICT (event_id) DO NOTHING",
        )
        .bind(event_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();
        if inserted == 0 {
            tx.rollback().await?;
            return Ok(Outcome::Duplicate);
        }
    }
    let outcome = match handler {
        Handler::TripCompleted { out_topic } => {
            let event = parse_trip_completed(payload)?;
            let entry = entry_from_trip(&event)?;
            match post_with_outbox(&mut tx, &entry, out_topic).await? {
                Some(_) => Outcome::Applied,
                None => Outcome::EntryAlreadyExists,
            }
        }
        Handler::EntryPosted => {
            let event = parse_entry_posted(payload)?;
            for posting in &event.postings {
                sqlx::query(
                    "INSERT INTO account_balances (account_id, balance_dong, version) \
                     VALUES ($1, $2, 1) \
                     ON CONFLICT (account_id) DO UPDATE SET \
                         balance_dong = account_balances.balance_dong + EXCLUDED.balance_dong, \
                         version = account_balances.version + 1, \
                         updated_at = now()",
                )
                .bind(&posting.account)
                .bind(posting.amount_dong)
                .execute(&mut *tx)
                .await?;
            }
            Outcome::Applied
        }
    };
    tx.commit().await?;
    Ok(outcome)
}

/// Tham số một lần chạy consumer.
#[derive(Debug, Clone)]
pub struct ConsumerConfig {
    /// Consumer group.
    pub group: String,
    /// Topic nguồn.
    pub topic: String,
    /// Topic DLQ.
    pub dlq_topic: String,
    /// Việc làm với mỗi thông điệp.
    pub handler: Handler,
    /// Dùng inbox hay không.
    pub use_inbox: bool,
    /// Số lần thử tối đa trước khi đưa vào DLQ.
    pub max_attempts: u32,
    /// Nghỉ giữa hai lần thử.
    pub retry_delay: Duration,
    /// Dừng sau khi đã xử lý (kể cả đưa DLQ) từng này thông điệp.
    pub stop_after: usize,
    /// `false`: không commit offset, giả lập chết trước khi commit.
    pub commit: bool,
    /// Dừng nếu từng này thời gian không có thông điệp.
    pub idle_timeout: Duration,
}

/// Số liệu một lần chạy.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Stats {
    /// Thông điệp tạo hiệu ứng mới.
    pub applied: usize,
    /// Thông điệp bị inbox chặn.
    pub duplicates: usize,
    /// Thông điệp mới nhưng bút toán đã có (khóa nghiệp vụ chặn).
    pub entry_replays: usize,
    /// Thông điệp đưa vào DLQ.
    pub dead_lettered: usize,
    /// Tổng số lần gọi xử lý, kể cả lần thử lại.
    pub attempts: usize,
}

/// Giá trị header dạng chuỗi.
fn header_value(message: &impl Message, name: &str) -> Option<String> {
    let headers = message.headers()?;
    (0..headers.count())
        .map(|i| headers.get(i))
        .find(|h| h.key == name)
        .and_then(|h| h.value)
        .map(|v| String::from_utf8_lossy(v).into_owned())
}

/// Đưa thông điệp hỏng sang DLQ, giữ nguyên byte gốc và ghi lý do vào header.
async fn dead_letter(
    producer: &FutureProducer,
    dlq_topic: &str,
    message: &impl Message,
    event_id: Option<&str>,
    error: &str,
    attempts: u32,
) -> Result<()> {
    let source = format!(
        "{}/{}/{}",
        message.topic(),
        message.partition(),
        message.offset()
    );
    let attempts = attempts.to_string();
    let headers = OwnedHeaders::new()
        .insert(Header {
            key: EVENT_ID_HEADER,
            value: Some(event_id.unwrap_or("unknown")),
        })
        .insert(Header {
            key: "x-error",
            value: Some(error),
        })
        .insert(Header {
            key: "x-attempts",
            value: Some(&attempts),
        })
        .insert(Header {
            key: "x-source",
            value: Some(&source),
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

/// Chạy consumer: mỗi thông điệp được thử tối đa `max_attempts` lần, hết lượt thì sang DLQ,
/// rồi commit offset để partition không kẹt ở thông điệp hỏng. DLQ được xác nhận TRƯỚC khi commit.
///
/// # Errors
/// Lỗi Kafka (nhận, gửi DLQ, commit). Lỗi xử lý từng thông điệp không làm hàm lỗi mà thành DLQ.
pub async fn run_consumer(
    pool: &PgPool,
    brokers: &str,
    producer: &FutureProducer,
    cfg: &ConsumerConfig,
) -> Result<Stats> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", &cfg.group)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .create()?;
    consumer.subscribe(&[&cfg.topic])?;

    let mut stats = Stats::default();
    let mut handled = 0;
    while handled < cfg.stop_after {
        let Ok(received) = tokio::time::timeout(cfg.idle_timeout, consumer.recv()).await else {
            break; // im lặng quá lâu
        };
        let message = received?;
        let event_id = header_value(&message, EVENT_ID_HEADER);
        let payload = message.payload().unwrap_or_default();

        let mut outcome = None;
        let mut last_error = String::new();
        for attempt in 1..=cfg.max_attempts {
            stats.attempts += 1;
            let result = match event_id.as_deref() {
                None => Err(ProcessError::MissingEventId),
                Some(id) => process_message(pool, &cfg.handler, cfg.use_inbox, id, payload).await,
            };
            match result {
                Ok(done) => {
                    outcome = Some(done);
                    break;
                }
                Err(err) => {
                    last_error = err.to_string();
                    if attempt < cfg.max_attempts {
                        tokio::time::sleep(cfg.retry_delay).await;
                    }
                }
            }
        }
        match outcome {
            Some(Outcome::Applied) => stats.applied += 1,
            Some(Outcome::Duplicate) => stats.duplicates += 1,
            Some(Outcome::EntryAlreadyExists) => stats.entry_replays += 1,
            None => {
                dead_letter(
                    producer,
                    &cfg.dlq_topic,
                    &message,
                    event_id.as_deref(),
                    &last_error,
                    cfg.max_attempts,
                )
                .await?;
                stats.dead_lettered += 1;
            }
        }
        if cfg.commit {
            consumer.commit_message(&message, CommitMode::Sync)?;
        }
        handled += 1;
    }
    Ok(stats)
}
