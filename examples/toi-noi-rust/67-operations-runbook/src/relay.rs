//! Relay outbox → Kafka: đọc dòng chưa gửi, gửi, chờ broker xác nhận rồi mới đánh dấu `published_at`.

use anyhow::Result;
use jiff::Timestamp;
use rdkafka::message::{Header, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use sqlx::{PgPool, Row};

const BATCH: i64 = 100;

/// Một lượt relay: gửi tối đa [`BATCH`] dòng chưa gửi, đánh dấu `published_at = at`.
/// Trả số dòng đã gửi; 0 nghĩa là outbox đã sạch.
///
/// Giữ khoá dòng (`FOR UPDATE SKIP LOCKED`) suốt lượt nên hai relay không gửi trùng một dòng.
/// Gửi lỗi giữa chừng thì transaction rollback và dòng đã gửi sẽ được gửi lại (at-least-once).
///
/// # Errors
/// Lỗi PostgreSQL hoặc broker không xác nhận.
pub async fn relay_once(pool: &PgPool, producer: &FutureProducer, at: Timestamp) -> Result<usize> {
    let mut tx = pool.begin().await?;
    let rows = sqlx::query(
        "SELECT id, topic, key, payload::text AS payload FROM outbox \
         WHERE published_at IS NULL ORDER BY id LIMIT $1 FOR UPDATE SKIP LOCKED",
    )
    .bind(BATCH)
    .fetch_all(&mut *tx)
    .await?;
    for row in &rows {
        let id: i64 = row.get("id");
        let topic: String = row.get("topic");
        let key: String = row.get("key");
        let payload: String = row.get("payload");
        let event_id = format!("ledger:{id}");
        let headers = OwnedHeaders::new().insert(Header {
            key: "event-id",
            value: Some(&event_id),
        });
        producer
            .send(
                FutureRecord::to(&topic)
                    .key(&key)
                    .payload(&payload)
                    .headers(headers),
                Timeout::Never,
            )
            .await
            .map_err(|(err, _)| err)?;
        sqlx::query("UPDATE outbox SET published_at = $2::timestamptz WHERE id = $1")
            .bind(id)
            .bind(at.to_string())
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(rows.len())
}

/// Chạy relay cho tới khi outbox sạch. Trả tổng số dòng đã gửi.
///
/// # Errors
/// Như [`relay_once`].
pub async fn drain(pool: &PgPool, producer: &FutureProducer, at: Timestamp) -> Result<usize> {
    let mut total = 0;
    loop {
        let sent = relay_once(pool, producer, at).await?;
        if sent == 0 {
            return Ok(total);
        }
        total += sent;
    }
}
