//! Worker: lấy tin từ hàng đợi, ghi bút toán, rồi mới lưu offset.

use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;

use rdkafka::consumer::{Consumer, StreamConsumer};
use sqlx::postgres::PgPool;
use tokio::sync::mpsc;
use tracing::{error, warn};

use crate::consumer::Job;
use crate::events::TripCompleted;
use crate::metrics::Metrics;
use crate::store::{Applied, apply_trip_completed};

/// Chạy đến khi mọi `Sender` bị drop VÀ hàng đợi cạn: đó chính là "drain".
pub async fn run_worker(
    mut rx: mpsc::Receiver<Job>,
    pool: PgPool,
    consumer: Arc<StreamConsumer>,
    metrics: Arc<Metrics>,
    topic_out: String,
    hold: Duration,
) {
    while let Some(job) = rx.recv().await {
        match serde_json::from_slice::<TripCompleted>(&job.payload) {
            Err(err) => {
                // Tin hỏng: ghi nhận và bỏ qua để không kẹt cả partition (bài 37 nói về DLQ).
                warn!(offset = job.offset, "tin không đọc được: {err}");
                metrics.poison.fetch_add(1, Relaxed);
            }
            Ok(event) => loop {
                metrics.in_flight.store(1, Relaxed);
                let applied = apply_trip_completed(&pool, &event, &topic_out, hold).await;
                metrics.in_flight.store(0, Relaxed);
                match applied {
                    Ok(Applied::Posted(_)) => {
                        metrics.posted.fetch_add(1, Relaxed);
                        break;
                    }
                    Ok(Applied::Duplicate) => {
                        metrics.duplicates.fetch_add(1, Relaxed);
                        break;
                    }
                    Err(err) => {
                        // Lỗi hạ tầng: thử lại CHÍNH tin này. Hàng đợi sẽ đầy và consumer tự pause.
                        error!(trip = %event.trip_id, "xử lý lỗi, thử lại: {err:#}");
                        tokio::time::sleep(Duration::from_millis(200)).await;
                    }
                }
            },
        }
        // Chỉ lưu offset SAU khi transaction đã commit. Truyền offset của CHÍNH tin vừa xong:
        // `rd_kafka_offset_store` tự lưu offset + 1. Cộng thêm 1 ở đây là bỏ lọt một tin.
        if let Err(err) = consumer.store_offset(&job.topic, job.partition, job.offset) {
            warn!("lưu offset lỗi: {err}");
        }
    }
}
