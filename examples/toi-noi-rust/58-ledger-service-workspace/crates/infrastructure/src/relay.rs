//! Relay outbox → Kafka: đọc dòng chưa phát, gửi, chờ broker xác nhận rồi mới đánh dấu.

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use rdkafka::producer::{FutureProducer, FutureRecord};
use sqlx::{PgPool, Row};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

use crate::stats::Stats;

const BATCH: i64 = 20;

/// Phát tối đa [`BATCH`] dòng chưa phát. Trả số dòng đã phát và đánh dấu.
///
/// Giữ khoá dòng (`FOR UPDATE SKIP LOCKED`) trong suốt batch nên hai relay không phát trùng một dòng.
/// Gửi lỗi giữa chừng thì transaction rollback: dòng đã gửi trước đó sẽ được gửi lại lần sau.
/// Đó là at-least-once, người nhận khử trùng theo `idempotencyKey`.
///
/// # Errors
/// Lỗi PostgreSQL hoặc broker không xác nhận.
pub async fn publish_batch(pool: &PgPool, producer: &FutureProducer) -> Result<u64> {
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
        producer
            .send(
                FutureRecord::to(&topic).key(&key).payload(&payload),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(err, _)| err)?;
        sqlx::query("UPDATE outbox SET published_at = now() WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        debug!(outbox_id = id, %topic, "đã phát");
    }
    tx.commit().await?;
    Ok(rows.len() as u64)
}

/// Vòng relay. Được đánh thức bởi `wake` (consumer vừa ghi) hoặc sau mỗi `poll`.
/// Khi bị hủy, relay phát nốt những gì còn trong outbox rồi mới thoát.
pub async fn run(
    pool: PgPool,
    producer: FutureProducer,
    stats: Arc<Stats>,
    wake: Arc<Notify>,
    poll: Duration,
    token: CancellationToken,
) {
    loop {
        let published = match publish_batch(&pool, &producer).await {
            Ok(n) => n,
            Err(err) => {
                warn!(error = %err, "relay lỗi, sẽ thử lại");
                0
            }
        };
        if published > 0 {
            Stats::bump(&stats.relayed, published);
            continue; // còn việc thì làm tiếp, chưa ngủ
        }
        if token.is_cancelled() {
            break; // outbox đã rỗng (hoặc đang lỗi): thoát
        }
        tokio::select! {
            () = token.cancelled() => {}
            () = wake.notified() => {}
            () = tokio::time::sleep(poll) => {}
        }
    }
}
