//! Chặn runtime: `std::thread::sleep` trong task async so với `spawn_blocking`.
//!
//! Đo trên runtime chỉ có MỘT worker thread. Một task «đồng hồ» đáng lẽ tick mỗi 10 ms;
//! ta ghi khoảng cách lớn nhất giữa hai tick liên tiếp trong lúc một task khác làm việc nặng.

use std::time::{Duration, Instant};

use tokio::runtime::Builder;
use tokio::time::{MissedTickBehavior, interval, sleep};
use tokio_util::sync::CancellationToken;

/// Cách chạy việc nặng 300 ms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Work {
    /// `std::thread::sleep` ngay trong task async: giữ luôn worker thread.
    BlockInTask,
    /// `tokio::task::spawn_blocking`: việc nặng chạy ở thread của blocking pool.
    SpawnBlocking,
}

/// Số đo một lần chạy.
#[derive(Debug, Clone, Copy)]
pub struct Stall {
    /// Khoảng cách lớn nhất giữa hai tick liên tiếp.
    pub max_gap: Duration,
    /// Số tick đã thấy.
    pub ticks: u32,
}

/// Độ dài việc nặng.
pub const HEAVY: Duration = Duration::from_millis(300);

/// Chạy việc nặng theo `work` trên runtime một worker và đo độ trễ của task đồng hồ.
/// Dùng đồng hồ thật (không phải đồng hồ ảo), nên con số thay đổi theo máy.
pub fn measure(work: Work) -> Stall {
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_time()
        .build()
        .expect("dựng được runtime");
    runtime.block_on(async move {
        let stop = CancellationToken::new();
        let ticker = tokio::spawn(ticker(stop.clone()));
        sleep(Duration::from_millis(60)).await; // cho đồng hồ chạy vài nhịp trước
        let heavy = tokio::spawn(async move {
            match work {
                Work::BlockInTask => std::thread::sleep(HEAVY),
                Work::SpawnBlocking => {
                    tokio::task::spawn_blocking(|| std::thread::sleep(HEAVY))
                        .await
                        .expect("việc nặng không panic");
                }
            }
        });
        heavy.await.expect("task nặng không panic");
        sleep(Duration::from_millis(60)).await;
        stop.cancel();
        ticker.await.expect("task đồng hồ không panic")
    })
}

/// Khoảng tick nhỏ nhất qua tối đa `attempts` lần đo, dừng sớm khi đã dưới `limit`.
///
/// Máy tải nặng có thể làm xấu một lần đo bất kỳ, nên ngưỡng trên không nên dựa vào một lần đo.
/// Nếu việc nặng thật sự giữ worker thì lần đo nào cũng từ 300 ms trở lên, nên lấy lần tốt nhất
/// không che được lỗi.
pub fn best_gap(work: Work, limit: Duration, attempts: u32) -> Duration {
    let mut best = Duration::MAX;
    for _ in 0..attempts {
        best = best.min(measure(work).max_gap);
        if best < limit {
            break;
        }
    }
    best
}

async fn ticker(stop: CancellationToken) -> Stall {
    let mut every = interval(Duration::from_millis(10));
    every.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut last = Instant::now();
    let mut stall = Stall {
        max_gap: Duration::ZERO,
        ticks: 0,
    };
    loop {
        tokio::select! {
            _ = every.tick() => {
                let now = Instant::now();
                stall.max_gap = stall.max_gap.max(now - last);
                stall.ticks += 1;
                last = now;
            }
            () = stop.cancelled() => {
                stall.max_gap = stall.max_gap.max(last.elapsed());
                return stall;
            }
        }
    }
}
