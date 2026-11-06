//! Sổ cái kèm số dư cộng dồn: đổi bộ nhớ lấy tốc độ đọc. Nguồn sự thật vẫn là các bút toán.

use std::collections::HashMap;

use ledger_core::{AccountId, JournalEntry, Ledger, LedgerError, Money};

/// `Ledger` + `HashMap` số dư được cập nhật ngay lúc ghi.
#[derive(Debug, Default)]
pub struct IndexedLedger {
    ledger: Ledger,
    running: HashMap<AccountId, Money>,
}

impl IndexedLedger {
    pub fn new() -> IndexedLedger {
        IndexedLedger::default()
    }

    pub fn ledger(&self) -> &Ledger {
        &self.ledger
    }

    /// Ghi bút toán; chỉ cộng dồn khi bút toán là MỚI (replay không làm đổi số dư).
    ///
    /// # Errors
    /// Lỗi của [`Ledger::post`], hoặc `Overflow` nếu số dư cộng dồn tràn `i64`.
    pub fn post(&mut self, entry: JournalEntry) -> Result<bool, LedgerError> {
        // Mượn rời hai trường của `self`: `ledger` (mut) và `running` (mut) không đè nhau.
        let posted = self.ledger.post(entry)?;
        if posted.replayed {
            return Ok(false);
        }
        for p in posted.entry.postings() {
            if let Some(slot) = self.running.get_mut(&p.account) {
                *slot = slot.checked_add(p.amount).ok_or(LedgerError::Overflow)?;
            } else {
                // Chỉ clone khóa khi tài khoản xuất hiện lần đầu.
                self.running.insert(p.account.clone(), p.amount);
            }
        }
        Ok(true)
    }

    /// Số dư O(1) từ bảng cộng dồn; tài khoản chưa có posting trả 0.
    pub fn balance(&self, account: &AccountId) -> Money {
        self.running.get(account).copied().unwrap_or(Money::ZERO)
    }
}
