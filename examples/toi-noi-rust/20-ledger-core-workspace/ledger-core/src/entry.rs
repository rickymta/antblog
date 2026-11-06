//! Mã bút toán, posting, bút toán (journal entry) và builder: bút toán chỉ tồn tại khi cân.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::account::AccountId;
use crate::error::LedgerError;
use crate::money::Money;

/// Mã bút toán dạng `JE-000001`. Chỉ [`Ledger`](crate::Ledger) cấp mã, theo thứ tự ghi.
///
/// ```
/// use ledger_core::EntryId;
///
/// let id = EntryId::parse("JE-000001").unwrap();
/// assert_eq!(id.to_string(), "JE-000001");
/// assert!(EntryId::parse("JE-000000").is_none());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct EntryId(u64);

impl EntryId {
    // Chỉ sổ cái cấp mã, theo chỉ số trong `Vec`.
    pub(crate) fn from_index(index: usize) -> EntryId {
        EntryId(index as u64 + 1)
    }

    // Ngược lại của `from_index`; `None` nếu không vừa `usize`.
    pub(crate) fn index(self) -> Option<usize> {
        usize::try_from(self.0.checked_sub(1)?).ok()
    }

    /// Đọc `JE-000001`; `None` nếu sai dạng chuẩn (`JE-1`, `JE-+00001`) hoặc số thứ tự bằng 0.
    pub fn parse(raw: &str) -> Option<EntryId> {
        let n: u64 = raw.strip_prefix("JE-")?.parse().ok()?;
        let id = EntryId(n);
        (n > 0 && id.to_string() == raw).then_some(id)
    }
}

impl fmt::Display for EntryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(&format!("JE-{:06}", self.0))
    }
}

impl TryFrom<String> for EntryId {
    type Error = LedgerError;

    fn try_from(raw: String) -> Result<EntryId, LedgerError> {
        EntryId::parse(&raw).ok_or(LedgerError::InvalidEntryId { raw })
    }
}

impl From<EntryId> for String {
    fn from(id: EntryId) -> String {
        id.to_string()
    }
}

/// Một dòng của bút toán: số tiền có dấu (dương vào tài khoản, âm ra khỏi tài khoản).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Posting {
    /// Tài khoản bị ảnh hưởng.
    pub account: AccountId,
    /// Số tiền có dấu.
    pub amount: Money,
}

impl Posting {
    /// Tạo một posting.
    pub fn new(account: AccountId, amount: Money) -> Posting {
        Posting { account, amount }
    }
}

/// Bút toán đã hợp lệ. Trường private nên chỉ [`JournalEntry::try_new`], builder,
/// [`JournalEntry::reversal_of`] hoặc đọc JSON (cũng qua kiểm tra) tạo được.
///
/// JSON dùng camelCase; trường `reverses` chỉ xuất hiện ở bút toán đảo.
///
/// ```
/// use ledger_core::{AccountId, JournalEntryBuilder, Money};
///
/// let entry = JournalEntryBuilder::new("trip:TN-0001:capture")
///     .posting(AccountId::new("customer:CU-001")?, Money::from_dong(-5_100))
///     .posting(AccountId::new("platform:revenue")?, Money::from_dong(5_100))
///     .build()?;
/// assert_eq!(entry.postings().len(), 2);
/// # Ok::<(), ledger_core::LedgerError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "EntryWire")]
pub struct JournalEntry {
    idempotency_key: String,
    postings: Vec<Posting>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reverses: Option<EntryId>,
}

/// Hình dạng dữ liệu trên dây, chưa kiểm bất biến. Riêng tư với crate.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EntryWire {
    idempotency_key: String,
    postings: Vec<Posting>,
    #[serde(default)]
    reverses: Option<EntryId>,
}

impl TryFrom<EntryWire> for JournalEntry {
    type Error = LedgerError;

    fn try_from(wire: EntryWire) -> Result<JournalEntry, LedgerError> {
        JournalEntry::validated(&wire.idempotency_key, wire.postings, wire.reverses)
    }
}

/// Tổng có kiểm tra tràn số của một danh sách posting.
fn total(postings: &[Posting]) -> Result<Money, LedgerError> {
    postings
        .iter()
        .try_fold(Money::ZERO, |sum, p| sum.checked_add(p.amount))
        .ok_or(LedgerError::Overflow)
}

impl JournalEntry {
    // Cổng kiểm tra duy nhất: try_new, builder, bút toán đảo và deserialize cùng đi qua đây.
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
    /// # Errors
    /// [`LedgerError::EmptyIdempotencyKey`], [`LedgerError::TooFewPostings`],
    /// [`LedgerError::Overflow`] hoặc [`LedgerError::Unbalanced`].
    pub fn try_new(
        idempotency_key: &str,
        postings: Vec<Posting>,
    ) -> Result<JournalEntry, LedgerError> {
        JournalEntry::validated(idempotency_key, postings, None)
    }

    /// Bút toán đảo của `original` (có mã `original_id`): cùng tài khoản, mọi số tiền đổi dấu.
    /// Key suy ra từ bút toán gốc (`<key>:reversal`), nên đảo hai lần là cùng một bút toán.
    ///
    /// # Errors
    /// [`LedgerError::Overflow`] nếu có posting bằng `i64::MIN` (không có đối số dương).
    pub fn reversal_of(
        original_id: EntryId,
        original: &JournalEntry,
    ) -> Result<JournalEntry, LedgerError> {
        let postings = original
            .postings
            .iter()
            .map(|p| {
                let flipped = p.amount.checked_neg().ok_or(LedgerError::Overflow)?;
                Ok(Posting::new(p.account.clone(), flipped))
            })
            .collect::<Result<Vec<_>, LedgerError>>()?;
        let key = format!("{}:reversal", original.idempotency_key);
        JournalEntry::validated(&key, postings, Some(original_id))
    }

    /// Khóa idempotency của bút toán.
    pub fn idempotency_key(&self) -> &str {
        &self.idempotency_key
    }

    /// Các posting, chỉ đọc.
    pub fn postings(&self) -> &[Posting] {
        &self.postings
    }

    /// Mã bút toán mà bút toán này đảo; `None` nếu là bút toán thường.
    pub fn reverses(&self) -> Option<EntryId> {
        self.reverses
    }
}

/// Builder cho [`JournalEntry`]: thêm posting từng dòng, kiểm tra một lần ở `build`.
#[derive(Debug, Clone)]
pub struct JournalEntryBuilder {
    idempotency_key: String,
    postings: Vec<Posting>,
}

impl JournalEntryBuilder {
    /// Bắt đầu dựng bút toán với `idempotency_key` cho trước.
    pub fn new(idempotency_key: &str) -> JournalEntryBuilder {
        JournalEntryBuilder {
            idempotency_key: idempotency_key.to_string(),
            postings: Vec::new(),
        }
    }

    /// Thêm một posting.
    #[must_use]
    pub fn posting(mut self, account: AccountId, amount: Money) -> JournalEntryBuilder {
        self.postings.push(Posting::new(account, amount));
        self
    }

    /// Kiểm tra và tạo bút toán, cùng quy tắc với [`JournalEntry::try_new`].
    ///
    /// # Errors
    /// Như [`JournalEntry::try_new`].
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

    fn capture() -> JournalEntry {
        JournalEntryBuilder::new("trip:TN-0001:capture")
            .posting(acct("customer:CU-001"), Money::from_dong(-25_500))
            .posting(acct("platform:revenue"), Money::from_dong(5_100))
            .posting(acct("driver:DR-001"), Money::from_dong(20_400))
            .build()
            .unwrap()
    }

    #[test]
    fn balanced_entry_is_accepted() {
        let entry = capture();
        assert_eq!(entry.postings().len(), 3);
        assert_eq!(entry.idempotency_key(), "trip:TN-0001:capture");
        assert_eq!(entry.reverses(), None);
    }

    #[test]
    fn unbalanced_entry_is_rejected() {
        let result = JournalEntry::try_new("k", vec![posting("a", -25_500), posting("b", 20_400)]);
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
    fn builder_and_try_new_agree() {
        let by_try_new = JournalEntry::try_new(
            "trip:TN-0001:capture",
            vec![
                posting("customer:CU-001", -25_500),
                posting("platform:revenue", 5_100),
                posting("driver:DR-001", 20_400),
            ],
        );
        assert_eq!(by_try_new, Ok(capture()));
    }

    #[test]
    fn reversal_flips_every_sign_and_links_back() -> Result<(), LedgerError> {
        let id = EntryId::from_index(0);
        let reversal = JournalEntry::reversal_of(id, &capture())?;
        let amounts: Vec<i64> = reversal
            .postings()
            .iter()
            .map(|p| p.amount.dong())
            .collect();
        assert_eq!(amounts, vec![25_500, -5_100, -20_400]);
        assert_eq!(reversal.idempotency_key(), "trip:TN-0001:capture:reversal");
        assert_eq!(reversal.reverses(), Some(id));
        Ok(())
    }

    #[test]
    fn reversal_of_min_overflows() {
        let entry = JournalEntry::try_new(
            "k",
            vec![
                posting("a", i64::MIN),
                posting("b", i64::MAX),
                posting("c", 1),
            ],
        )
        .unwrap();
        assert_eq!(
            JournalEntry::reversal_of(EntryId::from_index(0), &entry),
            Err(LedgerError::Overflow)
        );
    }

    #[test]
    fn json_roundtrip_keeps_the_entry() {
        let entry = capture();
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"idempotencyKey\""));
        assert!(!json.contains("reverses"));
        let back: JournalEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back, entry);
    }

    #[test]
    fn json_roundtrip_keeps_the_reversal_link() {
        let reversal = JournalEntry::reversal_of(EntryId::from_index(0), &capture()).unwrap();
        let json = serde_json::to_string(&reversal).unwrap();
        assert!(json.contains("\"reverses\":\"JE-000001\""));
        let back: JournalEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back, reversal);
    }

    #[test]
    fn json_unbalanced_entry_is_rejected_on_read() {
        let json = r#"{"idempotencyKey":"k","postings":[
            {"account":"a","amount":-100},{"account":"b","amount":90}]}"#;
        let result: Result<JournalEntry, _> = serde_json::from_str(json);
        let message = result.unwrap_err().to_string();
        assert!(message.contains("lệch"), "{message}");
    }

    #[test]
    fn json_unknown_field_is_rejected() {
        let json = r#"{"idempotencyKey":"k","postings":[
            {"account":"a","amount":-1},{"account":"b","amount":1}],"memo":"x"}"#;
        let result: Result<JournalEntry, _> = serde_json::from_str(json);
        assert!(result.unwrap_err().to_string().contains("unknown field"));
    }

    #[test]
    fn entry_id_display_parse_and_zero() {
        assert_eq!(EntryId::from_index(41).to_string(), "JE-000042");
        assert_eq!(EntryId::parse("JE-000042"), Some(EntryId::from_index(41)));
        assert_eq!(EntryId::parse("JE-000000"), None);
        assert_eq!(EntryId::parse("XX-000001"), None);
        assert_eq!(EntryId::parse("JE-1"), None);
        assert_eq!(EntryId::parse("JE-+00001"), None);
        assert_eq!(EntryId::from_index(5).index(), Some(5));
    }
}
