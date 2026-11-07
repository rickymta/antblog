//! Drop tất định và RAII: tài nguyên được giải phóng đúng chỗ, đúng thứ tự, kể cả khi panic.

use std::cell::RefCell;
use std::mem;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Mutex;

/// Nhật ký chung: mỗi `Drop` ghi một dòng để ta quan sát thứ tự.
#[derive(Default)]
pub struct Log(RefCell<Vec<String>>);

impl Log {
    /// Ghi một dòng.
    pub fn push(&self, line: impl Into<String>) {
        self.0.borrow_mut().push(line.into());
    }

    /// Lấy toàn bộ dòng đã ghi, xóa nhật ký.
    pub fn take(&self) -> Vec<String> {
        mem::take(&mut *self.0.borrow_mut())
    }
}

/// Tài nguyên giả (kết nối, khóa...) báo `drop <tên>` khi bị giải phóng.
pub struct Guard<'a> {
    name: &'static str,
    log: &'a Log,
}

impl<'a> Guard<'a> {
    /// Tạo tài nguyên và ghi `open <tên>`.
    pub fn new(log: &'a Log, name: &'static str) -> Guard<'a> {
        log.push(format!("open {name}"));
        Guard { name, log }
    }
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        self.log.push(format!("drop {}", self.name));
    }
}

/// Struct chứa hai tài nguyên: field bị drop theo thứ tự KHAI BÁO (`first` rồi `second`).
pub struct Pair<'a> {
    pub first: Guard<'a>,
    pub second: Guard<'a>,
}

/// Biến cục bộ bị drop theo thứ tự NGƯỢC khai báo; `drop(x)` giải phóng sớm; `mem::forget` thì không drop.
pub fn drop_order() -> Vec<String> {
    let log = Log::default();
    {
        let _conn = Guard::new(&log, "conn");
        let _lock = Guard::new(&log, "lock");
        let early = Guard::new(&log, "early");
        drop(early);
        let _pair = Pair {
            first: Guard::new(&log, "pair.first"),
            second: Guard::new(&log, "pair.second"),
        };
        let leaked = Guard::new(&log, "leaked");
        mem::forget(leaked); // an toàn, nhưng rò rỉ: Drop không bao giờ chạy
        log.push("end of scope");
    }
    log.take()
}

/// Giao dịch RAII: không `commit` thì lúc drop tự `rollback`.
pub struct TxGuard<'a> {
    log: &'a Log,
    committed: bool,
}

impl<'a> TxGuard<'a> {
    /// Mở giao dịch.
    pub fn begin(log: &'a Log) -> TxGuard<'a> {
        log.push("tx begin");
        TxGuard {
            log,
            committed: false,
        }
    }

    /// Chốt giao dịch. Nhận `self` theo giá trị: không dùng lại được sau khi commit.
    pub fn commit(mut self) {
        self.committed = true;
        self.log.push("tx commit");
    }
}

impl Drop for TxGuard<'_> {
    fn drop(&mut self) {
        if !self.committed {
            self.log.push("tx rollback");
        }
    }
}

/// Giao dịch thành công thì commit; trả lỗi sớm hoặc panic thì rollback, không cần `finally`.
pub fn transaction_outcomes() -> Vec<String> {
    let log = Log::default();

    // 1. Thành công.
    TxGuard::begin(&log).commit();

    // 2. Thoát sớm bằng `?`-kiểu: guard bị drop khi hàm trả về.
    let early_return = |fail: bool| -> Result<(), &'static str> {
        let tx = TxGuard::begin(&log);
        if fail {
            return Err("hết hạn mức");
        }
        tx.commit();
        Ok(())
    };
    let _ = early_return(true);

    // 3. Panic: stack được tháo (unwind), `Drop` của mọi biến cục bộ vẫn chạy.
    // Tắt hook mặc định để demo không in thông điệp panic ra stderr.
    let previous = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let _tx = TxGuard::begin(&log);
        panic!("lỗi lập trình giữa chừng");
    }));
    panic::set_hook(previous);
    log.push(format!("panic caught: {}", result.is_err()));
    log.take()
}

/// Khóa là RAII: `MutexGuard` nhả khóa khi ra khỏi phạm vi. Trả về
/// (`try_lock` lúc đang giữ khóa có lỗi không, `try_lock` sau khi nhả có được không).
pub fn mutex_guard_scope() -> (bool, bool) {
    let ledger_lock = Mutex::new(0_u64);
    let held_blocks_second = {
        let mut guard = ledger_lock.lock().expect("không bị poison");
        *guard += 1;
        ledger_lock.try_lock().is_err()
    }; // guard drop ở đây: khóa được nhả
    let free_after_scope = ledger_lock.try_lock().is_ok();
    (held_blocks_second, free_after_scope)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locals_drop_in_reverse_and_fields_in_order() {
        let order = drop_order();
        let drops: Vec<_> = order.iter().filter(|l| l.starts_with("drop")).collect();
        assert_eq!(
            drops,
            [
                "drop early",
                "drop pair.first",
                "drop pair.second",
                "drop lock",
                "drop conn"
            ]
        );
        // `mem::forget` bỏ qua Drop: không có "drop leaked".
        assert!(!order.contains(&"drop leaked".to_string()));
    }

    #[test]
    fn commit_consumes_the_guard_so_no_rollback_follows() {
        let log = Log::default();
        TxGuard::begin(&log).commit();
        assert_eq!(log.take(), ["tx begin", "tx commit"]);
    }

    #[test]
    fn panic_still_rolls_back() {
        assert_eq!(transaction_outcomes().last().unwrap(), "panic caught: true");
        assert_eq!(
            transaction_outcomes()
                .iter()
                .filter(|l| *l == "tx rollback")
                .count(),
            2
        );
    }

    #[test]
    fn mutex_guard_releases_at_scope_end() {
        assert_eq!(mutex_guard_scope(), (true, true));
    }
}
