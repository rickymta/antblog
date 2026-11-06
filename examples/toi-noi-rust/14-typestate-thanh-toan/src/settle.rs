//! Dữ kiện của một khoản thanh toán và bút toán sinh ra khi thu tiền hoặc hoàn tiền.
//! Cả hai cách (enum và typestate) dùng chung module này nên sinh cùng một bút toán.

use crate::account::AccountId;
use crate::entry::{JournalEntry, Posting};
use crate::error::LedgerError;
use crate::money::Money;
use crate::state::PaymentError;

/// Dữ kiện không đổi của khoản thanh toán cho một chuyến. Trường private, tạo qua [`PaymentTerms::new`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentTerms {
    trip_id: String,
    customer: AccountId,
    driver: AccountId,
    amount: Money,
    fee: Money,
}

impl PaymentTerms {
    /// `amount` là cước khách trả (phải dương), `fee` là phí nền tảng (0..=amount).
    pub fn new(
        trip_id: &str,
        customer: &str,
        driver: &str,
        amount: Money,
        fee: Money,
    ) -> Result<PaymentTerms, PaymentError> {
        let trip_id = trip_id.trim();
        if trip_id.is_empty() {
            return Err(PaymentError::EmptyTripId);
        }
        if !amount.is_positive() {
            return Err(PaymentError::InvalidAmount { amount });
        }
        if fee < Money::ZERO || fee > amount {
            return Err(PaymentError::InvalidFee { fee });
        }
        Ok(PaymentTerms {
            trip_id: trip_id.to_string(),
            customer: AccountId::new(customer)?,
            driver: AccountId::new(driver)?,
            amount,
            fee,
        })
    }

    pub fn trip_id(&self) -> &str {
        &self.trip_id
    }

    pub fn amount(&self) -> Money {
        self.amount
    }

    pub fn fee(&self) -> Money {
        self.fee
    }
}

fn neg(m: Money) -> Result<Money, LedgerError> {
    m.checked_neg().ok_or(LedgerError::Overflow)
}

/// Phần tài xế nhận = cước trừ phí.
fn payout(terms: &PaymentTerms) -> Result<Money, LedgerError> {
    terms
        .amount
        .checked_sub(terms.fee)
        .ok_or(LedgerError::Overflow)
}

/// Dựng bút toán ba dòng. Dấu: dương là vào tài khoản, âm là ra khỏi tài khoản (quy ước bài 10).
fn entry(
    key: &str,
    terms: &PaymentTerms,
    customer: Money,
    platform: Money,
    driver: Money,
) -> Result<JournalEntry, PaymentError> {
    let postings = vec![
        Posting::new(terms.customer.clone(), customer),
        Posting::new(AccountId::new("platform:revenue")?, platform),
        Posting::new(terms.driver.clone(), driver),
    ];
    Ok(JournalEntry::try_new(key, postings)?)
}

/// Bút toán thu cước: khách -25.500, nền tảng +5.100, tài xế +20.400 (với chuyến `TN-0001`).
pub(crate) fn capture_entry(terms: &PaymentTerms) -> Result<JournalEntry, PaymentError> {
    let key = format!("trip:{}:capture", terms.trip_id);
    entry(&key, terms, neg(terms.amount)?, terms.fee, payout(terms)?)
}

/// Bút toán đảo khi hoàn tiền: mọi posting đổi dấu so với bút toán thu cước.
pub(crate) fn refund_entry(terms: &PaymentTerms) -> Result<JournalEntry, PaymentError> {
    let key = format!("trip:{}:refund", terms.trip_id);
    entry(
        &key,
        terms,
        terms.amount,
        neg(terms.fee)?,
        neg(payout(terms)?)?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terms() -> PaymentTerms {
        PaymentTerms::new(
            "TN-0001",
            "customer:CU-001",
            "driver:DR-001",
            Money::from_dong(25_500),
            Money::from_dong(5_100),
        )
        .unwrap()
    }

    #[test]
    fn capture_entry_matches_trip_tn_0001() {
        let e = capture_entry(&terms()).unwrap();
        assert_eq!(e.idempotency_key(), "trip:TN-0001:capture");
        let amounts: Vec<i64> = e.postings().iter().map(|p| p.amount.dong()).collect();
        assert_eq!(amounts, vec![-25_500, 5_100, 20_400]);
    }

    #[test]
    fn refund_entry_is_exact_reversal() {
        let c = capture_entry(&terms()).unwrap();
        let r = refund_entry(&terms()).unwrap();
        assert_eq!(r.idempotency_key(), "trip:TN-0001:refund");
        for (a, b) in c.postings().iter().zip(r.postings()) {
            assert_eq!(a.account, b.account);
            assert_eq!(a.amount.checked_add(b.amount), Some(Money::ZERO));
        }
    }

    #[test]
    fn terms_reject_bad_input() {
        let new = |trip: &str, amount: i64, fee: i64| {
            PaymentTerms::new(
                trip,
                "customer:CU-001",
                "driver:DR-001",
                Money::from_dong(amount),
                Money::from_dong(fee),
            )
        };
        assert_eq!(new("  ", 100, 20), Err(PaymentError::EmptyTripId));
        assert!(matches!(
            new("TN-1", 0, 0),
            Err(PaymentError::InvalidAmount { .. })
        ));
        assert!(matches!(
            new("TN-1", 100, 101),
            Err(PaymentError::InvalidFee { .. })
        ));
        assert!(matches!(
            new("TN-1", 100, -1),
            Err(PaymentError::InvalidFee { .. })
        ));
        assert!(new("TN-1", 100, 100).is_ok());
    }
}
