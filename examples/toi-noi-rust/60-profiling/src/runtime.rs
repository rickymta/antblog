//! Task chặn runtime và bản đã sửa: cùng việc, một bản không `.await` trong lúc quét sổ,
//! một bản trả quyền cho scheduler sau mỗi đoạn.

use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use ledger_core::{AccountId, Money};
use tokio::task::{JoinHandle, yield_now};

use crate::books::{ScanBook, sum_slice};

/// Số bút toán quét giữa hai lần trả quyền cho scheduler.
pub const CHUNK: usize = 256;

/// Hai cách viết handler đọc số dư.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Quét trong một lần poll duy nhất.
    Blocking,
    /// Quét từng đoạn `CHUNK` bút toán, `yield_now().await` sau mỗi đoạn.
    Chunked,
}

impl Mode {
    /// Tên task của handler, hiện trong tokio-console.
    pub fn handler_task(self) -> &'static str {
        match self {
            Mode::Blocking => "balance-blocking",
            Mode::Chunked => "balance-chunked",
        }
    }

    /// Tên task của heartbeat (giả lập endpoint `/health`) chạy cùng handler.
    pub fn heartbeat_task(self) -> &'static str {
        match self {
            Mode::Blocking => "heartbeat-blocking",
            Mode::Chunked => "heartbeat-chunked",
        }
    }
}

// Có tokio_unstable và console thì đặt tên task để tokio-console hiển thị; còn lại là spawn thường.
#[cfg(all(tokio_unstable, feature = "console"))]
fn spawn_named<F>(name: &str, future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::task::Builder::new()
        .name(name)
        .spawn(future)
        .expect("spawn task có tên")
}

#[cfg(not(all(tokio_unstable, feature = "console")))]
fn spawn_named<F>(_name: &str, future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::spawn(future)
}

/// Handler chặn: `rounds` lần quét toàn sổ liền nhau, không có `.await` nào.
/// Trên runtime một luồng, mọi task khác đứng chờ tới khi hàm này trả về.
pub async fn handler_blocking(book: ScanBook, account: AccountId, rounds: u32) -> Money {
    let mut last = Money::ZERO;
    for _ in 0..rounds {
        last = sum_slice(book.entries(), &account).0;
    }
    last
}

/// Cùng việc, nhưng sau mỗi đoạn `CHUNK` bút toán thì `yield_now().await`.
pub async fn handler_chunked(book: ScanBook, account: AccountId, rounds: u32) -> Money {
    let mut last = Money::ZERO;
    for _ in 0..rounds {
        let mut sum = Money::ZERO;
        for chunk in book.entries().chunks(CHUNK) {
            let part = sum_slice(chunk, &account).0;
            sum = sum.checked_add(part).expect("không tràn i64");
            yield_now().await;
        }
        last = sum;
    }
    last
}

/// Đếm nhịp cho tới khi `stop` bật, mỗi nhịp trả quyền cho scheduler.
async fn heartbeat(ticks: Arc<AtomicU64>, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::Relaxed) {
        ticks.fetch_add(1, Ordering::Relaxed);
        yield_now().await;
    }
}

/// Kết quả một kịch bản.
#[derive(Debug, PartialEq, Eq)]
pub struct Run {
    /// Số dư handler tính được.
    pub balance: Money,
    /// Số nhịp heartbeat đã chạy xong tại thời điểm handler kết thúc.
    pub ticks_at_finish: u64,
}

/// Chạy handler và heartbeat cạnh nhau. Chỉ có ý nghĩa trên runtime `current_thread`
/// (một luồng), nơi một task chặn là chặn tất cả.
pub async fn run_scenario(mode: Mode, book: ScanBook, account: AccountId, rounds: u32) -> Run {
    let ticks = Arc::new(AtomicU64::new(0));
    let stop = Arc::new(AtomicBool::new(false));

    let seen = Arc::clone(&ticks);
    let handler = spawn_named(mode.handler_task(), async move {
        let balance = match mode {
            Mode::Blocking => handler_blocking(book, account, rounds).await,
            Mode::Chunked => handler_chunked(book, account, rounds).await,
        };
        Run {
            balance,
            ticks_at_finish: seen.load(Ordering::Relaxed),
        }
    });
    let beat = spawn_named(
        mode.heartbeat_task(),
        heartbeat(Arc::clone(&ticks), Arc::clone(&stop)),
    );

    let run = handler.await.expect("handler không panic");
    stop.store(true, Ordering::Relaxed);
    beat.await.expect("heartbeat không panic");
    run
}
