//! Replay an toàn: đọc một KHOẢNG offset bằng group riêng (không commit), có dry-run và giới hạn tốc độ.
//!
//! Replay đi qua đúng hàm xử lý của consumer (`apply_trip_completed`), nên idempotency do inbox và
//! `idempotency_key` bảo đảm, không do công cụ replay tự nghĩ ra thêm một cơ chế.

use std::time::Duration;

use anyhow::{Context, Result, bail};
use rdkafka::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::topic_partition_list::{Offset, TopicPartitionList};
use sqlx::PgPool;

use crate::kafka::watermarks;
use crate::store::{ApplyCtx, Outcome, apply_trip_completed, preview_trip_completed};

/// Nơi chờ giữa hai message. Production dùng [`RealSleeper`]; test và `--verify` dùng
/// [`RecordingSleeper`] để kiểm tốc độ mà không phải chờ.
pub trait Sleeper: Send {
    /// Chờ `by`.
    fn sleep(&mut self, by: Duration) -> impl Future<Output = ()> + Send;
}

/// Chờ thật bằng `tokio::time::sleep`.
#[derive(Debug, Default)]
pub struct RealSleeper;

impl Sleeper for RealSleeper {
    fn sleep(&mut self, by: Duration) -> impl Future<Output = ()> + Send {
        tokio::time::sleep(by)
    }
}

/// Chỉ ghi lại các lần xin chờ, không chờ.
#[derive(Debug, Default)]
pub struct RecordingSleeper {
    /// Các khoảng chờ đã được xin, theo thứ tự.
    pub waits: Vec<Duration>,
}

impl Sleeper for RecordingSleeper {
    fn sleep(&mut self, by: Duration) -> impl Future<Output = ()> + Send {
        self.waits.push(by);
        async {}
    }
}

/// Giãn tốc độ: chờ `interval` trước mọi message trừ message đầu tiên.
pub async fn throttle<S: Sleeper>(sleeper: &mut S, interval: Duration, processed: u64) {
    if processed > 0 && !interval.is_zero() {
        sleeper.sleep(interval).await;
    }
}

/// Chế độ chạy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Chỉ đọc và phân loại, không ghi gì vào PostgreSQL hay Kafka.
    DryRun,
    /// Xử lý thật.
    Apply,
}

/// Yêu cầu replay.
#[derive(Debug, Clone)]
pub struct ReplayRequest {
    /// Topic nguồn (DLQ hoặc topic gốc).
    pub topic: String,
    /// Offset đầu, tính cả.
    pub from_offset: i64,
    /// Offset cuối, KHÔNG tính.
    pub to_offset: i64,
    /// Group RIÊNG của lần replay. Replay không commit, group này không bao giờ có offset.
    pub group: String,
    /// Khoảng chờ giữa hai message (giới hạn tốc độ).
    pub interval: Duration,
    /// Dry-run hay thật.
    pub mode: Mode,
}

/// Một message không xử lý được.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    /// Offset của message.
    pub offset: i64,
    /// Lý do.
    pub reason: String,
}

/// Kết quả replay.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReplayReport {
    /// Số message đã đọc trong khoảng.
    pub read: u64,
    /// Sẽ ghi (dry-run) hoặc đã ghi (thật) bút toán mới.
    pub apply: u64,
    /// Đã xử lý trước đó (inbox hoặc `idempotency_key` đã có), bỏ qua.
    pub duplicate: u64,
    /// Không xử lý được.
    pub failed: Vec<Failure>,
}

/// Replay một khoảng offset của partition 0.
///
/// # Errors
/// Khoảng không hợp lệ (rỗng hoặc vượt watermark), lỗi Kafka, lỗi hạ tầng PostgreSQL.
pub async fn replay<S: Sleeper>(
    pool: &PgPool,
    brokers: &str,
    req: &ReplayRequest,
    ctx: &ApplyCtx<'_>,
    sleeper: &mut S,
) -> Result<ReplayReport> {
    let (low, high) = watermarks(brokers, &req.topic).await?;
    if req.from_offset >= req.to_offset {
        bail!(
            "khoảng rỗng hoặc ngược: [{}, {})",
            req.from_offset,
            req.to_offset
        );
    }
    if req.from_offset < low || req.to_offset > high {
        bail!(
            "khoảng [{}, {}) vượt phạm vi topic [{low}, {high})",
            req.from_offset,
            req.to_offset
        );
    }
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", &req.group)
        .set("enable.auto.commit", "false")
        .create()?;
    let mut tpl = TopicPartitionList::new();
    tpl.add_partition_offset(&req.topic, 0, Offset::Offset(req.from_offset))?;
    consumer.assign(&tpl)?;

    let mut report = ReplayReport::default();
    let mut processed = 0u64;
    loop {
        let message = tokio::time::timeout(Duration::from_secs(60), consumer.recv())
            .await
            .context("60 giây không có message")??;
        let offset = message.offset();
        if offset >= req.to_offset {
            break;
        }
        throttle(sleeper, req.interval, processed).await;
        processed += 1;
        report.read += 1;
        let payload = message.payload().unwrap_or_default();
        let outcome = match req.mode {
            Mode::DryRun => preview_trip_completed(pool, ctx, payload).await,
            Mode::Apply => apply_trip_completed(pool, ctx, payload).await,
        };
        match outcome {
            Ok(Outcome::Applied) => report.apply += 1,
            Ok(Outcome::Duplicate | Outcome::EntryAlreadyExists) => report.duplicate += 1,
            Err(err) if err.is_permanent() => report.failed.push(Failure {
                offset,
                reason: err.to_string(),
            }),
            Err(err) => return Err(err.into()),
        }
        if offset + 1 >= req.to_offset {
            break;
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn throttle_spaces_messages_by_the_interval_with_no_wait_before_the_first() {
        let start = tokio::time::Instant::now();
        let mut sleeper = RealSleeper;
        for processed in 0..4 {
            throttle(&mut sleeper, Duration::from_millis(200), processed).await;
        }
        // 4 message → 3 khoảng chờ; đồng hồ tạm dừng của tokio tiến đúng bằng thời gian được xin.
        assert_eq!(start.elapsed(), Duration::from_millis(600));
    }

    #[tokio::test]
    async fn zero_interval_never_asks_to_wait() {
        let mut sleeper = RecordingSleeper::default();
        for processed in 0..5 {
            throttle(&mut sleeper, Duration::ZERO, processed).await;
        }
        assert!(sleeper.waits.is_empty());
        throttle(&mut sleeper, Duration::from_millis(50), 3).await;
        assert_eq!(sleeper.waits, [Duration::from_millis(50)]);
    }
}
