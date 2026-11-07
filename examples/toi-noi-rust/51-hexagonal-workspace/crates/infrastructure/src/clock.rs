//! Adapter đồng hồ cho port `Clock`.

use std::time::{SystemTime, UNIX_EPOCH};

use application::Clock;

/// Đồng hồ hệ thống (dùng khi chạy thật).
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|d| i64::try_from(d.as_secs()).ok())
            .unwrap_or(0)
    }
}

/// Đồng hồ cố định (dùng trong test và `--verify`: kết quả không phụ thuộc giờ máy).
#[derive(Debug, Clone, Copy)]
pub struct FixedClock(pub i64);

impl Clock for FixedClock {
    fn now_unix(&self) -> i64 {
        self.0
    }
}
