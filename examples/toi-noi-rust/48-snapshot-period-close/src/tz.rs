//! Ranh giới ngày và tháng theo giờ Việt Nam, tính bằng `jiff`, trả về `Timestamp` (UTC).

use jiff::Timestamp;
use jiff::civil::Date;
use jiff::{Error, Span};

/// Múi giờ nghiệp vụ của Tới Nơi.
pub const VN_TZ: &str = "Asia/Ho_Chi_Minh";

/// Ranh giới nửa mở `[from, to)` của một ngày dương lịch theo giờ Việt Nam.
/// `to` là 00:00 của ngày kế tiếp: đó cũng là mốc `as_of` của snapshot cuối ngày.
///
/// # Errors
/// Lỗi của `jiff` nếu ngày nằm ngoài khoảng hỗ trợ hoặc thiếu cơ sở dữ liệu múi giờ.
pub fn day_bounds(day: Date) -> Result<(Timestamp, Timestamp), Error> {
    let from = day.in_tz(VN_TZ)?.timestamp();
    let to = day.tomorrow()?.in_tz(VN_TZ)?.timestamp();
    Ok((from, to))
}

/// Ranh giới nửa mở `[from, to)` của một tháng dương lịch theo giờ Việt Nam.
///
/// # Errors
/// Lỗi của `jiff` nếu năm/tháng không hợp lệ.
pub fn month_bounds(year: i16, month: i8) -> Result<(Timestamp, Timestamp), Error> {
    let first = Date::new(year, month, 1)?;
    let next = first.checked_add(Span::new().months(1))?;
    Ok((
        first.in_tz(VN_TZ)?.timestamp(),
        next.in_tz(VN_TZ)?.timestamp(),
    ))
}

/// Đọc một thời điểm RFC 3339 có offset, ví dụ `2026-11-05T23:58:00+07:00`.
///
/// # Errors
/// Lỗi nếu chuỗi sai dạng.
pub fn ts(raw: &str) -> Result<Timestamp, Error> {
    raw.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_bounds_are_half_open_in_vietnam_time() {
        let (from, to) = day_bounds(Date::constant(2026, 11, 5)).unwrap();
        // 00:00 giờ Việt Nam là 17:00 UTC của ngày hôm trước.
        assert_eq!(from.to_string(), "2026-11-04T17:00:00Z");
        assert_eq!(to.to_string(), "2026-11-05T17:00:00Z");
    }

    #[test]
    fn month_bounds_cross_year() {
        let (from, to) = month_bounds(2026, 12).unwrap();
        assert_eq!(from.to_string(), "2026-11-30T17:00:00Z");
        assert_eq!(to.to_string(), "2026-12-31T17:00:00Z");
    }

    #[test]
    fn parses_offset_timestamp() {
        let t = ts("2026-11-05T23:58:00+07:00").unwrap();
        assert_eq!(t.to_string(), "2026-11-05T16:58:00Z");
    }
}
