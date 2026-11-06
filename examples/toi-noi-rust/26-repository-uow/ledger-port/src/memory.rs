//! Adapter bộ nhớ: dùng cho test domain không cần database.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use ledger_core::{AccountId, JournalEntry, Money};
use tokio::sync::{Mutex, OwnedMutexGuard};

use crate::error::RepoError;
use crate::repository::{LedgerRepository, LedgerUnitOfWork, PostOutcome, validate_entry};
use crate::{CHART, entry_id_from_i64};

#[derive(Debug, Clone, Default)]
struct State {
    entries: Vec<JournalEntry>,
    by_key: HashMap<String, usize>,
    balances: BTreeMap<AccountId, Money>,
    accounts: BTreeSet<AccountId>,
}

impl State {
    fn balance(&self, account: &AccountId) -> Money {
        self.balances.get(account).copied().unwrap_or(Money::ZERO)
    }
}

/// Sổ cái trong bộ nhớ. Clone rẻ: các bản sao dùng chung một trạng thái.
#[derive(Debug, Clone)]
pub struct MemoryRepository {
    state: Arc<Mutex<State>>,
}

impl MemoryRepository {
    /// Sổ rỗng đã mở sẵn các tài khoản của [`CHART`].
    pub fn with_chart() -> MemoryRepository {
        let accounts = CHART
            .iter()
            .map(|(id, _)| AccountId::new(id).expect("CHART hợp lệ"))
            .collect();
        MemoryRepository {
            state: Arc::new(Mutex::new(State {
                accounts,
                ..State::default()
            })),
        }
    }
}

/// Giao dịch bộ nhớ: giữ khóa của kho suốt đời Uow (một người ghi tại một thời điểm) và ghi
/// vào bản nháp `staged`. `commit` đổi bản nháp thành trạng thái thật; thả Uow là bỏ bản nháp.
#[derive(Debug)]
pub struct MemoryUow {
    guard: OwnedMutexGuard<State>,
    staged: State,
}

impl LedgerRepository for MemoryRepository {
    type Uow = MemoryUow;

    async fn begin(&self) -> Result<MemoryUow, RepoError> {
        let guard = Arc::clone(&self.state).lock_owned().await;
        let staged = guard.clone();
        Ok(MemoryUow { guard, staged })
    }

    async fn balance(&self, account: &AccountId) -> Result<Money, RepoError> {
        Ok(self.state.lock().await.balance(account))
    }
}

impl LedgerUnitOfWork for MemoryUow {
    async fn post(&mut self, entry: &JournalEntry) -> Result<PostOutcome, RepoError> {
        validate_entry(entry)?;
        let s = &mut self.staged;
        if let Some(p) = entry
            .postings()
            .iter()
            .find(|p| !s.accounts.contains(&p.account))
        {
            return Err(RepoError::UnknownAccount(p.account.clone()));
        }
        if let Some(&index) = s.by_key.get(entry.idempotency_key()) {
            if s.entries[index] != *entry {
                return Err(ledger_core::LedgerError::IdempotencyConflict {
                    key: entry.idempotency_key().to_string(),
                }
                .into());
            }
            let id = entry_id_from_i64(index as i64 + 1).expect("index hợp lệ");
            return Ok(PostOutcome { id, replayed: true });
        }
        // Tính số dư mới trên bản sao trước, để lỗi tràn số không để lại trạng thái nửa vời.
        let mut next = s.balances.clone();
        for p in entry.postings() {
            let cur = next.get(&p.account).copied().unwrap_or(Money::ZERO);
            let sum = cur
                .checked_add(p.amount)
                .ok_or(ledger_core::LedgerError::Overflow)?;
            next.insert(p.account.clone(), sum);
        }
        s.balances = next;
        s.by_key
            .insert(entry.idempotency_key().to_string(), s.entries.len());
        s.entries.push(entry.clone());
        let id = entry_id_from_i64(s.entries.len() as i64).expect("độ dài hợp lệ");
        Ok(PostOutcome {
            id,
            replayed: false,
        })
    }

    async fn balance(&mut self, account: &AccountId) -> Result<Money, RepoError> {
        Ok(self.staged.balance(account))
    }

    async fn commit(self) -> Result<(), RepoError> {
        let MemoryUow { mut guard, staged } = self;
        *guard = staged;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::contract;

    #[tokio::test(flavor = "multi_thread")]
    async fn memory_adapter_passes_contract() {
        let repo = Arc::new(MemoryRepository::with_chart());
        let passed = contract::run(Arc::clone(&repo)).await.unwrap();
        assert_eq!(passed.len(), 7);
        contract::run_concurrent(repo).await.unwrap();
    }
}
