//! Đồng hồ truyền vào: lab không có chỗ nào gọi `Timestamp::now` (clippy.toml cấm).
//! Mọi thời điểm nghiệp vụ, tuổi outbox và cửa sổ burn rate đều tính từ giá trị này.

use std::sync::atomic::{AtomicI64, Ordering};

use jiff::{SignedDuration, Timestamp};

/// Nguồn thời gian. Lab chỉ có bản chỉnh tay; service thật sẽ có thêm bản đồng hồ hệ thống.
pub trait Clock: Send + Sync {
    /// Thời điểm hiện tại theo đồng hồ này.
    fn now(&self) -> Timestamp;
}

/// Đồng hồ chỉnh tay, độ phân giải giây: chỉ chạy khi gọi [`ManualClock::advance`].
#[derive(Debug)]
pub struct ManualClock(AtomicI64);

impl ManualClock {
    /// Mốc cố định của lab: 06/11/2026 08:00:00 UTC, tròn phút để chia bucket cho dễ đọc.
    pub fn lab_start() -> ManualClock {
        let start: Timestamp = "2026-11-06T08:00:00Z".parse().expect("hằng hợp lệ");
        ManualClock(AtomicI64::new(start.as_second()))
    }

    /// Cho đồng hồ chạy tới.
    pub fn advance(&self, by: SignedDuration) {
        self.0.fetch_add(by.as_secs(), Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Timestamp {
        Timestamp::from_second(self.0.load(Ordering::SeqCst)).expect("giây hợp lệ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_moves_only_when_told() {
        let clock = ManualClock::lab_start();
        let t0 = clock.now();
        assert_eq!(clock.now(), t0);
        clock.advance(SignedDuration::from_mins(15));
        assert_eq!(clock.now().as_second() - t0.as_second(), 900);
    }
}
