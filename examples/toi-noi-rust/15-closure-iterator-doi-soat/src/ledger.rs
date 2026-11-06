//! Sổ cái trong bộ nhớ: append-only, ghi idempotent theo `idempotency_key`.

use std::collections::HashMap;

use crate::account::AccountId;
use crate::entry::{self, JournalEntry};
use crate::error::LedgerError;
use crate::money::Money;

/// Kết quả của `post`: bút toán trong sổ, mã `JE-xxxxxx` và cờ ghi lại.
#[derive(Debug)]
pub struct Posted<'a> {
    pub id: String,
    pub entry: &'a JournalEntry,
    /// `true` nếu key đã có và ta trả lại bút toán cũ, không ghi thêm.
    pub replayed: bool,
}

#[derive(Debug, Default)]
pub struct Ledger {
    entries: Vec<JournalEntry>,
    by_key: HashMap<String, usize>,
}

fn entry_id(index: usize) -> String {
    format!("JE-{:06}", index + 1)
}

impl Ledger {
    pub fn new() -> Ledger {
        Ledger::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.len() == 0
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
                id: entry_id(index),
                entry: &self.entries[index],
                replayed: true,
            });
        }
        let index = self.entries.len();
        self.by_key
            .insert(entry.idempotency_key().to_string(), index);
        self.entries.push(entry);
        Ok(Posted {
            id: entry_id(index),
            entry: &self.entries[index],
            replayed: false,
        })
    }

    /// Duyệt các bút toán theo thứ tự ghi, kèm mã `JE-xxxxxx`.
    ///
    /// Kiểu trả về `impl Iterator`: người gọi chỉ biết "đây là iterator của
    /// `(String, &JournalEntry)`", không thấy `Map<Enumerate<…>>` bên trong.
    /// Edition 2024 tự cho iterator mượn `&self`, không cần viết `+ '_`.
    pub fn entries_with_id(&self) -> impl Iterator<Item = (String, &JournalEntry)> {
        self.entries
            .iter()
            .enumerate()
            .map(|(index, entry)| (entry_id(index), entry))
    }

    /// Số dư = tổng posting của tài khoản, có kiểm tra tràn số.
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
        assert_eq!(posted.id, "JE-000001");
        assert!(!posted.replayed);
    }

    #[test]
    fn post_same_key_is_idempotent() {
        let mut ledger = Ledger::new();
        ledger.post(capture_entry()).unwrap();
        let again = ledger.post(capture_entry()).unwrap();
        assert!(again.replayed);
        assert_eq!(again.id, "JE-000001");
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
        Ok(())
    }
}
