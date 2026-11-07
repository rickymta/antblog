//! Consumer hai topic (`TripCompleted`, `PaymentCaptured`): đổi message thành `Command`, giao cho
//! `handle`, và chỉ commit offset SAU khi PostgreSQL đã commit (at-least-once, bài 36 và 40).

use std::time::Duration;

use anyhow::{Context, Result};
use application::{Command, Outcome, handle};
use domain::{PaymentCaptured, TripCompleted};
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::BorrowedMessage;
use rdkafka::{ClientConfig, Message};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use crate::pg::PgStore;
use crate::progress::Progress;
use crate::{Fault, die};

/// Cấu hình consumer.
#[derive(Debug, Clone)]
pub struct ConsumerSettings {
    /// Broker.
    pub kafka: String,
    /// Consumer group.
    pub group: String,
    /// Topic `TripCompleted`.
    pub topic_trips: String,
    /// Topic `PaymentCaptured`.
    pub topic_captures: String,
}

enum Handled {
    Commit,
    Cancelled,
}

fn decode(settings: &ConsumerSettings, msg: &BorrowedMessage<'_>) -> Result<Command, String> {
    let payload = msg.payload().ok_or("payload rỗng")?;
    if msg.topic() == settings.topic_trips {
        serde_json::from_slice::<TripCompleted>(payload)
            .map(Command::TripCompleted)
            .map_err(|e| format!("TripCompleted sai dạng: {e}"))
    } else {
        serde_json::from_slice::<PaymentCaptured>(payload)
            .map(Command::PaymentCaptured)
            .map_err(|e| format!("PaymentCaptured sai dạng: {e}"))
    }
}

/// Chạy vòng consume cho tới khi bị hủy. Message đang xử lý được làm xong và commit trước khi thoát.
///
/// # Errors
/// Không tạo hoặc subscribe được consumer, hoặc commit offset lỗi.
pub async fn run(
    settings: &ConsumerSettings,
    store: &PgStore,
    progress: &Progress,
    relay_wake: &Notify,
    token: &CancellationToken,
    fault: Fault,
) -> Result<()> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &settings.kafka)
        .set("group.id", &settings.group)
        .set("enable.auto.commit", "false") // tự commit offset sau khi DB đã commit
        .set("auto.offset.reset", "earliest")
        .set("session.timeout.ms", "6000")
        .create()
        .context("tạo consumer")?;
    consumer
        .subscribe(&[&settings.topic_trips, &settings.topic_captures])
        .context("subscribe")?;
    loop {
        // `recv` an toàn khi bị hủy giữa chừng; việc xử lý bên dưới KHÔNG nằm trong select.
        let received = tokio::select! {
            biased;
            () = token.cancelled() => return Ok(()),
            received = consumer.recv() => received,
        };
        let msg = match received {
            Ok(msg) => msg,
            Err(err) => {
                eprintln!("lỗi khi nhận message: {err}");
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
        };
        match process(settings, store, progress, relay_wake, token, fault, &msg).await {
            Handled::Commit => consumer
                .commit_message(&msg, CommitMode::Sync)
                .context("commit offset")?,
            Handled::Cancelled => return Ok(()),
        }
    }
}

async fn process(
    settings: &ConsumerSettings,
    store: &PgStore,
    progress: &Progress,
    relay_wake: &Notify,
    token: &CancellationToken,
    fault: Fault,
    msg: &BorrowedMessage<'_>,
) -> Handled {
    let command = match decode(settings, msg) {
        Ok(command) => command,
        Err(why) => {
            // Chưa có DLQ (bài 37 nói tới): ghi log rồi bỏ qua để không kẹt cả partition.
            eprintln!("bỏ qua message hỏng: {why}");
            progress.update(|c| {
                c.handled += 1;
                c.poison += 1;
            });
            return Handled::Commit;
        }
    };
    let mut attempt = 0u32;
    loop {
        match handle(store, &command).await {
            Ok(outcome) => {
                record(progress, relay_wake, &outcome);
                // Mô phỏng tiến trình chết ĐÚNG khoảng này: DB đã commit, offset chưa.
                if fault == Fault::AbortAfterCommit && !outcome.posted.is_empty() {
                    die("sau COMMIT DB, trước khi commit offset");
                }
                return Handled::Commit;
            }
            Err(err) if err.is_permanent() => {
                eprintln!("bỏ qua {} của {}: {err}", command.name(), command.trip_id());
                progress.update(|c| {
                    c.handled += 1;
                    c.poison += 1;
                });
                return Handled::Commit;
            }
            Err(err) => {
                // Lỗi hạ tầng: thử lại CHÍNH message này, không commit offset, lùi dần tới 2 giây.
                attempt += 1;
                let backoff = Duration::from_millis(100 * u64::from(attempt.min(20)));
                eprintln!("lỗi tạm thời (lần {attempt}), thử lại: {err}");
                tokio::select! {
                    () = token.cancelled() => return Handled::Cancelled,
                    () = tokio::time::sleep(backoff) => {}
                }
            }
        }
    }
}

fn record(progress: &Progress, relay_wake: &Notify, outcome: &Outcome) {
    progress.update(|c| {
        c.handled += 1;
        c.noops += u64::from(outcome.is_noop());
        c.conflicts += u64::from(outcome.conflicts);
        c.posted += outcome.posted.len() as u64;
    });
    if !outcome.posted.is_empty() {
        relay_wake.notify_one();
    }
}
