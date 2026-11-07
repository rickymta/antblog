//! Đồng hồ truyền vào: logic hết hạn không bao giờ gọi `now()` của máy hay của PostgreSQL.

use std::sync::atomic::{AtomicI64, Ordering};

use jiff::{SignedDuration, Timestamp};

/// Nguồn thời gian. Production dùng [`SystemClock`], test dùng [`ManualClock`].
pub trait Clock: Send + Sync {
    /// Thời điểm hiện tại theo đồng hồ này.
    fn now(&self) -> Timestamp;
}

/// Đồng hồ thật.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        Timestamp::now()
    }
}

/// Đồng hồ chỉnh tay, độ phân giải giây: chỉ chạy khi gọi [`ManualClock::advance`].
#[derive(Debug)]
pub struct ManualClock(AtomicI64);

impl ManualClock {
    /// Đặt đồng hồ ở thời điểm `at`.
    pub fn at(at: Timestamp) -> ManualClock {
        ManualClock(AtomicI64::new(at.as_second()))
    }

    /// Mốc cố định của lab: 06/11/2026 08:00 UTC.
    pub fn lab_start() -> ManualClock {
        // Chuỗi hằng, parse không thể lỗi.
        ManualClock::at("2026-11-06T08:00:00Z".parse().expect("hằng hợp lệ"))
    }

    /// Cho đồng hồ chạy tới `by` giây (làm tròn xuống giây).
    pub fn advance(&self, by: SignedDuration) {
        self.0.fetch_add(by.as_secs(), Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Timestamp {
        // Giá trị luôn đến từ `Timestamp::as_second` cộng độ lệch nhỏ, nằm trong miền hợp lệ.
        Timestamp::from_second(self.0.load(Ordering::SeqCst)).expect("giây hợp lệ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_clock_moves_only_when_told() {
        let clock = ManualClock::lab_start();
        let t0 = clock.now();
        assert_eq!(clock.now(), t0);
        clock.advance(SignedDuration::from_mins(15));
        assert_eq!(clock.now().as_second() - t0.as_second(), 900);
    }
}
