//! Đo nhanh bằng `Instant` để so debug với release. Không thay criterion: một lượt, không thống kê.

use std::hint::black_box;
use std::time::{Duration, Instant};

use ledger_core::AccountId;

use crate::workload::make_entry;
use ledger_core::Ledger;

/// Thời gian từng bước của một lượt đo.
#[derive(Debug, Clone, Copy)]
pub struct Timings {
    pub n: u64,
    pub create_and_post: Duration,
    pub balance_scan_each: Duration,
}

/// Tạo + ghi `n` bút toán, rồi quét số dư `scans` lần.
pub fn measure(n: u64, scans: u32) -> Timings {
    let start = Instant::now();
    let mut ledger = Ledger::new();
    for i in 1..=n {
        ledger
            .post(black_box(make_entry(i)))
            .expect("key không trùng");
    }
    let create_and_post = start.elapsed();

    let driver = AccountId::new("driver:DR-001").expect("hợp lệ");
    let start = Instant::now();
    for _ in 0..scans {
        black_box(ledger.balance(black_box(&driver)).expect("không tràn"));
    }
    let balance_scan_each = start.elapsed() / scans;
    Timings {
        n,
        create_and_post,
        balance_scan_each,
    }
}

/// Số đo cho `n` mặc định của bài.
pub fn measure_default() -> Timings {
    measure(100_000, 20)
}
