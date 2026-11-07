//! Đồng hồ truyền vào để test không phụ thuộc giờ thật.

use std::sync::atomic::{AtomicI64, Ordering};

use jiff::Timestamp;

/// Nguồn thời gian của code nghiệp vụ.
pub trait Clock {
    /// Thời điểm hiện tại (UTC).
    fn now(&self) -> Timestamp;
}

/// Đồng hồ thật, dùng khi chạy production.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        Timestamp::now()
    }
}

/// Đồng hồ giả: mỗi lần gọi `now()` tiến thêm `step_ms` mili giây, bắt đầu từ `start`.
/// `step_ms = 0` cho đồng hồ đứng yên.
pub struct StepClock {
    start_ms: i64,
    step_ms: i64,
    calls: AtomicI64,
}

impl StepClock {
    /// Tạo đồng hồ giả bắt đầu ở `start`.
    #[must_use]
    pub fn new(start: Timestamp, step_ms: i64) -> StepClock {
        StepClock {
            start_ms: start.as_millisecond(),
            step_ms,
            calls: AtomicI64::new(0),
        }
    }
}

impl Clock for StepClock {
    fn now(&self) -> Timestamp {
        let n = self.calls.fetch_add(1, Ordering::SeqCst);
        Timestamp::from_millisecond(self.start_ms + self.step_ms * n)
            .expect("mốc thời gian giả nằm trong phạm vi")
    }
}
