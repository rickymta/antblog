//! Sinh dữ liệu giả của Tới Nơi: bút toán capture cho chuyến thứ `i` (bắt đầu từ 1).

use ledger_core::{AccountId, JournalEntry, Ledger, Money, Posting};

/// Số khách và số tài xế xoay vòng; chuyến 1 luôn là `CU-001` / `DR-001`.
pub const CUSTOMERS: u64 = 1_000;
pub const DRIVERS: u64 = 200;

fn account(raw: &str) -> AccountId {
    AccountId::new(raw).expect("mã tài khoản hợp lệ")
}

/// Bút toán capture chuyến `TN-{i:04}`: khách -25.500, nền tảng +5.100, tài xế +20.400.
pub fn make_entry(i: u64) -> JournalEntry {
    let customer = format!("customer:CU-{:03}", (i - 1) % CUSTOMERS + 1);
    let driver = format!("driver:DR-{:03}", (i - 1) % DRIVERS + 1);
    JournalEntry::try_new(
        &format!("trip:TN-{i:04}:capture"),
        vec![
            Posting::new(account(&customer), Money::from_dong(-25_500)),
            Posting::new(account("platform:revenue"), Money::from_dong(5_100)),
            Posting::new(account(&driver), Money::from_dong(20_400)),
        ],
    )
    .expect("bút toán cân")
}

/// Sổ cái có `n` bút toán `TN-0001..TN-n`.
pub fn make_ledger(n: u64) -> Ledger {
    let mut ledger = Ledger::new();
    for i in 1..=n {
        ledger.post(make_entry(i)).expect("key không trùng");
    }
    ledger
}

/// Mọi posting của sổ, thu bằng `Vec::new()` rồi `push`: vector tự tăng và cấp phát lại nhiều lần.
pub fn flatten_grow(ledger: &Ledger) -> Vec<&Posting> {
    let mut all = Vec::new();
    for entry in ledger.entries() {
        for posting in entry.postings() {
            all.push(posting);
        }
    }
    all
}

/// Cùng kết quả, nhưng đếm trước để cấp phát một lần. Việc đếm là một lượt duyệt thêm qua sổ.
pub fn flatten_with_capacity(ledger: &Ledger) -> Vec<&Posting> {
    let total: usize = ledger.entries().iter().map(|e| e.postings().len()).sum();
    flatten_with_known_capacity(ledger, total)
}

/// Cấp phát một lần với dung lượng người gọi đã biết (không tốn lượt đếm).
pub fn flatten_with_known_capacity(ledger: &Ledger, capacity: usize) -> Vec<&Posting> {
    let mut all = Vec::with_capacity(capacity);
    for entry in ledger.entries() {
        for posting in entry.postings() {
            all.push(posting);
        }
    }
    all
}
