//! Chính sách làm tròn và chính sách phí: hai trait, nhiều cách triển khai thay thế được.

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};

use crate::error::LedgerError;
use crate::money::Money;

/// Cách đưa một số thập phân về đồng nguyên.
pub trait RoundingPolicy {
    /// Tên hiển thị của chính sách.
    fn name(&self) -> &'static str;

    /// Phương thức bắt buộc: làm tròn về đồng; `Overflow` nếu không vừa `i64`.
    fn round_to_dong(&self, amount: Decimal) -> Result<Money, LedgerError>;

    /// Default method: nhân cước với tỷ lệ rồi làm tròn. Kiểu nào implement trait
    /// cũng có sẵn, chỉ cần viết `round_to_dong`.
    fn round_ratio(&self, base: Money, rate: Decimal) -> Result<Money, LedgerError> {
        let raw = Decimal::from(base.dong())
            .checked_mul(rate)
            .ok_or(LedgerError::Overflow)?;
        self.round_to_dong(raw)
    }
}

fn round_with(strategy: RoundingStrategy, amount: Decimal) -> Result<Money, LedgerError> {
    amount
        .round_dp_with_strategy(0, strategy)
        .to_i64()
        .map(Money::from_dong)
        .ok_or(LedgerError::Overflow)
}

/// Nửa đồng làm tròn ra xa 0 (1402,5 thành 1403): quy ước phí của Tới Nơi.
#[derive(Debug, Clone, Copy, Default)]
pub struct HalfAwayFromZero;

impl RoundingPolicy for HalfAwayFromZero {
    fn name(&self) -> &'static str {
        "nửa ra xa 0"
    }

    fn round_to_dong(&self, amount: Decimal) -> Result<Money, LedgerError> {
        round_with(RoundingStrategy::MidpointAwayFromZero, amount)
    }
}

/// Nửa đồng làm tròn về số chẵn gần nhất (1402,5 thành 1402), còn gọi banker's rounding.
#[derive(Debug, Clone, Copy, Default)]
pub struct NearestEven;

impl RoundingPolicy for NearestEven {
    fn name(&self) -> &'static str {
        "nửa về số chẵn"
    }

    fn round_to_dong(&self, amount: Decimal) -> Result<Money, LedgerError> {
        round_with(RoundingStrategy::MidpointNearestEven, amount)
    }
}

/// Kết quả chia một khoản cước giữa nền tảng và tài xế.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeeSplit {
    pub platform_fee: Money,
    pub driver_net: Money,
}

/// Cách tính phí nền tảng từ cước.
pub trait FeePolicy {
    fn name(&self) -> String;

    /// Phương thức bắt buộc: phí nền tảng của một khoản cước.
    fn fee(&self, fare: Money) -> Result<Money, LedgerError>;

    /// Default method: chia cước và kiểm hợp đồng (0 <= phí <= cước).
    /// Mọi chính sách đều dùng chung bước kiểm này, không phải tự viết lại.
    fn split(&self, fare: Money) -> Result<FeeSplit, LedgerError> {
        let fee = self.fee(fare)?;
        if fee < Money::ZERO || fee > fare {
            return Err(LedgerError::InvalidFee { fee, fare });
        }
        let driver_net = fare.checked_sub(fee).ok_or(LedgerError::Overflow)?;
        Ok(FeeSplit {
            platform_fee: fee,
            driver_net,
        })
    }
}

/// Phí theo tỷ lệ, ghép với một `RoundingPolicy` bất kỳ qua tham số kiểu `R`.
#[derive(Debug, Clone)]
pub struct PercentageFee<R: RoundingPolicy> {
    rate: Decimal,
    rounding: R,
}

impl<R: RoundingPolicy> PercentageFee<R> {
    pub fn new(rate: Decimal, rounding: R) -> Self {
        PercentageFee { rate, rounding }
    }
}

// Một impl dùng chung cho mọi R: trình biên dịch sinh bản riêng cho từng R được dùng.
impl<R: RoundingPolicy> FeePolicy for PercentageFee<R> {
    fn name(&self) -> String {
        let percent = (self.rate * Decimal::from(100)).normalize();
        format!("{percent}% · {}", self.rounding.name())
    }

    fn fee(&self, fare: Money) -> Result<Money, LedgerError> {
        self.rounding.round_ratio(fare, self.rate)
    }
}

/// Phí cố định mỗi chuyến, không vượt quá cước.
#[derive(Debug, Clone, Copy)]
pub struct FlatFee {
    amount: Money,
}

impl FlatFee {
    pub fn new(amount: Money) -> FlatFee {
        FlatFee { amount }
    }
}

impl FeePolicy for FlatFee {
    fn name(&self) -> String {
        format!("cố định {}", self.amount)
    }

    fn fee(&self, fare: Money) -> Result<Money, LedgerError> {
        Ok(self.amount.min(fare))
    }
}

/// Chính sách mặc định của Tới Nơi: 20%, nửa đồng làm tròn ra xa 0.
/// `impl FeePolicy` ở vị trí kiểu trả về: người gọi chỉ biết "có FeePolicy".
pub fn default_policy() -> impl FeePolicy {
    PercentageFee::new(Decimal::new(20, 2), HalfAwayFromZero)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dong(n: i64) -> Money {
        Money::from_dong(n)
    }

    #[test]
    fn default_policy_takes_20_percent() -> Result<(), LedgerError> {
        let split = default_policy().split(dong(25_500))?;
        assert_eq!(split.platform_fee, dong(5_100));
        assert_eq!(split.driver_net, dong(20_400));
        Ok(())
    }

    #[test]
    fn midpoint_differs_between_rounding_policies() -> Result<(), LedgerError> {
        let rate = Decimal::new(55, 3); // 5,5%: 25.500 x 0,055 = 1402,5
        assert_eq!(
            HalfAwayFromZero.round_ratio(dong(25_500), rate)?,
            dong(1_403)
        );
        assert_eq!(NearestEven.round_ratio(dong(25_500), rate)?, dong(1_402));
        Ok(())
    }

    #[test]
    fn rounding_overflow_is_an_error() {
        let huge = Decimal::from(i64::MAX) * Decimal::from(2);
        assert_eq!(
            HalfAwayFromZero.round_to_dong(huge),
            Err(LedgerError::Overflow)
        );
    }

    #[test]
    fn flat_fee_is_capped_at_fare() -> Result<(), LedgerError> {
        let policy = FlatFee::new(dong(3_000));
        assert_eq!(policy.split(dong(25_500))?.driver_net, dong(22_500));
        let small = policy.split(dong(2_000))?;
        assert_eq!(small.platform_fee, dong(2_000));
        assert_eq!(small.driver_net, Money::ZERO);
        Ok(())
    }

    #[test]
    fn negative_rate_violates_the_contract() {
        let policy = PercentageFee::new(Decimal::new(-1, 1), HalfAwayFromZero);
        assert_eq!(
            policy.split(dong(25_500)),
            Err(LedgerError::InvalidFee {
                fee: dong(-2_550),
                fare: dong(25_500)
            })
        );
    }

    #[test]
    fn names_describe_the_policy() {
        assert_eq!(default_policy().name(), "20% · nửa ra xa 0");
        assert_eq!(FlatFee::new(dong(3_000)).name(), "cố định 3.000 VND");
    }
}
