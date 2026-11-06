//! `Box<dyn FeePolicy>`: một danh sách chính sách phí khác kiểu trong cùng `Vec`.
//! Trait và generics được dạy ở bài 11; ở đây chỉ cần biết `dyn` chọn hàm lúc chạy.

use crate::account::AccountId;
use crate::entry::{JournalEntry, Posting};
use crate::error::LedgerError;
use crate::money::Money;

/// Chính sách tính phí nền tảng trên cước đã chốt.
pub trait FeePolicy {
    fn name(&self) -> &str;
    fn fee(&self, fare: Money) -> Result<Money, LedgerError>;
}

fn require_non_negative(fare: Money) -> Result<(), LedgerError> {
    if fare < Money::ZERO {
        return Err(LedgerError::NegativeFare { fare });
    }
    Ok(())
}

/// Phí theo phần trăm nguyên của cước, làm tròn nửa lên về đồng.
/// Rút gọn: bài 03 dùng `rust_decimal` cho tỷ lệ lẻ; ở đây `percent` là số nguyên.
pub struct PercentFee {
    pub percent: i64,
}

impl FeePolicy for PercentFee {
    fn name(&self) -> &str {
        "phần trăm"
    }

    fn fee(&self, fare: Money) -> Result<Money, LedgerError> {
        require_non_negative(fare)?;
        let scaled = fare
            .checked_mul(self.percent)
            .ok_or(LedgerError::Overflow)?;
        let rounded = scaled.dong().checked_add(50).ok_or(LedgerError::Overflow)?;
        Ok(Money::from_dong(rounded / 100))
    }
}

/// Phí cố định (khuyến mãi), không vượt quá cước.
pub struct FlatFee {
    pub amount: Money,
}

impl FeePolicy for FlatFee {
    fn name(&self) -> &str {
        "cố định"
    }

    fn fee(&self, fare: Money) -> Result<Money, LedgerError> {
        require_non_negative(fare)?;
        Ok(self.amount.min(fare))
    }
}

/// Phí phần trăm có trần.
pub struct CappedPercentFee {
    pub percent: i64,
    pub cap: Money,
}

impl FeePolicy for CappedPercentFee {
    fn name(&self) -> &str {
        "phần trăm có trần"
    }

    fn fee(&self, fare: Money) -> Result<Money, LedgerError> {
        let uncapped = PercentFee {
            percent: self.percent,
        }
        .fee(fare)?;
        Ok(uncapped.min(self.cap))
    }
}

/// Ba kiểu khác nhau, một `Vec`: mỗi phần tử là một `Box` trỏ tới kiểu cụ thể.
pub fn default_policies() -> Vec<Box<dyn FeePolicy>> {
    vec![
        Box::new(PercentFee { percent: 20 }),
        Box::new(FlatFee {
            amount: Money::from_dong(3_000),
        }),
        Box::new(CappedPercentFee {
            percent: 20,
            cap: Money::from_dong(4_000),
        }),
    ]
}

/// Tên chính sách và phí tính được trên cùng một mức cước.
pub fn quote_all(
    policies: &[Box<dyn FeePolicy>],
    fare: Money,
) -> Result<Vec<(String, Money)>, LedgerError> {
    policies
        .iter()
        .map(|p| Ok((p.name().to_string(), p.fee(fare)?)))
        .collect()
}

/// Bút toán thu cước: khách -cước, nền tảng +phí, tài xế +(cước - phí).
/// Nhận `&dyn FeePolicy` nên gọi được với `&PercentFee { .. }` lẫn `policy.as_ref()` của một `Box`.
pub fn capture_entry(
    key: &str,
    customer: &str,
    driver: &str,
    fare: Money,
    policy: &dyn FeePolicy,
) -> Result<JournalEntry, LedgerError> {
    let fee = policy.fee(fare)?;
    let driver_share = fare.checked_sub(fee).ok_or(LedgerError::Overflow)?;
    let negated = Money::ZERO.checked_sub(fare).ok_or(LedgerError::Overflow)?;
    JournalEntry::try_new(
        key,
        vec![
            Posting::new(AccountId::new(customer)?, negated),
            Posting::new(AccountId::new("platform:revenue")?, fee),
            Posting::new(AccountId::new(driver)?, driver_share),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_policy_gives_5_100_on_25_500() {
        let fee = PercentFee { percent: 20 }.fee(Money::from_dong(25_500));
        assert_eq!(fee, Ok(Money::from_dong(5_100)));
    }

    #[test]
    fn three_policies_in_one_vec() -> Result<(), LedgerError> {
        let quotes = quote_all(&default_policies(), Money::from_dong(25_500))?;
        let fees: Vec<i64> = quotes.iter().map(|(_, m)| m.dong()).collect();
        assert_eq!(fees, vec![5_100, 3_000, 4_000]);
        Ok(())
    }

    #[test]
    fn negative_fare_is_rejected() {
        let err = PercentFee { percent: 20 }.fee(Money::from_dong(-1));
        assert_eq!(
            err,
            Err(LedgerError::NegativeFare {
                fare: Money::from_dong(-1)
            })
        );
    }

    #[test]
    fn capture_entry_splits_25_500() -> Result<(), LedgerError> {
        let policies = default_policies();
        let entry = capture_entry(
            "trip:TN-0001:capture",
            "customer:CU-001",
            "driver:DR-001",
            Money::from_dong(25_500),
            policies[0].as_ref(),
        )?;
        let amounts: Vec<i64> = entry.postings().iter().map(|p| p.amount.dong()).collect();
        assert_eq!(amounts, vec![-25_500, 5_100, 20_400]);
        Ok(())
    }
}
