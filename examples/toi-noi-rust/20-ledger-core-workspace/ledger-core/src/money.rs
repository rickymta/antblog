//! Tiền VND: số nguyên `i64` đơn vị đồng, không bao giờ dùng float.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Số tiền có dấu, đơn vị đồng. Trường bên trong là private: bên ngoài crate
/// chỉ tạo qua [`Money::from_dong`].
///
/// Dương là tiền vào tài khoản, âm là tiền ra khỏi tài khoản. Khi serialize,
/// `Money` là một số nguyên JSON (không phải chuỗi, không phải số thực).
///
/// ```
/// use ledger_core::Money;
///
/// let fare = Money::from_dong(25_500);
/// assert_eq!(fare.to_string(), "25.500 VND");
/// assert_eq!(serde_json::to_string(&fare).unwrap(), "25500");
/// ```
///
/// Không thể viết `Money(5)` từ crate khác, vì trường là private:
///
/// ```compile_fail
/// let _m = ledger_core::Money(5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Money(i64);

impl Money {
    /// Không đồng nào.
    pub const ZERO: Money = Money(0);

    /// Tạo từ số đồng (có thể âm).
    pub const fn from_dong(dong: i64) -> Money {
        Money(dong)
    }

    /// Số đồng bên trong.
    pub const fn dong(self) -> i64 {
        self.0
    }

    /// Cộng có kiểm tra tràn số: `None` nếu vượt `i64`.
    pub fn checked_add(self, other: Money) -> Option<Money> {
        self.0.checked_add(other.0).map(Money)
    }

    /// Trừ có kiểm tra tràn số: `None` nếu vượt `i64`.
    pub fn checked_sub(self, other: Money) -> Option<Money> {
        self.0.checked_sub(other.0).map(Money)
    }

    /// Đổi dấu (dùng cho bút toán đảo): `None` với `i64::MIN`, vì không có đối số dương.
    pub fn checked_neg(self) -> Option<Money> {
        self.0.checked_neg().map(Money)
    }
}

// Hiển thị «25.500 VND», số âm «-25.500 VND» (dấu trừ ASCII), cùng cách với bài 08.
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
        // `pad` để `{:>12}` căn lề được như với kiểu chuẩn.
        f.pad(&format!("{sign}{grouped} VND"))
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
    fn checked_ops_report_overflow_as_none() {
        let max = Money::from_dong(i64::MAX);
        let min = Money::from_dong(i64::MIN);
        assert_eq!(max.checked_add(Money::from_dong(1)), None);
        assert_eq!(min.checked_sub(Money::from_dong(1)), None);
        assert_eq!(min.checked_neg(), None);
        assert_eq!(
            Money::from_dong(5_100).checked_add(Money::from_dong(20_400)),
            Some(Money::from_dong(25_500))
        );
    }

    #[test]
    fn serde_is_a_plain_integer() {
        let json = serde_json::to_string(&Money::from_dong(-25_500)).unwrap();
        assert_eq!(json, "-25500");
        let back: Money = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Money::from_dong(-25_500));
    }
}
