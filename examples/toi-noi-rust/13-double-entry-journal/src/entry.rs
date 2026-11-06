//! Posting, bút toán (journal entry) và builder: bút toán chỉ tồn tại khi cân.

use crate::account::AccountId;
use crate::entry_id::EntryId;
use crate::error::LedgerError;
use crate::money::Money;

/// Chiều tiền của một posting đối với tài khoản của nó.
/// Đây KHÔNG phải nợ/có của kế toán: nợ/có còn tùy loại tài khoản (xem `chart.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    /// Tiền vào tài khoản: số dương.
    In,
    /// Tiền ra khỏi tài khoản: số âm.
    Out,
}

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

    /// Chiều tiền của dòng; `None` nếu số tiền bằng 0.
    pub fn flow(&self) -> Option<Flow> {
        match self.amount.dong() {
            n if n > 0 => Some(Flow::In),
            n if n < 0 => Some(Flow::Out),
            _ => None,
        }
    }
}

/// Bút toán đã hợp lệ. Mọi trường private: chỉ [`JournalEntry::try_new`],
/// [`JournalEntryBuilder::build`] và [`JournalEntry::reversal_of`] tạo được,
/// và cả ba đều đi qua cùng một hàm kiểm tra.
///
/// Không thể dựng bằng struct literal từ crate khác, nên không có đường vòng qua kiểm cân:
///
/// ```compile_fail
/// let _e = ledger_lab::JournalEntry {
///     idempotency_key: String::new(),
///     postings: vec![],
///     reverses: None,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalEntry {
    idempotency_key: String,
    postings: Vec<Posting>,
    /// Với bút toán đảo: mã bút toán gốc bị đảo.
    reverses: Option<EntryId>,
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
    /// Cửa duy nhất tạo bút toán: kiểm key, số posting, tổng bằng 0.
    fn validated(
        idempotency_key: &str,
        postings: Vec<Posting>,
        reverses: Option<EntryId>,
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
            reverses,
        })
    }

    /// Tạo bút toán: ít nhất 2 posting, tổng có dấu bằng 0, không tràn số.
    ///
    /// ```
    /// use ledger_lab::{AccountId, JournalEntry, Money, Posting};
    ///
    /// let a = AccountId::new("customer:CU-001").unwrap();
    /// let b = AccountId::new("psp:vixanh:clearing").unwrap();
    /// let entry = JournalEntry::try_new(
    ///     "trip:TN-0001:capture",
    ///     vec![
    ///         Posting::new(a, Money::from_dong(-25_500)),
    ///         Posting::new(b, Money::from_dong(25_500)),
    ///     ],
    /// );
    /// assert!(entry.is_ok());
    /// ```
    pub fn try_new(
        idempotency_key: &str,
        postings: Vec<Posting>,
    ) -> Result<JournalEntry, LedgerError> {
        JournalEntry::validated(idempotency_key, postings, None)
    }

    /// Bút toán đảo: cùng tài khoản, số tiền đổi dấu, trỏ về bút toán gốc `original_id`.
    /// Không sửa, không xoá bản gốc. Key là key gốc kèm `:reversal`, nên đảo hai lần
    /// cùng một bút toán chỉ ghi một lần.
    pub fn reversal_of(
        original_id: EntryId,
        original: &JournalEntry,
    ) -> Result<JournalEntry, LedgerError> {
        let mut postings = Vec::with_capacity(original.postings.len());
        for p in &original.postings {
            let flipped = p.amount.checked_neg().ok_or(LedgerError::Overflow)?;
            postings.push(Posting::new(p.account.clone(), flipped));
        }
        let key = format!("{}:reversal", original.idempotency_key);
        JournalEntry::validated(&key, postings, Some(original_id))
    }

    pub fn idempotency_key(&self) -> &str {
        &self.idempotency_key
    }

    pub fn postings(&self) -> &[Posting] {
        &self.postings
    }

    /// Bút toán gốc mà bút toán này đảo (`None` nếu là bút toán thường).
    pub fn reverses(&self) -> Option<EntryId> {
        self.reverses
    }
}

/// Builder: thêm từng posting rồi `build()` kiểm cân một lần ở cuối.
/// Mỗi phương thức nhận `self` và trả `Self` (ownership, bài 04), nên viết nối chuỗi được.
#[derive(Debug, Clone)]
pub struct JournalEntryBuilder {
    idempotency_key: String,
    postings: Vec<Posting>,
}

impl JournalEntryBuilder {
    pub fn new(idempotency_key: &str) -> JournalEntryBuilder {
        JournalEntryBuilder {
            idempotency_key: idempotency_key.to_string(),
            postings: Vec::new(),
        }
    }

    /// Thêm một posting; chưa kiểm gì cả, kiểm ở `build`.
    pub fn posting(mut self, account: AccountId, amount: Money) -> JournalEntryBuilder {
        self.postings.push(Posting::new(account, amount));
        self
    }

    /// Kiểm và tạo bút toán, đúng các luật của [`JournalEntry::try_new`].
    pub fn build(self) -> Result<JournalEntry, LedgerError> {
        JournalEntry::try_new(&self.idempotency_key, self.postings)
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
            "trip:TN-0001:split",
            vec![
                posting("psp:vixanh:clearing", -25_500),
                posting("platform:revenue", 5_100),
                posting("driver:DR-001", 20_400),
            ],
        )?;
        assert_eq!(entry.postings().len(), 3);
        assert_eq!(entry.idempotency_key(), "trip:TN-0001:split");
        assert_eq!(entry.reverses(), None);
        Ok(())
    }

    #[test]
    fn unbalanced_entry_is_rejected() {
        let result = JournalEntry::try_new(
            "k",
            vec![
                posting("psp:vixanh:clearing", -25_500),
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

    #[test]
    fn flow_follows_sign() {
        assert_eq!(posting("a", 5).flow(), Some(Flow::In));
        assert_eq!(posting("a", -5).flow(), Some(Flow::Out));
        assert_eq!(posting("a", 0).flow(), None);
    }

    #[test]
    fn builder_matches_try_new() -> Result<(), LedgerError> {
        let built = JournalEntryBuilder::new("trip:TN-0001:split")
            .posting(acct("psp:vixanh:clearing"), Money::from_dong(-25_500))
            .posting(acct("platform:revenue"), Money::from_dong(5_100))
            .posting(acct("driver:DR-001"), Money::from_dong(20_400))
            .build()?;
        let direct = JournalEntry::try_new(
            "trip:TN-0001:split",
            vec![
                posting("psp:vixanh:clearing", -25_500),
                posting("platform:revenue", 5_100),
                posting("driver:DR-001", 20_400),
            ],
        )?;
        assert_eq!(built, direct);
        Ok(())
    }

    #[test]
    fn builder_rejects_unbalanced_only_at_build() {
        // Thêm posting thứ nhất thì chưa cân, nhưng chưa lỗi: lỗi chỉ ở `build`.
        let partial = JournalEntryBuilder::new("k").posting(acct("a"), Money::from_dong(-100));
        let result = partial.posting(acct("b"), Money::from_dong(90)).build();
        assert_eq!(
            result,
            Err(LedgerError::Unbalanced {
                sum: Money::from_dong(-10)
            })
        );
    }

    #[test]
    fn reversal_negates_every_posting_and_points_to_original() -> Result<(), LedgerError> {
        let original = JournalEntry::try_new(
            "trip:TN-0001:split",
            vec![
                posting("psp:vixanh:clearing", -25_500),
                posting("platform:revenue", 5_100),
                posting("driver:DR-001", 20_400),
            ],
        )?;
        let id = EntryId::from_index(1);
        let reversal = JournalEntry::reversal_of(id, &original)?;
        assert_eq!(reversal.reverses(), Some(id));
        assert_eq!(reversal.idempotency_key(), "trip:TN-0001:split:reversal");
        assert_eq!(reversal.postings()[0].amount, Money::from_dong(25_500));
        assert_eq!(reversal.postings()[2].amount, Money::from_dong(-20_400));
        // Bản gốc nguyên vẹn.
        assert_eq!(original.postings()[0].amount, Money::from_dong(-25_500));
        Ok(())
    }

    #[test]
    fn reversal_of_min_value_overflows() {
        let original = JournalEntry::try_new(
            "k",
            vec![
                posting("a", i64::MIN),
                posting("b", i64::MAX),
                posting("c", 1),
            ],
        )
        .unwrap();
        let result = JournalEntry::reversal_of(EntryId::from_index(0), &original);
        assert_eq!(result, Err(LedgerError::Overflow));
    }

    // Đây là panic thật vì `unwrap` trên `Err`; thông điệp chứa Debug của lỗi.
    #[test]
    #[should_panic(expected = "Unbalanced")]
    fn unwrap_on_unbalanced_entry_panics() {
        JournalEntry::try_new("k", vec![posting("a", 1), posting("b", 2)]).unwrap();
    }
}
