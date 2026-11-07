//! Adapter bộ nhớ: dùng cho test nhanh và cho chạy thử không cần hạ tầng.

use std::collections::HashMap;
use std::sync::Mutex;

use application::{EventPublisher, LedgerStore, PostOutcome, PublishError, StoreError};
use domain::{AccountId, EntryId, EntryPosted, JournalEntry, LedgerError, Money};

use crate::CHART;
use crate::wire;

#[derive(Default)]
struct State {
    // (key, các dòng (tài khoản, số tiền)) theo thứ tự ghi; chỉ số + 1 là mã JE-xxxxxx.
    entries: Vec<(String, Vec<(String, i64)>)>,
    balances: HashMap<String, i64>,
}

/// Kho trong bộ nhớ. Cùng hợp đồng với `PgStore`: idempotent, từ chối tài khoản lạ.
#[derive(Default)]
pub struct MemoryStore {
    state: Mutex<State>,
}

impl MemoryStore {
    /// Kho rỗng, chỉ biết các tài khoản của [`CHART`].
    pub fn new() -> MemoryStore {
        MemoryStore::default()
    }
}

fn entry_id(index: usize) -> Result<EntryId, StoreError> {
    EntryId::parse(&format!("JE-{:06}", index + 1))
        .ok_or_else(|| StoreError::Storage("mã bút toán ngoài khoảng".into()))
}

fn lines(entry: &JournalEntry) -> Vec<(String, i64)> {
    entry
        .postings()
        .iter()
        .map(|p| (p.account.as_str().to_string(), p.amount.dong()))
        .collect()
}

impl LedgerStore for MemoryStore {
    async fn post(&self, entry: &JournalEntry) -> Result<PostOutcome, StoreError> {
        if let Some(p) = entry
            .postings()
            .iter()
            .find(|p| !CHART.iter().any(|(id, _)| *id == p.account.as_str()))
        {
            return Err(StoreError::UnknownAccount(p.account.as_str().to_string()));
        }
        let wanted = lines(entry);
        // Khoá `std::sync::Mutex` chỉ giữ trong đoạn không có `.await`.
        let mut state = self
            .state
            .lock()
            .map_err(|_| StoreError::Storage("mutex hỏng".into()))?;
        if let Some(i) = state
            .entries
            .iter()
            .position(|(k, _)| k == entry.idempotency_key())
        {
            if state.entries[i].1 != wanted {
                return Err(StoreError::IdempotencyConflict(
                    entry.idempotency_key().to_string(),
                ));
            }
            return Ok(PostOutcome {
                id: entry_id(i)?,
                replayed: true,
            });
        }
        for (account, dong) in &wanted {
            let slot = state.balances.entry(account.clone()).or_insert(0);
            *slot = slot.checked_add(*dong).ok_or(LedgerError::Overflow)?;
        }
        state
            .entries
            .push((entry.idempotency_key().to_string(), wanted));
        Ok(PostOutcome {
            id: entry_id(state.entries.len() - 1)?,
            replayed: false,
        })
    }

    async fn balance(&self, account: &AccountId) -> Result<Money, StoreError> {
        let state = self
            .state
            .lock()
            .map_err(|_| StoreError::Storage("mutex hỏng".into()))?;
        Ok(Money::from_dong(
            state.balances.get(account.as_str()).copied().unwrap_or(0),
        ))
    }
}

/// Nơi phát sự kiện trong bộ nhớ: giữ JSON đúng như sẽ gửi lên Kafka.
#[derive(Default)]
pub struct MemoryPublisher {
    sent: Mutex<Vec<String>>,
}

impl MemoryPublisher {
    /// Publisher rỗng.
    pub fn new() -> MemoryPublisher {
        MemoryPublisher::default()
    }

    /// Các JSON đã phát, theo thứ tự.
    pub fn sent(&self) -> Vec<String> {
        self.sent.lock().map(|v| v.clone()).unwrap_or_default()
    }
}

impl EventPublisher for MemoryPublisher {
    async fn publish(&self, event: &EntryPosted) -> Result<(), PublishError> {
        let json = wire::to_json(event)?;
        self.sent
            .lock()
            .map_err(|_| PublishError("mutex hỏng".into()))?
            .push(json);
        Ok(())
    }
}
