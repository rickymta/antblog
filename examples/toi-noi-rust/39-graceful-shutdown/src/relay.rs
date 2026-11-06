//! Relay outbox: đọc dòng chưa phát, gửi lên Kafka, đánh dấu đã phát. At-least-once.

use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;

use anyhow::Result;
use rdkafka::producer::{FutureProducer, FutureRecord};
use sqlx::Row;
use sqlx::postgres::PgPool;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::metrics::Metrics;

/// Chạy đến khi `stop` bị huỷ VÀ outbox không còn dòng chưa phát (lượt dọn cuối).
pub async fn run_relay(
    pool: PgPool,
    producer: FutureProducer,
    stop: CancellationToken,
    metrics: Arc<Metrics>,
) {
    loop {
        match relay_batch(&pool, &producer, &metrics).await {
            Ok(0) => {
                if stop.is_cancelled() {
                    break; // đã huỷ và hết việc
                }
                tokio::select! {
                    () = stop.cancelled() => {}
                    () = tokio::time::sleep(Duration::from_millis(50)) => {}
                }
            }
            Ok(_) => {}
            Err(err) => {
                warn!("relay lỗi: {err:#}");
                if stop.is_cancelled() {
                    break; // đang tắt mà vẫn lỗi: bỏ, dòng chưa phát nằm lại outbox cho lần chạy sau
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
    info!("relay dừng: outbox đã dọn hoặc để lại cho lần chạy sau");
}

/// Phát tối đa 20 dòng theo thứ tự id. Gửi xong mới `UPDATE`: chết giữa hai bước thì phát lại.
async fn relay_batch(pool: &PgPool, producer: &FutureProducer, metrics: &Metrics) -> Result<usize> {
    let rows = sqlx::query(
        "SELECT id, topic, key, payload::text AS payload FROM outbox \
         WHERE published_at IS NULL ORDER BY id LIMIT 20",
    )
    .fetch_all(pool)
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
            .map_err(|(err, _msg)| err)?;
        sqlx::query("UPDATE outbox SET published_at = now() WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        metrics.relayed.fetch_add(1, Relaxed);
    }
    Ok(rows.len())
}
