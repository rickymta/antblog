//! Khối lượng việc dùng chung: ghi `n` bút toán capture vào `ledger-core`.
//! `make_entry` giống lab 30 (3 posting mỗi bút toán) để số đo so được với bài 30.

use ledger_core::{AccountId, JournalEntry, Ledger, Money, Posting};

const CUSTOMERS: u64 = 1_000;
const DRIVERS: u64 = 200;

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
pub fn build_ledger(n: u64) -> Ledger {
    let mut ledger = Ledger::new();
    for i in 1..=n {
        ledger.post(make_entry(i)).expect("key không trùng");
    }
    ledger
}

/// Ba số dùng để kiểm kết quả nghiệp vụ không đổi khi đổi allocator:
/// (số dư `driver:DR-001`, số dư `platform:revenue`, tổng toàn sổ), đơn vị đồng.
pub fn summarize(ledger: &Ledger) -> (i64, i64, i64) {
    let dong = |raw: &str| ledger.balance(&account(raw)).expect("không tràn").dong();
    (
        dong("driver:DR-001"),
        dong("platform:revenue"),
        ledger.grand_total().expect("không tràn").dong(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_entry_is_the_standard_trip() {
        let entry = make_entry(1);
        let amounts: Vec<i64> = entry.postings().iter().map(|p| p.amount.dong()).collect();
        assert_eq!(amounts, [-25_500, 5_100, 20_400]);
        assert_eq!(entry.idempotency_key(), "trip:TN-0001:capture");
    }

    #[test]
    fn ten_thousand_entries_balance() {
        assert_eq!(summarize(&build_ledger(10_000)), (1_020_000, 51_000_000, 0));
    }
}
