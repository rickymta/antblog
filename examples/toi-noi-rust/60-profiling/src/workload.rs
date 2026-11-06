//! Dữ liệu giả của Tới Nơi và kịch bản đo cố định theo SỐ BÚT TOÁN (không theo thông lượng).

use ledger_core::{AccountId, JournalEntry, Money, Posting};

use crate::books::Book;

/// Số khách và số tài xế xoay vòng; chuyến 1 luôn là `CU-001` / `DR-001`.
pub const CUSTOMERS: u64 = 1_000;
pub const DRIVERS: u64 = 200;

/// Mã tài khoản khách của chuyến thứ `i` (bắt đầu từ 1).
pub fn customer_of(i: u64) -> AccountId {
    AccountId::new(&format!("customer:CU-{:03}", (i - 1) % CUSTOMERS + 1)).expect("mã hợp lệ")
}

/// Mã tài khoản tài xế của chuyến thứ `i`.
pub fn driver_of(i: u64) -> AccountId {
    AccountId::new(&format!("driver:DR-{:03}", (i - 1) % DRIVERS + 1)).expect("mã hợp lệ")
}

/// Bút toán capture chuyến `TN-{i:04}`: khách -25.500, nền tảng +5.100, tài xế +20.400.
pub fn make_entry(i: u64) -> JournalEntry {
    JournalEntry::try_new(
        &format!("trip:TN-{i:04}:capture"),
        vec![
            Posting::new(customer_of(i), Money::from_dong(-25_500)),
            Posting::new(
                AccountId::new("platform:revenue").expect("mã hợp lệ"),
                Money::from_dong(5_100),
            ),
            Posting::new(driver_of(i), Money::from_dong(20_400)),
        ],
    )
    .expect("bút toán cân")
}

/// Dữ liệu một lần đo, dựng sẵn để phần đo không tính chi phí dựng chuỗi và bút toán.
pub struct Workload {
    /// `n` bút toán `TN-0001..TN-n`.
    pub entries: Vec<JournalEntry>,
    /// Với mỗi chuyến: tài khoản khách và tài khoản tài xế sẽ được đọc số dư.
    pub probes: Vec<(AccountId, AccountId)>,
}

/// Dựng dữ liệu cho `n` chuyến.
pub fn make_workload(n: u64) -> Workload {
    Workload {
        entries: (1..=n).map(make_entry).collect(),
        probes: (1..=n).map(|i| (customer_of(i), driver_of(i))).collect(),
    }
}

/// Kết quả một lần chạy kịch bản.
#[derive(Debug, PartialEq, Eq)]
pub struct Outcome {
    /// Tổng mọi số dư đã đọc (để kiểm ba cách cài cho cùng đáp án).
    pub checksum: i64,
    /// Số lần gọi `balance`.
    pub balance_calls: u64,
    /// Số posting đã duyệt qua khi tính số dư.
    pub scanned_postings: u64,
}

/// Kịch bản: mỗi chuyến ghi một bút toán, rồi đọc số dư khách và tài xế của chuyến đó
/// (Admin BFF hiển thị ví ngay sau khi chuyến kết thúc). Tổng cộng `2 * n` lần đọc số dư.
/// Bản thân kịch bản không cấp phát: mọi chuỗi đã dựng sẵn trong [`Workload`].
pub fn post_and_read_balances<B: Book>(book: &mut B, workload: Workload) -> Outcome {
    let mut checksum = 0i64;
    for (entry, (customer, driver)) in workload.entries.into_iter().zip(&workload.probes) {
        book.post(entry);
        checksum += book.balance(customer).dong() + book.balance(driver).dong();
    }
    let stats = book.stats();
    Outcome {
        checksum,
        balance_calls: stats.balance_calls,
        scanned_postings: stats.scanned_postings,
    }
}

/// Số posting phải duyệt của một kịch bản dùng cách quét toàn sổ: mỗi lần đọc thứ `k`
/// (sau khi sổ có `k` bút toán) duyệt `3k` posting, và có hai lần đọc mỗi chuyến.
pub fn expected_scanned(n: u64) -> u64 {
    3 * n * (n + 1)
}
