//! Hai cách ghi kép SAI: lab dùng chúng để chứng minh lỗi, không dùng ở đâu khác.

use anyhow::Result;
use ledger_core::JournalEntry;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use sqlx::PgPool;

use crate::events::{LedgerEntryPosted, PostingDto, SCHEMA_VERSION};
use crate::store::{entry_posted, insert_entry};

/// Ghi DB, commit, RỒI mới gửi Kafka. `die_between` gọi `process::exit(137)` giữa hai bước:
/// giả lập bị kill. Destructor không chạy nên `Drop` của producer không flush, nhưng đây là
/// tiến trình tự thoát, không phải SIGKILL (hook `atexit` vẫn chạy).
///
/// # Errors
/// Lỗi SQL hoặc lỗi gửi Kafka.
pub async fn db_then_kafka(
    pool: &PgPool,
    producer: &FutureProducer,
    topic: &str,
    entry: &JournalEntry,
    die_between: bool,
) -> Result<()> {
    let mut tx = pool.begin().await?;
    let inserted = insert_entry(&mut tx, entry).await?;
    tx.commit().await?;
    if die_between {
        // Bút toán đã commit; sự kiện chưa rời process.
        std::process::exit(137);
    }
    if let Some(inserted) = inserted {
        let payload = serde_json::to_string(&entry_posted(&inserted, entry))?;
        producer
            .send(
                FutureRecord::to(topic)
                    .key(entry.idempotency_key())
                    .payload(&payload),
                Timeout::Never,
            )
            .await
            .map_err(|(err, _)| err)?;
    }
    Ok(())
}

/// Gửi Kafka (đã được broker xác nhận), RỒI mới ghi DB. Ghi DB lỗi thì thông điệp
/// vẫn nằm trong topic.
///
/// # Errors
/// Lỗi gửi Kafka hoặc lỗi SQL (cố ý gây ra bằng tài khoản không tồn tại).
pub async fn kafka_then_db(
    pool: &PgPool,
    producer: &FutureProducer,
    topic: &str,
    entry: &JournalEntry,
) -> Result<()> {
    // Id và giờ do người gửi tự đặt trước khi DB cấp: chính là chỗ sai của cách này.
    let event = LedgerEntryPosted {
        schema_version: SCHEMA_VERSION,
        entry_id: "JE-000001".to_string(),
        idempotency_key: entry.idempotency_key().to_string(),
        postings: entry
            .postings()
            .iter()
            .map(|p| PostingDto {
                account: p.account.as_str().to_string(),
                amount_dong: p.amount.dong(),
            })
            .collect(),
        posted_at: "2026-11-06T02:15:00.000000Z".to_string(),
    };
    let payload = serde_json::to_string(&event)?;
    producer
        .send(
            FutureRecord::to(topic)
                .key(entry.idempotency_key())
                .payload(&payload),
            Timeout::Never,
        )
        .await
        .map_err(|(err, _)| err)?;
    let mut tx = pool.begin().await?;
    insert_entry(&mut tx, entry).await?;
    tx.commit().await?;
    Ok(())
}
