//! Sổ cái trong bộ nhớ: append-only, ghi idempotent theo `idempotency_key`.
//! Số dư không được lưu: luôn tính lại từ posting (projection).

use std::collections::{BTreeMap, HashMap};

use crate::account::AccountId;
use crate::entry::{self, JournalEntry};
use crate::entry_id::EntryId;
use crate::error::LedgerError;
use crate::money::Money;

/// Kết quả của `post`: bút toán trong sổ, mã `JE-xxxxxx` và cờ ghi lại.
#[derive(Debug)]
pub struct Posted<'a> {
    pub id: EntryId,
    pub entry: &'a JournalEntry,
    /// `true` nếu key đã có và ta trả lại bút toán cũ, không ghi thêm.
    pub replayed: bool,
}

#[derive(Debug, Default)]
pub struct Ledger {
    entries: Vec<JournalEntry>,
    by_key: HashMap<String, usize>,
}

impl Ledger {
    pub fn new() -> Ledger {
        Ledger::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Mọi bút toán theo thứ tự ghi; mã của phần tử `i` là `JE-(i+1)`.
    pub fn entries(&self) -> &[JournalEntry] {
        &self.entries
    }

    /// Tra bút toán theo mã.
    pub fn get(&self, id: EntryId) -> Option<&JournalEntry> {
        self.entries.get(id.index()?)
    }

    /// Ghi bút toán. Cùng key và cùng nội dung: trả bút toán cũ (`replayed = true`).
    /// Cùng key nhưng nội dung khác: `IdempotencyConflict`.
    pub fn post(&mut self, entry: JournalEntry) -> Result<Posted<'_>, LedgerError> {
        if let Some(&index) = self.by_key.get(entry.idempotency_key()) {
            if self.entries[index] != entry {
                return Err(LedgerError::IdempotencyConflict {
                    key: entry.idempotency_key().to_string(),
                });
            }
            return Ok(Posted {
                id: EntryId::from_index(index),
                entry: &self.entries[index],
                replayed: true,
            });
        }
        let index = self.entries.len();
        self.by_key
            .insert(entry.idempotency_key().to_string(), index);
        self.entries.push(entry);
        Ok(Posted {
            id: EntryId::from_index(index),
            entry: &self.entries[index],
            replayed: false,
        })
    }

    /// Đảo bút toán `id` bằng một bút toán MỚI; bản gốc không bị sửa hay xoá.
    /// Gọi lại với cùng `id` là replay (key suy ra từ bút toán gốc).
    pub fn reverse(&mut self, id: EntryId) -> Result<Posted<'_>, LedgerError> {
        let original = self.get(id).ok_or(LedgerError::UnknownEntry { id })?;
        let reversal = JournalEntry::reversal_of(id, original)?;
        self.post(reversal)
    }

    /// Số dư một tài khoản = tổng posting của nó, có kiểm tra tràn số.
    pub fn balance(&self, account: &AccountId) -> Result<Money, LedgerError> {
        let mut sum = Money::ZERO;
        for e in &self.entries {
            for p in e.postings() {
                if &p.account == account {
                    sum = sum.checked_add(p.amount).ok_or(LedgerError::Overflow)?;
                }
            }
        }
        Ok(sum)
    }

    /// Projection đầy đủ: số dư mọi tài khoản đã xuất hiện, sắp theo mã tài khoản.
    pub fn balances(&self) -> Result<BTreeMap<AccountId, Money>, LedgerError> {
        let mut map: BTreeMap<AccountId, Money> = BTreeMap::new();
        for e in &self.entries {
            for p in e.postings() {
                let slot = map.entry(p.account.clone()).or_insert(Money::ZERO);
                *slot = slot.checked_add(p.amount).ok_or(LedgerError::Overflow)?;
            }
        }
        Ok(map)
    }

    /// Tổng mọi posting của toàn sổ; luôn bằng 0 nếu mọi bút toán đều cân.
    pub fn grand_total(&self) -> Result<Money, LedgerError> {
        let mut sum = Money::ZERO;
        for e in &self.entries {
            sum = sum
                .checked_add(entry::total(e.postings())?)
                .ok_or(LedgerError::Overflow)?;
        }
        Ok(sum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::JournalEntryBuilder;

    fn acct(raw: &str) -> AccountId {
        AccountId::new(raw).unwrap()
    }

    fn capture_entry() -> JournalEntry {
        JournalEntryBuilder::new("trip:TN-0001:capture")
            .posting(acct("customer:CU-001"), Money::from_dong(-25_500))
            .posting(acct("psp:vixanh:clearing"), Money::from_dong(25_500))
            .build()
            .unwrap()
    }

    fn split_entry() -> JournalEntry {
        JournalEntryBuilder::new("trip:TN-0001:split")
            .posting(acct("psp:vixanh:clearing"), Money::from_dong(-25_500))
            .posting(acct("platform:revenue"), Money::from_dong(5_100))
            .posting(acct("driver:DR-001"), Money::from_dong(20_400))
            .build()
            .unwrap()
    }

    fn bal(ledger: &Ledger, raw: &str) -> Money {
        ledger.balance(&acct(raw)).unwrap()
    }

    #[test]
    fn new_ledger_is_empty() {
        let ledger = Ledger::new();
        assert!(ledger.is_empty());
        assert_eq!(ledger.len(), 0);
    }

    #[test]
    fn post_assigns_sequential_ids() {
        let mut ledger = Ledger::new();
        assert_eq!(
            ledger.post(capture_entry()).unwrap().id.to_string(),
            "JE-000001"
        );
        assert_eq!(
            ledger.post(split_entry()).unwrap().id.to_string(),
            "JE-000002"
        );
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
        let other = JournalEntryBuilder::new("trip:TN-0001:capture")
            .posting(acct("customer:CU-001"), Money::from_dong(-100))
            .posting(acct("psp:vixanh:clearing"), Money::from_dong(100))
            .build()
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
    fn two_step_flow_moves_money_through_clearing() -> Result<(), LedgerError> {
        let mut ledger = Ledger::new();
        ledger.post(capture_entry())?;
        // Sau bước 1: tiền nằm ở clearing.
        assert_eq!(
            bal(&ledger, "psp:vixanh:clearing"),
            Money::from_dong(25_500)
        );
        ledger.post(split_entry())?;
        // Sau bước 2: clearing về 0, nền tảng và tài xế nhận phần của mình.
        assert_eq!(bal(&ledger, "psp:vixanh:clearing"), Money::ZERO);
        assert_eq!(bal(&ledger, "platform:revenue"), Money::from_dong(5_100));
        assert_eq!(bal(&ledger, "driver:DR-001"), Money::from_dong(20_400));
        assert_eq!(bal(&ledger, "customer:CU-001"), Money::from_dong(-25_500));
        assert_eq!(ledger.grand_total()?, Money::ZERO);
        Ok(())
    }

    #[test]
    fn reverse_restores_balances_but_keeps_history() -> Result<(), LedgerError> {
        let mut ledger = Ledger::new();
        ledger.post(capture_entry())?;
        let split_id = ledger.post(split_entry())?.id;
        let reversal = ledger.reverse(split_id)?;
        assert_eq!(reversal.id.to_string(), "JE-000003");
        assert!(!reversal.replayed);
        assert_eq!(reversal.entry.reverses(), Some(split_id));
        assert_eq!(ledger.len(), 3);
        // Số dư trở về như sau bước 1, bản gốc JE-000002 vẫn còn nguyên.
        assert_eq!(bal(&ledger, "driver:DR-001"), Money::ZERO);
        assert_eq!(
            bal(&ledger, "psp:vixanh:clearing"),
            Money::from_dong(25_500)
        );
        assert_eq!(ledger.get(split_id), Some(&split_entry()));
        assert_eq!(ledger.grand_total()?, Money::ZERO);
        Ok(())
    }

    #[test]
    fn reversing_twice_is_a_replay() -> Result<(), LedgerError> {
        let mut ledger = Ledger::new();
        ledger.post(capture_entry())?;
        let split_id = ledger.post(split_entry())?.id;
        ledger.reverse(split_id)?;
        let again = ledger.reverse(split_id)?;
        assert!(again.replayed);
        assert_eq!(again.id.to_string(), "JE-000003");
        assert_eq!(ledger.len(), 3);
        Ok(())
    }

    #[test]
    fn reversing_unknown_entry_fails() {
        let mut ledger = Ledger::new();
        let id = EntryId::parse("JE-000009").unwrap();
        assert_eq!(
            ledger.reverse(id).map(|p| p.id),
            Err(LedgerError::UnknownEntry { id })
        );
    }

    #[test]
    fn balances_projection_agrees_with_single_balance() -> Result<(), LedgerError> {
        let mut ledger = Ledger::new();
        ledger.post(capture_entry())?;
        ledger.post(split_entry())?;
        let all = ledger.balances()?;
        assert_eq!(all.len(), 4);
        for (account, amount) in &all {
            assert_eq!(*amount, ledger.balance(account)?);
        }
        Ok(())
    }
}
