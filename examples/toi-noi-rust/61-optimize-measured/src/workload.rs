//! Dữ liệu giả của Tới Nơi: bút toán capture chuyến `TN-{i:04}` và mã tài khoản "từ bên ngoài".

use ledger_core::{AccountId, JournalEntry, Money, Posting};

/// Số khách và số tài xế xoay vòng (chuyến 1 luôn là `CU-001` / `DR-001`).
pub const CUSTOMERS: u64 = 1_000;
/// Số tài xế xoay vòng.
pub const DRIVERS: u64 = 200;

fn account(raw: &str) -> AccountId {
    AccountId::new(raw).expect("mã tài khoản hợp lệ")
}

/// Bút toán capture chuyến `i` (từ 1): khách -25.500, nền tảng +5.100, tài xế +20.400.
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

/// Các bút toán `from..from + count`.
pub fn make_entries(from: u64, count: u64) -> Vec<JournalEntry> {
    (from..from + count).map(make_entry).collect()
}

/// Mọi tài khoản mà `make_entry` có thể dùng, để seed bảng `accounts` (kèm `kind`).
pub fn standard_accounts() -> Vec<(String, &'static str)> {
    let mut all = vec![("platform:revenue".to_string(), "revenue")];
    all.extend((1..=CUSTOMERS).map(|n| (format!("customer:CU-{n:03}"), "liability")));
    all.extend((1..=DRIVERS).map(|n| (format!("driver:DR-{n:03}"), "liability")));
    all
}

/// `n` mã tài khoản như nhận từ bên ngoài: 90% đã chuẩn, 5% chỉ thừa khoảng trắng,
/// 5% thừa khoảng trắng và loại tài khoản viết hoa chữ đầu. Xác định theo chỉ số.
pub fn raw_account_ids(n: usize) -> Vec<String> {
    (0..n)
        .map(|i| {
            let base = match i % 3 {
                0 => format!("customer:CU-{:03}", i as u64 % CUSTOMERS + 1),
                1 => format!("driver:DR-{:03}", i as u64 % DRIVERS + 1),
                _ => "platform:revenue".to_string(),
            };
            match i % 20 {
                0 => format!("  {}{}  ", base[..1].to_uppercase(), &base[1..]),
                10 => format!(" {base}"),
                _ => base,
            }
        })
        .collect()
}
