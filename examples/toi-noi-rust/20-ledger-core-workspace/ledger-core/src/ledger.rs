//! Sổ cái trong bộ nhớ: append-only, ghi idempotent theo `idempotency_key`.

use std::collections::{BTreeMap, HashMap};

use crate::account::AccountId;
use crate::entry::{EntryId, JournalEntry};
use crate::error::LedgerError;
use crate::money::Money;

/// Kết quả của [`Ledger::post`] và [`Ledger::reverse`].
#[derive(Debug)]
pub struct Posted<'a> {
    /// Mã `JE-xxxxxx` của bút toán trong sổ.
    pub id: EntryId,
    /// Bút toán trong sổ (mượn từ sổ, không sở hữu).
    pub entry: &'a JournalEntry,
    /// `true` nếu key đã có và ta trả lại bút toán cũ, không ghi thêm.
    pub replayed: bool,
}

/// Sổ cái append-only. Không có phương thức sửa hay xóa bút toán đã ghi;
/// số dư luôn được tính lại từ bút toán (projection), không lưu riêng.
///
/// ```
/// use ledger_core::{AccountId, JournalEntry, Ledger, Money, Posting};
///
/// let mut ledger = Ledger::new();
/// let entry = JournalEntry::try_new(
///     "trip:TN-0001:capture",
///     vec![
///         Posting::new(AccountId::new("customer:CU-001")?, Money::from_dong(-25_500)),
///         Posting::new(AccountId::new("driver:DR-001")?, Money::from_dong(25_500)),
///     ],
/// )?;
/// let posted = ledger.post(entry)?;
/// assert_eq!(posted.id.to_string(), "JE-000001");
/// assert_eq!(
///     ledger.balance(&AccountId::new("driver:DR-001")?)?,
///     Money::from_dong(25_500)
/// );
/// # Ok::<(), ledger_core::LedgerError>(())
/// ```
///
/// Bút toán đã ghi không sửa được từ bên ngoài, vì `entries()` chỉ trả slice chỉ đọc:
///
/// ```compile_fail
/// let ledger = ledger_core::Ledger::new();
/// ledger.entries().push(todo!());
/// ```
#[derive(Debug, Default)]
pub struct Ledger {
    entries: Vec<JournalEntry>,
    by_key: HashMap<String, usize>,
}

impl Ledger {
    /// Sổ cái rỗng.
    pub fn new() -> Ledger {
        Ledger::default()
    }

    /// Số bút toán đã ghi.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` nếu chưa có bút toán nào.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Mọi bút toán theo thứ tự ghi, chỉ đọc.
    pub fn entries(&self) -> &[JournalEntry] {
        &self.entries
    }

    /// Tìm bút toán theo mã.
    pub fn get(&self, id: EntryId) -> Option<&JournalEntry> {
        self.entries.get(id.index()?)
    }

    fn posted_at(&self, index: usize, replayed: bool) -> Posted<'_> {
        Posted {
            id: EntryId::from_index(index),
            entry: &self.entries[index],
            replayed,
        }
    }

    /// Ghi bút toán. Cùng key và cùng nội dung: trả bút toán cũ (`replayed = true`).
    ///
    /// # Errors
    /// [`LedgerError::IdempotencyConflict`] nếu key đã dùng cho nội dung khác.
    pub fn post(&mut self, entry: JournalEntry) -> Result<Posted<'_>, LedgerError> {
        if let Some(&index) = self.by_key.get(entry.idempotency_key()) {
            if self.entries[index] != entry {
                return Err(LedgerError::IdempotencyConflict {
                    key: entry.idempotency_key().to_string(),
                });
            }
            return Ok(self.posted_at(index, true));
        }
        let index = self.entries.len();
        self.by_key
            .insert(entry.idempotency_key().to_string(), index);
        self.entries.push(entry);
        Ok(self.posted_at(index, false))
    }

    /// Đảo bút toán `id` bằng một bút toán MỚI; bản gốc không bị sửa hay xoá.
    /// Key của bút toán đảo suy ra từ bản gốc, nên gọi lại với cùng `id` là replay.
    ///
    /// # Errors
    /// [`LedgerError::UnknownEntry`] nếu không có `id`, cùng các lỗi của
    /// [`JournalEntry::reversal_of`] và [`Ledger::post`].
    pub fn reverse(&mut self, id: EntryId) -> Result<Posted<'_>, LedgerError> {
        let original = self.get(id).ok_or(LedgerError::UnknownEntry { id })?;
        let reversal = JournalEntry::reversal_of(id, original)?;
        self.post(reversal)
    }

    /// Số dư của một tài khoản = tổng posting của nó.
    ///
    /// # Errors
    /// [`LedgerError::Overflow`] nếu tổng vượt `i64`.
    pub fn balance(&self, account: &AccountId) -> Result<Money, LedgerError> {
        self.entries
            .iter()
            .flat_map(JournalEntry::postings)
            .filter(|p| &p.account == account)
            .try_fold(Money::ZERO, |sum, p| sum.checked_add(p.amount))
            .ok_or(LedgerError::Overflow)
    }

    /// Số dư mọi tài khoản đã xuất hiện, sắp theo mã tài khoản (projection đầy đủ).
    ///
    /// # Errors
    /// [`LedgerError::Overflow`] nếu một tổng vượt `i64`.
    pub fn balances(&self) -> Result<BTreeMap<AccountId, Money>, LedgerError> {
        self.entries
            .iter()
            .flat_map(JournalEntry::postings)
            .try_fold(BTreeMap::new(), |mut map, p| {
                let slot = map.entry(p.account.clone()).or_insert(Money::ZERO);
                *slot = slot.checked_add(p.amount).ok_or(LedgerError::Overflow)?;
                Ok(map)
            })
    }

    /// Tổng mọi posting của toàn sổ; luôn bằng 0 vì mọi bút toán đều cân.
    ///
    /// # Errors
    /// [`LedgerError::Overflow`] nếu một tổng riêng phần vượt `i64`.
    pub fn grand_total(&self) -> Result<Money, LedgerError> {
        self.entries
            .iter()
            .flat_map(JournalEntry::postings)
            .try_fold(Money::ZERO, |sum, p| sum.checked_add(p.amount))
            .ok_or(LedgerError::Overflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::Posting;

    fn acct(raw: &str) -> AccountId {
        AccountId::new(raw).unwrap()
    }

    fn capture_entry() -> JournalEntry {
        JournalEntry::try_new(
            "trip:TN-0001:capture",
            vec![
                Posting::new(acct("customer:CU-001"), Money::from_dong(-25_500)),
                Posting::new(acct("platform:revenue"), Money::from_dong(5_100)),
                Posting::new(acct("driver:DR-001"), Money::from_dong(20_400)),
            ],
        )
        .unwrap()
    }

    #[test]
    fn new_ledger_is_empty() {
        let ledger = Ledger::new();
        assert!(ledger.is_empty());
        assert_eq!(ledger.len(), 0);
    }

    #[test]
    fn post_assigns_sequential_id() {
        let mut ledger = Ledger::new();
        let posted = ledger.post(capture_entry()).unwrap();
        assert_eq!(posted.id.to_string(), "JE-000001");
        assert!(!posted.replayed);
    }

    #[test]
    fn post_same_key_is_idempotent() {
        let mut ledger = Ledger::new();
        ledger.post(capture_entry()).unwrap();
        let again = ledger.post(capture_entry()).unwrap();
        assert!(again.replayed);
        assert_eq!(again.id.to_string(), "JE-000001");
        assert_eq!(ledger.len(), 1);
    }

    #[test]
    fn same_key_different_content_conflicts() {
        let mut ledger = Ledger::new();
        ledger.post(capture_entry()).unwrap();
        let other = JournalEntry::try_new(
            "trip:TN-0001:capture",
            vec![
                Posting::new(acct("customer:CU-001"), Money::from_dong(-100)),
                Posting::new(acct("driver:DR-001"), Money::from_dong(100)),
            ],
        )
        .unwrap();
        let err = ledger.post(other).unwrap_err();
        assert_eq!(
            err,
            LedgerError::IdempotencyConflict {
                key: "trip:TN-0001:capture".to_string()
            }
        );
        assert_eq!(ledger.len(), 1);
    }

    #[test]
    fn balance_and_grand_total() -> Result<(), LedgerError> {
        let mut ledger = Ledger::new();
        ledger.post(capture_entry())?;
        assert_eq!(
            ledger.balance(&acct("driver:DR-001"))?,
            Money::from_dong(20_400)
        );
        assert_eq!(
            ledger.balance(&acct("customer:CU-001"))?,
            Money::from_dong(-25_500)
        );
        assert_eq!(ledger.balance(&acct("psp:vixanh:clearing"))?, Money::ZERO);
        assert_eq!(ledger.grand_total()?, Money::ZERO);
        assert_eq!(ledger.balances()?.len(), 3);
        Ok(())
    }

    #[test]
    fn reverse_brings_every_balance_back_to_zero() -> Result<(), LedgerError> {
        let mut ledger = Ledger::new();
        let id = ledger.post(capture_entry())?.id;
        let reversal = ledger.reverse(id)?;
        assert_eq!(reversal.id.to_string(), "JE-000002");
        assert_eq!(reversal.entry.reverses(), Some(id));
        assert!(ledger.balances()?.values().all(|m| *m == Money::ZERO));
        // Bút toán gốc vẫn còn nguyên trong sổ.
        assert_eq!(ledger.get(id), Some(&capture_entry()));
        Ok(())
    }

    #[test]
    fn reversing_twice_is_a_replay() -> Result<(), LedgerError> {
        let mut ledger = Ledger::new();
        let id = ledger.post(capture_entry())?.id;
        ledger.reverse(id)?;
        let again = ledger.reverse(id)?;
        assert!(again.replayed);
        assert_eq!(again.id.to_string(), "JE-000002");
        assert_eq!(ledger.len(), 2);
        Ok(())
    }

    #[test]
    fn reverse_unknown_entry_fails() {
        let mut ledger = Ledger::new();
        let id = EntryId::parse("JE-000009").unwrap();
        assert_eq!(
            ledger.reverse(id).map(|p| p.id),
            Err(LedgerError::UnknownEntry { id })
        );
    }
}
