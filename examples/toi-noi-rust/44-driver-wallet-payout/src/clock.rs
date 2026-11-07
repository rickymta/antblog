//! Đồng hồ truyền vào: thời gian nghiệp vụ không bao giờ đọc `now()` ngầm.

use anyhow::Result;
use jiff::Timestamp;

/// Múi giờ của tài xế: ngày thu nhập và hạn mức rút tính theo múi giờ này.
pub const LOCAL_TZ: &str = "Asia/Ho_Chi_Minh";

/// Nguồn thời gian. Production truyền [`SystemClock`], test truyền [`FixedClock`].
pub trait Clock {
    /// Thời điểm hiện tại theo đồng hồ này.
    fn now(&self) -> Timestamp;
}

/// Đồng hồ hệ thống (không dùng trong lab, có để thấy chỗ cắm production).
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        Timestamp::now()
    }
}

/// Đồng hồ đứng yên ở một thời điểm, đổi bằng [`FixedClock::set`] thay cho `sleep`.
pub struct FixedClock(std::sync::Mutex<Timestamp>);

impl FixedClock {
    /// Đồng hồ đứng ở `rfc3339`, ví dụ `2026-11-06T10:00:00Z`.
    ///
    /// # Errors
    /// Chuỗi không phải thời điểm RFC 3339.
    pub fn at(rfc3339: &str) -> Result<FixedClock> {
        Ok(FixedClock(std::sync::Mutex::new(rfc3339.parse()?)))
    }

    /// Đặt lại đồng hồ.
    ///
    /// # Errors
    /// Chuỗi không phải thời điểm RFC 3339.
    pub fn set(&self, rfc3339: &str) -> Result<()> {
        *self.0.lock().expect("mutex") = rfc3339.parse()?;
        Ok(())
    }
}

impl FixedClock {
    /// Thời điểm hiện tại của đồng hồ (cùng giá trị với [`Clock::now`]).
    pub fn now_ts(&self) -> Timestamp {
        Clock::now(self)
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        *self.0.lock().expect("mutex")
    }
}

/// Ngày lịch theo giờ Việt Nam của một thời điểm, dạng `2026-11-06`.
///
/// # Errors
/// Không tìm thấy múi giờ trong cơ sở dữ liệu múi giờ của hệ thống.
pub fn local_date(at: Timestamp) -> Result<String> {
    Ok(at.in_tz(LOCAL_TZ)?.date().to_string())
}

/// Nửa mở `[0h hôm nay, 0h ngày mai)` theo giờ Việt Nam, đổi về UTC.
///
/// # Errors
/// Không tìm thấy múi giờ trong cơ sở dữ liệu múi giờ của hệ thống.
pub fn local_day_bounds(at: Timestamp) -> Result<(Timestamp, Timestamp)> {
    let start = at.in_tz(LOCAL_TZ)?.start_of_day()?;
    let end = start.tomorrow()?.start_of_day()?;
    Ok((start.timestamp(), end.timestamp()))
}
