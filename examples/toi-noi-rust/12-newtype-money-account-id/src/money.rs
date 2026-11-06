//! Tiền: newtype quanh `i64`, mang loại tiền ở kiểu (phantom type).

use std::fmt;
use std::marker::PhantomData;
use std::ops::{Add, Sub};

use crate::error::MoneyError;

/// Loại tiền. Chỉ là nhãn ở thời điểm biên dịch, không tốn byte nào lúc chạy.
pub trait Currency {
    /// Mã ISO 4217, dùng khi hiển thị.
    const CODE: &'static str;
}

/// Đồng Việt Nam. `enum` rỗng: không bao giờ tạo được giá trị, chỉ dùng làm tham số kiểu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Vnd {}

impl Currency for Vnd {
    const CODE: &'static str = "VND";
}

/// Đô la Mỹ, chỉ để chứng minh trình biên dịch chặn cộng khác loại tiền.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Usd {}

impl Currency for Usd {
    const CODE: &'static str = "USD";
}

/// Số tiền có dấu, tính theo đơn vị nhỏ nhất của loại tiền `C`
/// (VND: đồng, vì VND không có phần lẻ). Mặc định là VND: `Money` nghĩa là `Money<Vnd>`.
///
/// `derive` bên dưới chỉ sinh impl khi `C` cũng có trait đó (hai loại tiền ở trên đã derive đủ).
/// Trường là private: ngoài module chỉ tạo qua `new`, `from_dong` hoặc `TryFrom`.
///
/// Cộng hai loại tiền khác nhau là lỗi biên dịch (E0308):
///
/// ```compile_fail,E0308
/// use ledger_lab::{Money, Usd, Vnd};
///
/// let vnd = Money::<Vnd>::from_dong(25_500);
/// let usd = Money::<Usd>::new(100);
/// let _ = vnd + usd;
/// ```
///
/// Cộng Money với số trần cũng vậy:
///
/// ```compile_fail,E0308
/// use ledger_lab::Money;
///
/// let _ = Money::from_dong(25_500) + 5_100_i64;
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Money<C: Currency = Vnd> {
    amount: i64,
    currency: PhantomData<C>,
}

impl<C: Currency> Money<C> {
    pub const ZERO: Money<C> = Money::new(0);

    pub const fn new(amount: i64) -> Money<C> {
        Money {
            amount,
            currency: PhantomData,
        }
    }

    pub const fn amount(self) -> i64 {
        self.amount
    }

    /// Cộng có kiểm tra tràn số: `None` nếu vượt `i64`.
    pub fn checked_add(self, other: Money<C>) -> Option<Money<C>> {
        self.amount.checked_add(other.amount).map(Money::new)
    }

    /// Trừ có kiểm tra tràn số: `None` nếu vượt `i64`.
    pub fn checked_sub(self, other: Money<C>) -> Option<Money<C>> {
        self.amount.checked_sub(other.amount).map(Money::new)
    }
}

// Tên quen thuộc từ các bài trước, chỉ có cho VND.
impl Money<Vnd> {
    pub const fn from_dong(dong: i64) -> Money<Vnd> {
        Money::new(dong)
    }

    pub const fn dong(self) -> i64 {
        self.amount
    }
}

// Chọn: toán tử trả `Result`, không panic, không quay vòng. Muốn cộng tiếp thì `?`.
impl<C: Currency> Add for Money<C> {
    type Output = Result<Money<C>, MoneyError>;

    fn add(self, rhs: Money<C>) -> Self::Output {
        self.checked_add(rhs).ok_or(MoneyError::Overflow)
    }
}

impl<C: Currency> Sub for Money<C> {
    type Output = Result<Money<C>, MoneyError>;

    fn sub(self, rhs: Money<C>) -> Self::Output {
        self.checked_sub(rhs).ok_or(MoneyError::Overflow)
    }
}

// Chuyển từ kiểu rộng hơn có thể thất bại, nên là `TryFrom`, không phải `From`.
// Cố ý không có `From<i64>`: `5_i64.into()` sẽ che mất đơn vị.
impl<C: Currency> TryFrom<i128> for Money<C> {
    type Error = MoneyError;

    fn try_from(wide: i128) -> Result<Money<C>, MoneyError> {
        i64::try_from(wide)
            .map(Money::new)
            .map_err(|_| MoneyError::OutOfRange)
    }
}

// `Debug` viết tay: derive sẽ in cả `PhantomData<…>` rất dài.
impl<C: Currency> fmt::Debug for Money<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Money({} {})", self.amount, C::CODE)
    }
}

// Hiển thị «25.500 VND», số âm «-25.500 VND» (dấu trừ ASCII), giống bài 08.
impl<C: Currency> fmt::Display for Money<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let digits = self.amount.unsigned_abs().to_string();
        let mut grouped = String::new();
        for (i, ch) in digits.chars().enumerate() {
            if i > 0 && (digits.len() - i).is_multiple_of(3) {
                grouped.push('.');
            }
            grouped.push(ch);
        }
        let sign = if self.amount < 0 { "-" } else { "" };
        // `pad` để `{:>12}` và `{:<12}` căn lề được như với kiểu chuẩn.
        f.pad(&format!("{sign}{grouped} {}", C::CODE))
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
        assert_eq!(Money::<Usd>::new(100).to_string(), "100 USD");
    }

    #[test]
    fn debug_shows_currency() {
        assert_eq!(format!("{:?}", Money::from_dong(5_100)), "Money(5100 VND)");
    }

    #[test]
    fn add_and_sub_return_result() {
        let fee = Money::from_dong(5_100);
        let driver = Money::from_dong(20_400);
        assert_eq!(fee + driver, Ok(Money::from_dong(25_500)));
        assert_eq!(driver - fee, Ok(Money::from_dong(15_300)));
        assert_eq!(fee - driver, Ok(Money::from_dong(-15_300)));
    }

    #[test]
    fn overflow_is_an_error_not_a_panic() {
        let max = Money::from_dong(i64::MAX);
        let min = Money::from_dong(i64::MIN);
        assert_eq!(max + Money::from_dong(1), Err(MoneyError::Overflow));
        assert_eq!(min - Money::from_dong(1), Err(MoneyError::Overflow));
        assert_eq!(max.checked_add(Money::from_dong(1)), None);
    }

    #[test]
    fn try_from_i128_checks_range() {
        assert_eq!(
            Money::<Vnd>::try_from(25_500_i128),
            Ok(Money::from_dong(25_500))
        );
        let too_big = i128::from(i64::MAX) + 1;
        assert_eq!(Money::<Vnd>::try_from(too_big), Err(MoneyError::OutOfRange));
        let too_small = i128::from(i64::MIN) - 1;
        assert_eq!(
            Money::<Vnd>::try_from(too_small),
            Err(MoneyError::OutOfRange)
        );
    }

    #[test]
    fn ordering_and_copy() {
        let a = Money::from_dong(1);
        let b = a; // Copy: `a` vẫn dùng được sau khi gán.
        assert!(a < Money::from_dong(2));
        assert_eq!(a, b);
        assert_eq!(Money::<Vnd>::ZERO, Money::from_dong(0));
    }

    #[test]
    fn phantom_type_costs_nothing() {
        assert_eq!(size_of::<Money<Vnd>>(), size_of::<i64>());
        assert_eq!(size_of::<Money<Usd>>(), size_of::<i64>());
    }
}
