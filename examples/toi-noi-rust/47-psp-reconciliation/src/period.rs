//! Kỳ đối soát, múi giờ nghiệp vụ và đồng hồ truyền vào.

use jiff::civil::Date;
use jiff::tz::TimeZone;
use jiff::{SignedDuration, Timestamp};

/// Múi giờ cắt kỳ của Tới Nơi. Thời điểm lưu UTC, ngày nghiệp vụ tính theo múi giờ này.
pub const BUSINESS_TZ: &str = "Asia/Ho_Chi_Minh";

/// Khoảng `[start, end)` tính bằng thời điểm tuyệt đối (UTC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period {
    /// Đầu kỳ (gồm).
    pub start: Timestamp,
    /// Cuối kỳ (không gồm).
    pub end: Timestamp,
}

impl Period {
    /// Kỳ là một ngày dương lịch theo giờ `Asia/Ho_Chi_Minh`: 00:00 hôm đó đến 00:00 hôm sau.
    ///
    /// # Errors
    /// Ngày vượt phạm vi của `jiff` hoặc không tìm thấy múi giờ trong tzdb của máy.
    pub fn business_day(date: Date) -> Result<Period, jiff::Error> {
        let start = date.in_tz(BUSINESS_TZ)?.timestamp();
        let end = date.tomorrow()?.in_tz(BUSINESS_TZ)?.timestamp();
        Ok(Period { start, end })
    }

    /// `t` có nằm trong kỳ không.
    pub fn contains(&self, t: Timestamp) -> bool {
        t >= self.start && t < self.end
    }

    /// Cửa sổ đọc sổ cái: kỳ mở rộng 24 giờ mỗi phía, để thấy giao dịch bị lệch qua ranh giới.
    ///
    /// # Errors
    /// Vượt phạm vi của `jiff`.
    pub fn window(&self) -> Result<(Timestamp, Timestamp), jiff::Error> {
        let day = SignedDuration::from_hours(24);
        Ok((self.start.checked_sub(day)?, self.end.checked_add(day)?))
    }
}

/// Đồng hồ truyền vào: job không tự gọi `Timestamp::now()`.
pub trait Clock: Send + Sync {
    /// Thời điểm hiện tại theo đồng hồ này.
    fn now(&self) -> Timestamp;
}

/// Đồng hồ hệ thống, dùng ở `main` thật.
#[derive(Debug, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        Timestamp::now()
    }
}

/// Đồng hồ đứng yên, dùng trong test và `--verify`.
#[derive(Debug, Clone, Copy)]
pub struct FixedClock(pub Timestamp);

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        self.0
    }
}

/// Ngày nghiệp vụ của sao kê cần đối soát: hôm qua theo giờ `Asia/Ho_Chi_Minh` của `now`.
///
/// # Errors
/// Không tìm thấy múi giờ hoặc vượt phạm vi của `jiff`.
pub fn previous_business_date(now: Timestamp) -> Result<Date, jiff::Error> {
    now.to_zoned(TimeZone::get(BUSINESS_TZ)?).date().yesterday()
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;

    use super::*;

    #[test]
    fn business_day_is_17h_utc_to_17h_utc() {
        let p = Period::business_day(date(2026, 11, 5)).unwrap();
        assert_eq!(p.start.to_string(), "2026-11-04T17:00:00Z");
        assert_eq!(p.end.to_string(), "2026-11-05T17:00:00Z");
    }

    #[test]
    fn contains_is_half_open() {
        let p = Period::business_day(date(2026, 11, 5)).unwrap();
        assert!(p.contains(p.start));
        assert!(!p.contains(p.end));
    }

    #[test]
    fn window_extends_one_day_each_side() {
        let p = Period::business_day(date(2026, 11, 5)).unwrap();
        let (a, b) = p.window().unwrap();
        assert_eq!(a.to_string(), "2026-11-03T17:00:00Z");
        assert_eq!(b.to_string(), "2026-11-06T17:00:00Z");
    }

    #[test]
    fn previous_date_follows_business_zone_not_utc() {
        // 18:30 UTC ngày 5 là 01:30 ngày 6 ở Việt Nam, nên «hôm qua» là ngày 5.
        let now: Timestamp = "2026-11-05T18:30:00Z".parse().unwrap();
        assert_eq!(previous_business_date(now).unwrap(), date(2026, 11, 5));
        // 16:30 UTC ngày 5 vẫn là 23:30 ngày 5 ở Việt Nam, nên «hôm qua» là ngày 4.
        let earlier: Timestamp = "2026-11-05T16:30:00Z".parse().unwrap();
        assert_eq!(previous_business_date(earlier).unwrap(), date(2026, 11, 4));
    }

    #[test]
    fn fixed_clock_never_moves() {
        let t: Timestamp = "2026-11-06T00:00:00Z".parse().unwrap();
        let clock = FixedClock(t);
        assert_eq!(clock.now(), clock.now());
    }
}
