//! Ba cách cài sổ cái cho cùng một việc: ghi bút toán và đọc số dư.
//!
//! - [`SlowBook`]: quét toàn sổ, `clone` cả danh sách posting, `format!` để so tài khoản.
//! - [`ScanBook`]: vẫn quét toàn sổ nhưng không cấp phát gì khi quét.
//! - [`IndexedBook`]: giữ số dư cộng dồn, đọc không quét.
//!
//! Cả ba là mô hình đồ chơi của bài đo: không idempotent, không kiểm lại bút toán.

use std::cell::Cell;
use std::collections::HashMap;

use ledger_core::{AccountId, JournalEntry, Money};

/// Bộ đếm không phụ thuộc tải máy: số lần gọi và số posting đã duyệt.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    /// Số lần gọi `balance`.
    pub balance_calls: u64,
    /// Tổng số posting đã duyệt qua trong mọi lần gọi.
    pub scanned_postings: u64,
}

/// Giao diện chung để kịch bản đo chạy trên cả ba cách cài.
pub trait Book {
    /// Ghi một bút toán (đã cân).
    fn post(&mut self, entry: JournalEntry);
    /// Số dư của một tài khoản.
    fn balance(&self, account: &AccountId) -> Money;
    /// Bộ đếm tích lũy từ lúc tạo.
    fn stats(&self) -> Stats;
}

fn add(sum: Money, amount: Money) -> Money {
    sum.checked_add(amount).expect("không tràn i64")
}

// ---------------------------------------------------------------- SlowBook

/// Bản đầu tiên, viết "cho chạy được". Ba chỗ chậm đều nằm trong [`SlowBook::balance`].
#[derive(Debug, Default)]
pub struct SlowBook {
    entries: Vec<JournalEntry>,
    balance_calls: Cell<u64>,
    scanned_postings: Cell<u64>,
}

impl SlowBook {
    /// Sổ rỗng.
    pub fn new() -> SlowBook {
        SlowBook::default()
    }
}

impl Book for SlowBook {
    fn post(&mut self, entry: JournalEntry) {
        self.entries.push(entry);
    }

    fn balance(&self, account: &AccountId) -> Money {
        self.balance_calls.set(self.balance_calls.get() + 1);
        let mut sum = Money::ZERO;
        let mut scanned = 0;
        // Chậm #1: mỗi lần đọc quét TOÀN sổ, độ phức tạp O(số bút toán).
        for entry in &self.entries {
            // Chậm #2: sao chép cả danh sách posting (một Vec và ba String) chỉ để duyệt.
            let postings = entry.postings().to_vec();
            for posting in postings {
                scanned += 1;
                // Chậm #3: dựng hai chuỗi mới chỉ để so hai mã tài khoản.
                if format!("{}", posting.account) == format!("{account}") {
                    sum = add(sum, posting.amount);
                }
            }
        }
        self.scanned_postings
            .set(self.scanned_postings.get() + scanned);
        sum
    }

    fn stats(&self) -> Stats {
        Stats {
            balance_calls: self.balance_calls.get(),
            scanned_postings: self.scanned_postings.get(),
        }
    }
}

// ---------------------------------------------------------------- ScanBook

/// Tổng posting của `account` trong một đoạn bút toán. Duyệt bằng tham chiếu, không cấp phát.
/// Trả thêm số posting đã duyệt.
pub fn sum_slice(entries: &[JournalEntry], account: &AccountId) -> (Money, u64) {
    let mut sum = Money::ZERO;
    let mut scanned = 0;
    for posting in entries.iter().flat_map(JournalEntry::postings) {
        scanned += 1;
        if &posting.account == account {
            sum = add(sum, posting.amount);
        }
    }
    (sum, scanned)
}

/// Sửa lỗi #2 và #3 (hết `clone`, hết `format!`), chưa sửa #1: vẫn quét toàn sổ.
#[derive(Debug, Default)]
pub struct ScanBook {
    entries: Vec<JournalEntry>,
    balance_calls: Cell<u64>,
    scanned_postings: Cell<u64>,
}

impl ScanBook {
    /// Sổ rỗng.
    pub fn new() -> ScanBook {
        ScanBook::default()
    }

    /// Mọi bút toán theo thứ tự ghi.
    pub fn entries(&self) -> &[JournalEntry] {
        &self.entries
    }
}

impl Book for ScanBook {
    fn post(&mut self, entry: JournalEntry) {
        self.entries.push(entry);
    }

    fn balance(&self, account: &AccountId) -> Money {
        let (sum, scanned) = sum_slice(&self.entries, account);
        self.balance_calls.set(self.balance_calls.get() + 1);
        self.scanned_postings
            .set(self.scanned_postings.get() + scanned);
        sum
    }

    fn stats(&self) -> Stats {
        Stats {
            balance_calls: self.balance_calls.get(),
            scanned_postings: self.scanned_postings.get(),
        }
    }
}

// ------------------------------------------------------------- IndexedBook

/// Sửa cả ba: `post` cộng dồn vào bảng số dư, `balance` chỉ tra bảng.
#[derive(Debug, Default)]
pub struct IndexedBook {
    entries: Vec<JournalEntry>,
    balances: HashMap<AccountId, Money>,
    balance_calls: Cell<u64>,
}

impl IndexedBook {
    /// Sổ rỗng.
    pub fn new() -> IndexedBook {
        IndexedBook::default()
    }
}

impl Book for IndexedBook {
    fn post(&mut self, entry: JournalEntry) {
        for posting in entry.postings() {
            // Chỉ `clone` mã tài khoản lần đầu gặp nó; các lần sau tra bằng tham chiếu.
            match self.balances.get_mut(&posting.account) {
                Some(balance) => *balance = add(*balance, posting.amount),
                None => {
                    self.balances
                        .insert(posting.account.clone(), posting.amount);
                }
            }
        }
        self.entries.push(entry);
    }

    fn balance(&self, account: &AccountId) -> Money {
        self.balance_calls.set(self.balance_calls.get() + 1);
        self.balances.get(account).copied().unwrap_or(Money::ZERO)
    }

    fn stats(&self) -> Stats {
        Stats {
            balance_calls: self.balance_calls.get(),
            scanned_postings: 0,
        }
    }
}
