//! Tiền VND: số nguyên `i64` đơn vị đồng, không bao giờ dùng float.

use std::fmt;

/// Số tiền có dấu, đơn vị đồng. Trường bên trong là private:
/// bên ngoài module chỉ tạo qua [`Money::from_dong`].
///
/// Không thể viết `Money(5)` từ crate khác, vì trường là private:
///
/// ```compile_fail
/// let _m = ledger_lab::Money(5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::strategies;

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

    proptest! {
        // `any::<i64>()` phủ cả khoảng i64 nhưng chia đều, nên hiếm khi ra số nhỏ hay đầu mút.
        #[test]
        fn dong_round_trips(x in any::<i64>()) {
            prop_assert_eq!(Money::from_dong(x).dong(), x);
        }

        #[test]
        fn checked_add_is_commutative(a in any::<i64>(), b in any::<i64>()) {
            let (a, b) = (Money::from_dong(a), Money::from_dong(b));
            prop_assert_eq!(a.checked_add(b), b.checked_add(a));
        }

        // Bỏ dấu chấm, dấu trừ và hậu tố ra phải còn đúng chữ số của |x|; strategy có cả i64::MIN.
        #[test]
        fn display_keeps_the_digits(x in strategies::total_dong()) {
            let text = Money::from_dong(x).to_string();
            prop_assert!(text.ends_with(" VND"));
            let digits: String = text.chars().filter(char::is_ascii_digit).collect();
            prop_assert_eq!(digits, x.unsigned_abs().to_string());
            prop_assert_eq!(text.starts_with('-'), x < 0);
        }
    }
}
