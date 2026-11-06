//! `Box`: một chủ, dữ liệu nằm trên heap. Hai lý do dùng ở bài này:
//! kiểu đệ quy (`FareExpr`) và struct lớn (`RawStatement`).

use std::mem::size_of;

use crate::error::LedgerError;
use crate::money::Money;

/// Công thức cước dạng cây: một nút có thể chứa nút khác cùng kiểu.
/// Không có `Box`, compiler không tính được kích thước của `FareExpr` (E0072).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FareExpr {
    /// Một khoản cố định, ví dụ 12.000 VND cho 2 km đầu.
    Fixed(Money),
    /// `count` đơn vị, mỗi đơn vị `unit`, ví dụ 3 km × 4.500 VND.
    PerUnit { unit: Money, count: i64 },
    /// Tổng hai công thức con.
    Sum(Box<FareExpr>, Box<FareExpr>),
    /// Số nhỏ hơn của hai công thức con (trần cước).
    Min(Box<FareExpr>, Box<FareExpr>),
}

impl FareExpr {
    pub fn sum(left: FareExpr, right: FareExpr) -> FareExpr {
        FareExpr::Sum(Box::new(left), Box::new(right))
    }

    pub fn min(left: FareExpr, right: FareExpr) -> FareExpr {
        FareExpr::Min(Box::new(left), Box::new(right))
    }

    /// Tính cước; mọi phép cộng/nhân đều `checked_*`, tràn số thành `Err(Overflow)`.
    pub fn eval(&self) -> Result<Money, LedgerError> {
        match self {
            FareExpr::Fixed(m) => Ok(*m),
            FareExpr::PerUnit { unit, count } => {
                unit.checked_mul(*count).ok_or(LedgerError::Overflow)
            }
            FareExpr::Sum(a, b) => a
                .eval()?
                .checked_add(b.eval()?)
                .ok_or(LedgerError::Overflow),
            FareExpr::Min(a, b) => Ok(a.eval()?.min(b.eval()?)),
        }
    }

    /// Độ sâu của cây (nút lá có độ sâu 1).
    pub fn depth(&self) -> usize {
        match self {
            FareExpr::Fixed(_) | FareExpr::PerUnit { .. } => 1,
            FareExpr::Sum(a, b) | FareExpr::Min(a, b) => 1 + a.depth().max(b.depth()),
        }
    }
}

/// Cước chuyến TN-0001: 12.000 cho 2 km đầu + 3 km × 4.500 = 25.500 VND.
pub fn tn_0001_fare() -> FareExpr {
    FareExpr::sum(
        FareExpr::Fixed(Money::from_dong(12_000)),
        FareExpr::PerUnit {
            unit: Money::from_dong(4_500),
            count: 3,
        },
    )
}

/// Một bản sao kê thô: 512 dòng × 8 byte nằm thẳng trong struct, tức 4.096 byte.
/// Mỗi lần *move* struct này là một lần copy 4.096 byte; move `Box<RawStatement>` chỉ copy 8 byte.
pub struct RawStatement {
    pub rows: [Money; 512],
}

/// Kích thước (byte) của vài kiểu, đo bằng `size_of` trên máy chạy lab (64-bit).
pub fn sizes() -> Vec<(&'static str, usize)> {
    vec![
        ("Money", size_of::<Money>()),
        ("Box<Money>", size_of::<Box<Money>>()),
        ("RawStatement", size_of::<RawStatement>()),
        ("Box<RawStatement>", size_of::<Box<RawStatement>>()),
        (
            "Box<dyn FeePolicy>",
            size_of::<Box<dyn crate::policy::FeePolicy>>(),
        ),
        ("&str", size_of::<&str>()),
        ("String", size_of::<String>()),
        ("FareExpr", size_of::<FareExpr>()),
        ("Cell<i64>", size_of::<std::cell::Cell<i64>>()),
        ("RefCell<i64>", size_of::<std::cell::RefCell<i64>>()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tn_0001_fare_is_25_500() {
        assert_eq!(tn_0001_fare().eval(), Ok(Money::from_dong(25_500)));
        assert_eq!(tn_0001_fare().depth(), 2);
    }

    #[test]
    fn min_caps_the_fare() {
        let capped = FareExpr::min(tn_0001_fare(), FareExpr::Fixed(Money::from_dong(20_000)));
        assert_eq!(capped.eval(), Ok(Money::from_dong(20_000)));
        assert_eq!(capped.depth(), 3);
    }

    #[test]
    fn overflow_is_an_error_not_a_wrap() {
        let huge = FareExpr::PerUnit {
            unit: Money::from_dong(i64::MAX),
            count: 2,
        };
        assert_eq!(huge.eval(), Err(LedgerError::Overflow));
    }

    #[test]
    fn box_is_one_pointer_wide() {
        assert_eq!(size_of::<Box<RawStatement>>(), size_of::<usize>());
        assert_eq!(size_of::<RawStatement>(), 4_096);
        // Con trỏ béo của `dyn`: địa chỉ dữ liệu + địa chỉ vtable.
        assert_eq!(
            size_of::<Box<dyn crate::policy::FeePolicy>>(),
            2 * size_of::<usize>()
        );
    }
}
