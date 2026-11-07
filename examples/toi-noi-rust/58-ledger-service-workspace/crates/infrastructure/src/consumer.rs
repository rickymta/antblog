//! Consumer `TripCompleted`: commit offset SAU khi PostgreSQL đã commit (at-least-once).
//! Adapter chỉ giải mã JSON rồi gọi use case; mọi luật nằm ở application và domain.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use ledger_application::model::Applied;
use ledger_application::readiness::Readiness;
use ledger_application::{Ledger, LedgerStore};
use rdkafka::consumer::{
    BaseConsumer, CommitMode, Consumer, ConsumerContext, Rebalance, StreamConsumer,
};
use rdkafka::{ClientConfig, ClientContext, Message};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, info, info_span, warn};

use crate::stats::Stats;
use crate::wire::TripCompleted;

/// Cấu hình riêng của consumer.
#[derive(Debug, Clone)]
pub struct ConsumerConfig {
    /// Danh sách broker.
    pub brokers: String,
    /// Topic `TripCompleted`.
    pub topic: String,
    /// Consumer group.
    pub group: String,
}

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

enum Handled {
    /// Xong (hoặc bỏ qua vĩnh viễn): commit offset.
    Commit,
    /// Có tín hiệu tắt giữa lúc thử lại: không commit, message sẽ đến lại lần sau.
    Cancelled,
}

/// Chạy vòng consume cho tới khi bị hủy. Message đang xử lý luôn xong trước khi dừng.
///
/// # Errors
/// Không tạo/subscribe được consumer, hoặc commit offset lỗi.
pub async fn run<S: LedgerStore>(
    cfg: &ConsumerConfig,
    ledger: Arc<Ledger<S>>,
    stats: Arc<Stats>,
    ready: Arc<Readiness>,
    relay_wake: Arc<Notify>,
    token: CancellationToken,
) -> Result<()> {
    let consumer: StreamConsumer<ReadyContext> = ClientConfig::new()
        .set("bootstrap.servers", &cfg.brokers)
        .set("group.id", &cfg.group)
        .set("enable.auto.commit", "false") // tự commit offset sau khi DB đã commit
        .set("auto.offset.reset", "earliest")
        .set("session.timeout.ms", "6000")
        .create_with_context(ReadyContext { ready })
        .context("tạo consumer")?;
    consumer.subscribe(&[&cfg.topic]).context("subscribe")?;
    info!(topic = %cfg.topic, group = %cfg.group, "consumer đã subscribe");

    loop {
        // `recv` an toàn khi bị hủy giữa chừng; còn việc xử lý bên dưới KHÔNG nằm trong select.
        let received = tokio::select! {
            biased;
            () = token.cancelled() => return Ok(()),
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
            &ledger,
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
            Handled::Cancelled => return Ok(()),
        }
    }
}

async fn handle<S: LedgerStore>(
    ledger: &Ledger<S>,
    stats: &Stats,
    relay_wake: &Notify,
    token: &CancellationToken,
    payload: &[u8],
) -> Handled {
    let command = match serde_json::from_slice::<TripCompleted>(payload)
        .map_err(|e| e.to_string())
        .and_then(|wire| wire.into_command().map_err(|e| e.to_string()))
    {
        Ok(command) => command,
        Err(err) => {
            // Chưa có DLQ: ghi log rồi bỏ qua để không kẹt cả partition.
            warn!(error = %err, "TripCompleted sai dạng, bỏ qua");
            Stats::bump(&stats.poison, 1);
            return Handled::Commit;
        }
    };
    let mut attempt = 0u32;
    loop {
        match ledger.on_trip_completed(&command).await {
            Ok(Applied::Posted(entries)) => {
                info!(trip_id = %command.trip_id, entries = entries.len(), "đã ghi bút toán");
                Stats::bump(&stats.processed, 1);
                relay_wake.notify_one();
                return Handled::Commit;
            }
            Ok(Applied::Duplicate) => {
                info!(trip_id = %command.trip_id, "sự kiện đã xử lý trước đó, bỏ qua");
                Stats::bump(&stats.duplicates, 1);
                return Handled::Commit;
            }
            Err(err) if err.is_permanent() => {
                warn!(trip_id = %command.trip_id, error = %err, "sự kiện sai nội dung, bỏ qua");
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
