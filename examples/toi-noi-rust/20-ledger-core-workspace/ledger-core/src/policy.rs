//! Chính sách làm tròn và chính sách phí: hai trait, nhiều cách triển khai thay thế được.

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};

use crate::error::LedgerError;
use crate::money::Money;

/// Cách đưa một số thập phân về đồng nguyên.
pub trait RoundingPolicy {
    /// Tên hiển thị của chính sách.
    fn name(&self) -> &'static str;

    /// Phương thức bắt buộc: làm tròn về đồng.
    ///
    /// # Errors
    /// [`LedgerError::Overflow`] nếu kết quả không vừa `i64`.
    fn round_to_dong(&self, amount: Decimal) -> Result<Money, LedgerError>;

    /// Default method: nhân cước với tỷ lệ rồi làm tròn. Chỉ cần viết `round_to_dong`.
    ///
    /// # Errors
    /// [`LedgerError::Overflow`] nếu phép nhân hoặc kết quả không vừa.
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
/// Luôn có `platform_fee + driver_net == fare`: không mất, không thừa đồng nào.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeeSplit {
    /// Phí nền tảng giữ lại.
    pub platform_fee: Money,
    /// Phần tài xế nhận.
    pub driver_net: Money,
}

/// Cách tính phí nền tảng từ cước. Chỉ phải viết `name` và [`FeePolicy::fee`];
/// [`FeePolicy::split`] có sẵn và kiểm hợp đồng `0 <= phí <= cước`.
///
/// ```
/// use ledger_core::{FeePolicy, Money, default_policy};
///
/// let split = default_policy().split(Money::from_dong(25_500))?;
/// assert_eq!(split.platform_fee, Money::from_dong(5_100));
/// assert_eq!(split.driver_net, Money::from_dong(20_400));
/// # Ok::<(), ledger_core::LedgerError>(())
/// ```
pub trait FeePolicy {
    /// Tên hiển thị của chính sách.
    fn name(&self) -> String;

    /// Phương thức bắt buộc: phí nền tảng của một khoản cước.
    ///
    /// # Errors
    /// Tuỳ chính sách, ví dụ [`LedgerError::Overflow`].
    fn fee(&self, fare: Money) -> Result<Money, LedgerError>;

    /// Default method: chia cước và kiểm hợp đồng. Mọi chính sách dùng chung bước kiểm này.
    ///
    /// # Errors
    /// Mọi lỗi của [`FeePolicy::fee`]; [`LedgerError::InvalidFee`] nếu phí âm hoặc lớn hơn cước.
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

/// Phí theo tỷ lệ, ghép với một [`RoundingPolicy`] bất kỳ qua tham số kiểu `R`.
#[derive(Debug, Clone)]
pub struct PercentageFee<R: RoundingPolicy> {
    rate: Decimal,
    rounding: R,
}

impl<R: RoundingPolicy> PercentageFee<R> {
    /// Phí `rate` (ví dụ `Decimal::new(20, 2)` là 20%) với cách làm tròn `rounding`.
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
    /// Phí cố định `amount`.
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
/// `impl FeePolicy` ở vị trí kiểu trả về: người gọi chỉ biết "có `FeePolicy`".
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
    fn default_policy_takes_twenty_percent() -> Result<(), LedgerError> {
        let split = default_policy().split(dong(25_500))?;
        assert_eq!(split.platform_fee, dong(5_100));
        assert_eq!(split.driver_net, dong(20_400));
        assert_eq!(default_policy().name(), "20% · nửa ra xa 0");
        Ok(())
    }

    // 15% của 1.010 là 151,5: đúng một nửa đồng, hai cách làm tròn cho hai kết quả.
    #[test]
    fn rounding_policy_changes_the_half_dong() -> Result<(), LedgerError> {
        let rate = Decimal::new(15, 2);
        let away = PercentageFee::new(rate, HalfAwayFromZero);
        let even = PercentageFee::new(rate, NearestEven);
        assert_eq!(away.fee(dong(1_010))?, dong(152));
        assert_eq!(even.fee(dong(1_010))?, dong(152));
        // 15% của 1.030 là 154,5: số chẵn gần nhất là 154.
        assert_eq!(away.fee(dong(1_030))?, dong(155));
        assert_eq!(even.fee(dong(1_030))?, dong(154));
        Ok(())
    }

    #[test]
    fn policies_are_interchangeable_behind_dyn() -> Result<(), LedgerError> {
        let policies: Vec<Box<dyn FeePolicy>> = vec![
            Box::new(default_policy()),
            Box::new(FlatFee::new(dong(3_000))),
        ];
        let fees: Result<Vec<Money>, LedgerError> =
            policies.iter().map(|p| p.fee(dong(25_500))).collect();
        assert_eq!(fees?, vec![dong(5_100), dong(3_000)]);
        Ok(())
    }

    #[test]
    fn flat_fee_never_exceeds_the_fare() -> Result<(), LedgerError> {
        assert_eq!(
            FlatFee::new(dong(3_000)).split(dong(2_000))?.driver_net,
            Money::ZERO
        );
        Ok(())
    }

    // Chính sách sai hợp đồng (phí lớn hơn cước) bị `split` chặn.
    #[test]
    fn split_rejects_a_fee_above_the_fare() {
        let policy = PercentageFee::new(Decimal::new(150, 2), HalfAwayFromZero);
        assert_eq!(
            policy.split(dong(1_000)),
            Err(LedgerError::InvalidFee {
                fee: dong(1_500),
                fare: dong(1_000)
            })
        );
    }
}
