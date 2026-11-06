//! Posting và bút toán (journal entry): bút toán chỉ tồn tại khi cân.

use crate::account::AccountId;
use crate::error::LedgerError;
use crate::money::Money;

/// Một dòng của bút toán: số tiền có dấu (dương cộng vào tài khoản, âm trừ đi).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posting {
    pub account: AccountId,
    pub amount: Money,
}

impl Posting {
    pub fn new(account: AccountId, amount: Money) -> Posting {
        Posting { account, amount }
    }
}

/// Bút toán đã hợp lệ. Trường private nên chỉ [`JournalEntry::try_new`] tạo được.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalEntry {
    idempotency_key: String,
    postings: Vec<Posting>,
}

/// Tổng có kiểm tra tràn số. `pub(crate)`: các module khác trong crate dùng được
/// (ledger.rs kiểm tổng toàn sổ), người dùng thư viện thì không.
pub(crate) fn total(postings: &[Posting]) -> Result<Money, LedgerError> {
    let mut sum = Money::ZERO;
    for p in postings {
        sum = sum.checked_add(p.amount).ok_or(LedgerError::Overflow)?;
    }
    Ok(sum)
}

impl JournalEntry {
    /// Tạo bút toán: ít nhất 2 posting, tổng có dấu bằng 0, không tràn số.
    ///
    /// ```
    /// use ledger_lab::{AccountId, JournalEntry, Money, Posting};
    ///
    /// let a = AccountId::new("customer:CU-001").unwrap();
    /// let b = AccountId::new("platform:revenue").unwrap();
    /// let entry = JournalEntry::try_new(
    ///     "trip:TN-0001:capture",
    ///     vec![
    ///         Posting::new(a, Money::from_dong(-5_100)),
    ///         Posting::new(b, Money::from_dong(5_100)),
    ///     ],
    /// );
    /// assert!(entry.is_ok());
    /// ```
    pub fn try_new(
        idempotency_key: &str,
        postings: Vec<Posting>,
    ) -> Result<JournalEntry, LedgerError> {
        if idempotency_key.trim().is_empty() {
            return Err(LedgerError::EmptyIdempotencyKey);
        }
        if postings.len() < 2 {
            return Err(LedgerError::TooFewPostings {
                count: postings.len(),
            });
        }
        let sum = total(&postings)?;
        if sum != Money::ZERO {
            return Err(LedgerError::Unbalanced { sum });
        }
        Ok(JournalEntry {
            idempotency_key: idempotency_key.to_string(),
            postings,
        })
    }

    pub fn idempotency_key(&self) -> &str {
        &self.idempotency_key
    }

    pub fn postings(&self) -> &[Posting] {
        &self.postings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn acct(raw: &str) -> AccountId {
        AccountId::new(raw).unwrap()
    }

    fn posting(raw: &str, dong: i64) -> Posting {
        Posting::new(acct(raw), Money::from_dong(dong))
    }

    #[test]
    fn balanced_entry_is_accepted() -> Result<(), LedgerError> {
        let entry = JournalEntry::try_new(
            "trip:TN-0001:capture",
            vec![
                posting("customer:CU-001", -25_500),
                posting("platform:revenue", 5_100),
                posting("driver:DR-001", 20_400),
            ],
        )?;
        assert_eq!(entry.postings().len(), 3);
        assert_eq!(entry.idempotency_key(), "trip:TN-0001:capture");
        Ok(())
    }

    #[test]
    fn unbalanced_entry_is_rejected() {
        let result = JournalEntry::try_new(
            "k",
            vec![
                posting("customer:CU-001", -25_500),
                posting("driver:DR-001", 20_400),
            ],
        );
        assert_eq!(
            result,
            Err(LedgerError::Unbalanced {
                sum: Money::from_dong(-5_100)
            })
        );
    }

    #[test]
    fn overflow_is_reported_not_wrapped() {
        let result = JournalEntry::try_new("k", vec![posting("a", i64::MAX), posting("b", 1)]);
        assert_eq!(result, Err(LedgerError::Overflow));
    }

    #[test]
    fn single_posting_is_too_few() {
        let result = JournalEntry::try_new("k", vec![posting("a", 0)]);
        assert_eq!(result, Err(LedgerError::TooFewPostings { count: 1 }));
    }

    #[test]
    fn empty_key_is_rejected() {
        let result = JournalEntry::try_new("  ", vec![posting("a", 1), posting("b", -1)]);
        assert_eq!(result, Err(LedgerError::EmptyIdempotencyKey));
    }

    // Đây là panic thật vì `unwrap` trên `Err`; thông điệp chứa Debug của lỗi.
    #[test]
    #[should_panic(expected = "Unbalanced")]
    fn unwrap_on_unbalanced_entry_panics() {
        JournalEntry::try_new("k", vec![posting("a", 1), posting("b", 2)]).unwrap();
    }
}
