//! Sổ cái trong bộ nhớ: append-only, ghi idempotent theo `idempotency_key`.

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

    /// Tìm bút toán theo mã. `EntryId` là `Copy` nên truyền theo giá trị.
    pub fn get(&self, id: EntryId) -> Option<&JournalEntry> {
        let index = usize::try_from(id.sequence().checked_sub(1)?).ok()?;
        self.entries.get(index)
    }

    /// Số dư = tổng posting của tài khoản, có kiểm tra tràn số.
    pub fn balance(&self, account: &AccountId) -> Result<Money, LedgerError> {
        let mut sum = Money::ZERO;
        for e in &self.entries {
            for p in e.postings() {
                if &p.account == account {
                    sum = (sum + p.amount)?;
                }
            }
        }
        Ok(sum)
    }

    /// Số dư mọi tài khoản đã có posting, xếp theo mã (nhờ `Ord` của `AccountId`).
    pub fn balances(&self) -> Result<BTreeMap<AccountId, Money>, LedgerError> {
        let mut map: BTreeMap<AccountId, Money> = BTreeMap::new();
        for e in &self.entries {
            for p in e.postings() {
                let slot = map.entry(p.account.clone()).or_insert(Money::ZERO);
                *slot = (*slot + p.amount)?;
            }
        }
        Ok(map)
    }

    /// Tổng mọi posting của toàn sổ; luôn bằng 0 nếu mọi bút toán đều cân.
    pub fn grand_total(&self) -> Result<Money, LedgerError> {
        let mut sum = Money::ZERO;
        for e in &self.entries {
            sum = (sum + entry::total(e.postings())?)?;
        }
        Ok(sum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::Posting;

    fn acct(raw: &str) -> AccountId {
        raw.parse().unwrap()
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
    fn get_finds_entry_by_id() {
        let mut ledger = Ledger::new();
        let id = ledger.post(capture_entry()).unwrap().id;
        assert_eq!(
            ledger.get(id).map(JournalEntry::idempotency_key),
            Some("trip:TN-0001:capture")
        );
        let missing = EntryId::try_from(2).unwrap();
        assert!(ledger.get(missing).is_none());
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
        Ok(())
    }

    #[test]
    fn balances_are_sorted_by_account_id() -> Result<(), LedgerError> {
        let mut ledger = Ledger::new();
        ledger.post(capture_entry())?;
        let keys: Vec<String> = ledger.balances()?.keys().map(ToString::to_string).collect();
        assert_eq!(
            keys,
            ["customer:CU-001", "driver:DR-001", "platform:revenue"]
        );
        Ok(())
    }
}
