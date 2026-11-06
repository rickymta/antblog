//! `Mutex` so với `RwLock` khi đọc nhiều. Chỉ chạy qua `--bench`, không nằm trong `--verify`
//! vì thời gian phụ thuộc máy.
//!
//! Cả ba kịch bản dùng `std::sync` và không có `.await` trong vòng lặp: mục đích là đo khóa,
//! không đo scheduler.

use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;

use ledger_core::{AccountId, Ledger, LedgerError};

use crate::fixtures::trip_entry;

/// Một kịch bản đo.
#[derive(Debug, Clone, Copy)]
pub enum Scenario {
    /// Mọi thao tác là `balance` (duyệt cả sổ, vài chục micro giây).
    HeavyRead,
    /// Như trên nhưng cứ 20 thao tác có 1 thao tác ghi (ghi lại bút toán đã có).
    HeavyReadWithWrites,
    /// Mọi thao tác là `len()`: phần việc trong khóa gần như bằng 0.
    TinyRead,
}

impl Scenario {
    /// Tên hiển thị.
    pub fn name(self) -> &'static str {
        match self {
            Scenario::HeavyRead => "đọc nặng (balance, 100% đọc)",
            Scenario::HeavyReadWithWrites => "đọc nặng + 1/20 ghi",
            Scenario::TinyRead => "đọc nhẹ (len, 100% đọc)",
        }
    }
}

/// Cấu hình đo.
#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// Số bút toán có sẵn trong sổ.
    pub entries: u32,
    /// Số worker thread của runtime và cũng là số task.
    pub tasks: usize,
    /// Số thao tác mỗi task.
    pub ops_per_task: usize,
    /// Số lần lặp; kết quả là trung vị.
    pub repeats: usize,
}

/// Trung vị thời gian (mili giây) của hai loại khóa.
#[derive(Debug, Clone, Copy)]
pub struct Timing {
    /// Với `std::sync::Mutex`.
    pub mutex_ms: f64,
    /// Với `std::sync::RwLock`.
    pub rwlock_ms: f64,
}

/// Sổ cái có `entries` bút toán thu cước khác nhau.
///
/// # Errors
/// Lỗi dựng hoặc ghi bút toán.
pub fn seeded_ledger(entries: u32) -> Result<Ledger, LedgerError> {
    let mut ledger = Ledger::new();
    for n in 1..=entries {
        ledger.post(trip_entry(n, 5)?)?;
    }
    Ok(ledger)
}

/// Chạy `tasks` task, mỗi task gọi `op(i)` đủ `ops_per_task` lần; trả thời gian chờ tất cả xong.
fn timed<F>(cfg: Config, rt: &tokio::runtime::Runtime, make_op: impl Fn() -> F) -> f64
where
    F: FnMut(usize) + Send + 'static,
{
    let start = Instant::now();
    rt.block_on(async {
        let mut handles = Vec::new();
        for _ in 0..cfg.tasks {
            let mut op = make_op();
            handles.push(tokio::spawn(async move {
                for i in 0..cfg.ops_per_task {
                    op(i);
                }
            }));
        }
        for handle in handles {
            handle.await.expect("task không panic");
        }
    });
    start.elapsed().as_secs_f64() * 1000.0
}

fn median(mut samples: Vec<f64>) -> f64 {
    samples.sort_by(f64::total_cmp);
    samples[samples.len() / 2]
}

/// Đo một kịch bản với cả hai loại khóa.
///
/// # Errors
/// Lỗi dựng sổ cái hoặc runtime.
pub fn measure(scenario: Scenario, cfg: Config) -> Result<Timing, Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(cfg.tasks)
        .build()?;
    let driver = AccountId::new("driver:DR-001")?;
    let replay = trip_entry(1, 5)?;
    let mutex = Arc::new(Mutex::new(seeded_ledger(cfg.entries)?));
    let rwlock = Arc::new(RwLock::new(seeded_ledger(cfg.entries)?));

    let mut mutex_runs = Vec::new();
    let mut rwlock_runs = Vec::new();
    for _ in 0..cfg.repeats {
        mutex_runs.push(timed(cfg, &rt, || {
            let lock = Arc::clone(&mutex);
            let (driver, replay) = (driver.clone(), replay.clone());
            move |i| match scenario {
                Scenario::TinyRead => {
                    std::hint::black_box(lock.lock().expect("không poisoned").len());
                }
                Scenario::HeavyRead => {
                    std::hint::black_box(
                        lock.lock().expect("không poisoned").balance(&driver).ok(),
                    );
                }
                Scenario::HeavyReadWithWrites => {
                    let mut guard = lock.lock().expect("không poisoned");
                    if i % 20 == 0 {
                        std::hint::black_box(guard.post(replay.clone()).is_ok());
                    } else {
                        std::hint::black_box(guard.balance(&driver).ok());
                    }
                }
            }
        }));
        rwlock_runs.push(timed(cfg, &rt, || {
            let lock = Arc::clone(&rwlock);
            let (driver, replay) = (driver.clone(), replay.clone());
            move |i| match scenario {
                Scenario::TinyRead => {
                    std::hint::black_box(lock.read().expect("không poisoned").len());
                }
                Scenario::HeavyRead => {
                    std::hint::black_box(
                        lock.read().expect("không poisoned").balance(&driver).ok(),
                    );
                }
                Scenario::HeavyReadWithWrites => {
                    if i % 20 == 0 {
                        let mut guard = lock.write().expect("không poisoned");
                        std::hint::black_box(guard.post(replay.clone()).is_ok());
                    } else {
                        std::hint::black_box(
                            lock.read().expect("không poisoned").balance(&driver).ok(),
                        );
                    }
                }
            }
        }));
    }
    Ok(Timing {
        mutex_ms: median(mutex_runs),
        rwlock_ms: median(rwlock_runs),
    })
}
