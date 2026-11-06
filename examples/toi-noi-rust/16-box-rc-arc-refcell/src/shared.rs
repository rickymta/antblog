//! `RefCell` (kiểm tra mượn lúc chạy) và `Cell` (đổi giá trị `Copy` qua `&`).

use std::cell::{Cell, RefCell};
use std::panic::{self, AssertUnwindSafe};
use std::rc::Rc;

use crate::account::AccountId;
use crate::entry::{JournalEntry, Posting};
use crate::error::LedgerError;
use crate::ledger::Ledger;
use crate::money::Money;

/// Sổ cái dùng chung trong một luồng: nhiều chủ (`Rc`), sửa qua `&` (`RefCell`).
pub type SharedLedger = Rc<RefCell<Ledger>>;

pub fn new_shared() -> SharedLedger {
    Rc::new(RefCell::new(Ledger::new()))
}

/// LỖI CỐ Ý: guard `view` (mượn đọc) sống đến hết hàm, trong khi dòng sau cần mượn ghi.
/// Compiler không bắt được; chạy lên là panic `RefCell already borrowed` (Rust 1.98.1).
pub fn post_buggy(shared: &SharedLedger, entry: JournalEntry) -> Result<String, LedgerError> {
    let view = shared.borrow();
    let before = view.len();
    let id = shared.borrow_mut().post(entry)?.id;
    Ok(format!("{id} (trước đó có {before} bút toán)"))
}

/// Cách sửa 1: guard tạm hết sống ở dấu `;`, rồi mới mượn ghi.
pub fn post_fixed(shared: &SharedLedger, entry: JournalEntry) -> Result<String, LedgerError> {
    let before = shared.borrow().len();
    let id = shared.borrow_mut().post(entry)?.id;
    Ok(format!("{id} (trước đó có {before} bút toán)"))
}

/// Cách sửa 2: `try_borrow_mut` biến vi phạm thành `Err`, không panic.
pub fn post_try(shared: &SharedLedger, entry: JournalEntry) -> Result<String, LedgerError> {
    let mut ledger = shared.try_borrow_mut().map_err(|_| LedgerError::Busy)?;
    let id = ledger.post(entry)?.id;
    Ok(id)
}

/// Bút toán mẫu để dựng panic.
pub fn sample_entry() -> Result<JournalEntry, LedgerError> {
    JournalEntry::try_new(
        "trip:TN-0001:capture",
        vec![
            Posting::new(
                AccountId::new("customer:CU-001")?,
                Money::from_dong(-25_500),
            ),
            Posting::new(AccountId::new("platform:revenue")?, Money::from_dong(5_100)),
            Posting::new(AccountId::new("driver:DR-001")?, Money::from_dong(20_400)),
        ],
    )
}

/// Chạy `post_buggy` và bắt panic; trả thông điệp panic (`None` nếu không panic).
/// Tạm tắt hook mặc định để `--verify` không in stack trace ra stderr.
pub fn buggy_panic_message() -> Result<Option<String>, LedgerError> {
    let shared = new_shared();
    let entry = sample_entry()?;
    let previous = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let outcome = panic::catch_unwind(AssertUnwindSafe(|| post_buggy(&shared, entry)));
    panic::set_hook(previous);
    Ok(outcome.err().map(|payload| {
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_else(|| "panic không có thông điệp".to_string())
    }))
}

/// Sổ cái đếm số lần tra cứu số dư. `balance` nhận `&self` nhưng vẫn đếm được nhờ `Cell`.
pub struct MeteredLedger {
    ledger: Ledger,
    lookups: Cell<u32>,
}

impl MeteredLedger {
    pub fn new(ledger: Ledger) -> MeteredLedger {
        MeteredLedger {
            ledger,
            lookups: Cell::new(0),
        }
    }

    pub fn balance(&self, account: &AccountId) -> Result<Money, LedgerError> {
        // `Cell<u32>`: u32 là Copy, nên `get`/`set` sao chép giá trị, không phát tham chiếu.
        self.lookups.set(self.lookups.get() + 1);
        self.ledger.balance(account)
    }

    pub fn lookups(&self) -> u32 {
        self.lookups.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "already borrowed")]
    fn buggy_version_panics_at_runtime() {
        let shared = new_shared();
        let _ = post_buggy(&shared, sample_entry().expect("bút toán cân"));
    }

    #[test]
    fn panic_message_is_captured() -> Result<(), LedgerError> {
        let message = buggy_panic_message()?.expect("phải panic");
        assert!(message.contains("already borrowed"), "{message}");
        Ok(())
    }

    #[test]
    fn fixed_version_posts() -> Result<(), LedgerError> {
        let shared = new_shared();
        let text = post_fixed(&shared, sample_entry()?)?;
        assert_eq!(text, "JE-000001 (trước đó có 0 bút toán)");
        assert_eq!(shared.borrow().len(), 1);
        Ok(())
    }

    #[test]
    fn try_version_reports_busy_instead_of_panicking() -> Result<(), LedgerError> {
        let shared = new_shared();
        let reader = shared.borrow();
        assert_eq!(post_try(&shared, sample_entry()?), Err(LedgerError::Busy));
        drop(reader);
        assert_eq!(post_try(&shared, sample_entry()?)?, "JE-000001");
        Ok(())
    }

    #[test]
    fn cell_counts_through_shared_reference() -> Result<(), LedgerError> {
        let mut ledger = Ledger::new();
        ledger.post(sample_entry()?)?;
        let metered = MeteredLedger::new(ledger);
        let driver = AccountId::new("driver:DR-001")?;
        let first = metered.balance(&driver)?;
        let second = metered.balance(&driver)?;
        assert_eq!(first, Money::from_dong(20_400));
        assert_eq!(first, second);
        assert_eq!(metered.lookups(), 2);
        Ok(())
    }
}
