//! Quãng đường và cước chuyến: một newtype nữa để thấy `Km` không lẫn với `Money`.

use crate::money::Money;

/// Số km nguyên của chuyến.
///
/// Cộng Money với Km là lỗi biên dịch (E0308):
///
/// ```compile_fail,E0308
/// use ledger_lab::{Km, Money};
///
/// let _ = Money::from_dong(12_000) + Km::new(5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Km(u32);

impl Km {
    pub const fn new(km: u32) -> Km {
        Km(km)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Cước: 12.000 đồng cho 2 km đầu, 4.500 đồng mỗi km vượt. Nhận `Km`, trả `Money`.
pub fn trip_fare(distance: Km) -> Money {
    let extra_km = i64::from(distance.get().saturating_sub(2));
    // `u32` tối đa ≈ 4,3 tỷ km, nhân 4.500 vẫn nhỏ hơn nhiều so với `i64::MAX`.
    Money::from_dong(12_000 + 4_500 * extra_km)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fare_matches_the_sample_trip() {
        assert_eq!(trip_fare(Km::new(5)), Money::from_dong(25_500));
        assert_eq!(trip_fare(Km::new(2)), Money::from_dong(12_000));
        assert_eq!(trip_fare(Km::new(1)), Money::from_dong(12_000));
    }

    #[test]
    fn huge_distance_does_not_overflow() {
        assert_eq!(
            trip_fare(Km::new(u32::MAX)),
            Money::from_dong(12_000 + 4_500 * (i64::from(u32::MAX) - 2))
        );
    }
}
