//! Consumer `TripCompleted`: commit offset SAU khi PostgreSQL đã commit (at-least-once).

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use rdkafka::consumer::{
    BaseConsumer, CommitMode, Consumer, ConsumerContext, Rebalance, StreamConsumer,
};
use rdkafka::{ClientConfig, ClientContext, Message};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, info, info_span, warn};

use crate::config::{Config, Fault};
use crate::events::TripCompleted;
use crate::state::{Readiness, Stats};
use crate::store::{Applied, Store, StoreError};

/// Context để biết lúc nào consumer thật sự có partition (đó mới là "sẵn sàng").
pub struct ReadyContext {
    ready: Arc<Readiness>,
}

impl ClientContext for ReadyContext {}

impl ConsumerContext for ReadyContext {
    fn post_rebalance(&self, _consumer: &BaseConsumer<Self>, rebalance: &Rebalance<'_>) {
        match rebalance {
            Rebalance::Assign(partitions) => self.ready.set_assigned(partitions.count() > 0),
            Rebalance::Revoke(_) | Rebalance::Error(_) => self.ready.set_assigned(false),
        }
    }
}

/// Vì sao consumer dừng.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumerExit {
    /// Nhận tín hiệu tắt; message đang xử lý đã xong và đã commit.
    Stopped,
    /// Điểm chết giả lập: dừng NGAY, không commit offset, không dọn gì.
    Crashed,
}

enum Handled {
    /// Xử lý xong (hoặc bỏ qua vĩnh viễn): commit offset.
    Commit,
    /// Chết giả lập.
    Crash,
    /// Có tín hiệu tắt giữa lúc thử lại: không commit, message sẽ đến lại lần sau.
    Cancelled,
}

/// Chạy vòng consume cho tới khi bị hủy hoặc chết giả lập.
///
/// # Errors
/// Không tạo/subscribe được consumer.
pub async fn run(
    cfg: &Config,
    store: Store,
    stats: Arc<Stats>,
    ready: Arc<Readiness>,
    relay_wake: Arc<Notify>,
    token: CancellationToken,
) -> Result<ConsumerExit> {
    let consumer: StreamConsumer<ReadyContext> = ClientConfig::new()
        .set("bootstrap.servers", &cfg.kafka)
        .set("group.id", &cfg.group)
        .set("enable.auto.commit", "false") // tự commit offset sau khi DB đã commit
        .set("auto.offset.reset", "earliest")
        .set("session.timeout.ms", "6000")
        .create_with_context(ReadyContext { ready })
        .context("tạo consumer")?;
    consumer.subscribe(&[&cfg.topic_in]).context("subscribe")?;
    info!(topic = %cfg.topic_in, group = %cfg.group, "consumer đã subscribe");

    loop {
        // `recv` an toàn khi bị hủy giữa chừng; còn việc xử lý bên dưới KHÔNG nằm trong select.
        let received = tokio::select! {
            biased;
            () = token.cancelled() => return Ok(ConsumerExit::Stopped),
            received = consumer.recv() => received,
        };
        let msg = match received {
            Ok(msg) => msg,
            Err(err) => {
                warn!(error = %err, "lỗi khi nhận message");
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
        };
        let handled = handle(
            cfg.fault,
            &store,
            &stats,
            &relay_wake,
            &token,
            msg.payload().unwrap_or_default(),
        )
        .instrument(info_span!(
            "consume",
            partition = msg.partition(),
            offset = msg.offset()
        ))
        .await;
        match handled {
            Handled::Commit => consumer
                .commit_message(&msg, CommitMode::Sync)
                .context("commit offset")?,
            Handled::Crash => return Ok(ConsumerExit::Crashed),
            Handled::Cancelled => return Ok(ConsumerExit::Stopped),
        }
    }
}

async fn handle(
    fault: Fault,
    store: &Store,
    stats: &Stats,
    relay_wake: &Notify,
    token: &CancellationToken,
    payload: &[u8],
) -> Handled {
    let event: TripCompleted = match serde_json::from_slice(payload) {
        Ok(event) => event,
        Err(err) => {
            // Chưa có DLQ (bài 37 nói tới): ghi log rồi bỏ qua để không kẹt cả partition.
            warn!(error = %err, "TripCompleted sai dạng, bỏ qua");
            Stats::bump(&stats.poison, 1);
            return Handled::Commit;
        }
    };
    let mut attempt = 0u32;
    loop {
        match store.apply_trip_completed(&event).await {
            Ok(Applied::Posted(posted)) => {
                info!(trip_id = %event.trip_id, entry_id = %posted.entry_id, "đã ghi bút toán");
                Stats::bump(&stats.processed, 1);
                relay_wake.notify_one();
                // Mô phỏng process chết ĐÚNG khoảng này: DB đã commit, offset chưa.
                return if fault == Fault::AfterDbCommitBeforeOffset {
                    Handled::Crash
                } else {
                    Handled::Commit
                };
            }
            Ok(Applied::Duplicate) => {
                info!(trip_id = %event.trip_id, "sự kiện đã xử lý trước đó, bỏ qua");
                Stats::bump(&stats.duplicates, 1);
                return Handled::Commit;
            }
            Err(StoreError::InjectedCrash) => return Handled::Crash,
            Err(err) if err.is_permanent() => {
                warn!(trip_id = %event.trip_id, error = %err, "sự kiện sai nội dung, bỏ qua");
                Stats::bump(&stats.poison, 1);
                return Handled::Commit;
            }
            Err(err) => {
                // Lỗi hạ tầng: thử lại CHÍNH message này, không commit offset, lùi dần tới 2 giây.
                attempt += 1;
                let backoff = Duration::from_millis(100 * u64::from(attempt.min(20)));
                warn!(attempt, error = %err, "lỗi hạ tầng, thử lại");
                tokio::select! {
                    () = token.cancelled() => return Handled::Cancelled,
                    () = tokio::time::sleep(backoff) => {}
                }
            }
        }
    }
}
