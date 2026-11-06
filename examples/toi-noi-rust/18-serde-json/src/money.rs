//! Tiền VND: số nguyên `i64` đơn vị đồng, không bao giờ dùng float.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Số tiền có dấu, đơn vị đồng. Trường bên trong là private:
/// bên ngoài module chỉ tạo qua [`Money::from_dong`].
///
/// Không thể viết `Money(5)` từ crate khác, vì trường là private:
///
/// ```compile_fail
/// let _m = ledger_lab::Money(5);
/// ```
///
/// `#[serde(transparent)]`: trên JSON `Money` chỉ là số nguyên đồng, `25500`.
/// (serde_json vốn đã ghi newtype như giá trị bên trong; thuộc tính này biến
/// điều đó thành hợp đồng viết rõ, không phụ thuộc định dạng hay cách sửa sau này.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Money(i64);

impl Money {
    pub const ZERO: Money = Money(0);

    pub const fn from_dong(dong: i64) -> Money {
        Money(dong)
    }

    pub const fn dong(self) -> i64 {
        self.0
    }

    /// Cộng có kiểm tra tràn số: `None` nếu vượt `i64`.
    pub fn checked_add(self, other: Money) -> Option<Money> {
        self.0.checked_add(other.0).map(Money)
    }
}

// Hiển thị «25.500 VND», số âm «-25.500 VND» (dấu trừ ASCII), giống bài 08.
impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let digits = self.0.unsigned_abs().to_string();
        let mut grouped = String::new();
        for (i, ch) in digits.chars().enumerate() {
            if i > 0 && (digits.len() - i).is_multiple_of(3) {
                grouped.push('.');
            }
            grouped.push(ch);
        }
        let sign = if self.0 < 0 { "-" } else { "" };
        // `pad` để `{:>12}` và `{:<12}` căn lề được như với kiểu chuẩn.
        f.pad(&format!("{sign}{grouped} VND"))
    }
}

/// Dạng thay thế cho khi client không giữ nổi `i64` (ví dụ JavaScript): tiền là chuỗi số.
/// Dùng bằng `#[serde(with = "ledger_lab::money::as_string")]` trên từng trường.
pub mod as_string {
    use super::Money;
    use serde::{Deserialize, Deserializer, Serializer, de};

    pub fn serialize<S: Serializer>(money: &Money, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&money.dong())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Money, D::Error> {
        let raw = String::deserialize(deserializer)?;
        let dong: i64 = raw.parse().map_err(de::Error::custom)?;
        Ok(Money::from_dong(dong))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_groups_thousands() {
        assert_eq!(Money::from_dong(25_500).to_string(), "25.500 VND");
        assert_eq!(Money::from_dong(0).to_string(), "0 VND");
        assert_eq!(Money::from_dong(-1_234_567).to_string(), "-1.234.567 VND");
        assert_eq!(format!("{:>12}", Money::from_dong(-25_500)), " -25.500 VND");
    }

    #[test]
    fn checked_add_detects_overflow() {
        let max = Money::from_dong(i64::MAX);
        assert_eq!(max.checked_add(Money::from_dong(1)), None);
        assert_eq!(
            Money::from_dong(5_100).checked_add(Money::from_dong(20_400)),
            Some(Money::from_dong(25_500))
        );
    }

    #[test]
    fn different_amounts_are_not_equal() {
        assert_ne!(Money::from_dong(1), Money::from_dong(2));
    }
}
