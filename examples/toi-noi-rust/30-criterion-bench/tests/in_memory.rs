//! Kiểm đúng nghiệp vụ của workload dùng cho benchmark (không cần PostgreSQL).

use ledger_bench::workload::{flatten_grow, flatten_with_capacity};
use ledger_bench::{IndexedLedger, make_entry, make_ledger};
use ledger_core::{AccountId, Money};

fn acct(raw: &str) -> AccountId {
    AccountId::new(raw).unwrap()
}

#[test]
fn trip_one_matches_fixed_numbers() {
    let entry = make_entry(1);
    assert_eq!(entry.idempotency_key(), "trip:TN-0001:capture");
    let postings: Vec<(&str, i64)> = entry
        .postings()
        .iter()
        .map(|p| (p.account.as_str(), p.amount.dong()))
        .collect();
    assert_eq!(
        postings,
        [
            ("customer:CU-001", -25_500),
            ("platform:revenue", 5_100),
            ("driver:DR-001", 20_400),
        ]
    );
}

#[test]
fn running_balance_equals_full_scan() {
    let mut book = IndexedLedger::new();
    for i in 1..=2_000 {
        assert!(book.post(make_entry(i)).unwrap());
    }
    for account in [
        "driver:DR-001",
        "customer:CU-001",
        "platform:revenue",
        "driver:DR-200",
    ] {
        let a = acct(account);
        assert_eq!(
            book.balance(&a),
            book.ledger().balance(&a).unwrap(),
            "{account}"
        );
    }
    assert_eq!(
        book.balance(&acct("platform:revenue")),
        Money::from_dong(2_000 * 5_100)
    );
    assert_eq!(book.ledger().grand_total().unwrap(), Money::ZERO);
}

#[test]
fn replay_leaves_running_balance_unchanged() {
    let mut book = IndexedLedger::new();
    assert!(book.post(make_entry(1)).unwrap());
    assert!(!book.post(make_entry(1)).unwrap());
    assert_eq!(
        book.balance(&acct("driver:DR-001")),
        Money::from_dong(20_400)
    );
    assert_eq!(book.ledger().len(), 1);
}

#[test]
fn unknown_account_has_zero_balance() {
    let book = IndexedLedger::new();
    assert_eq!(book.balance(&acct("driver:DR-404")), Money::ZERO);
}

#[test]
fn both_flatten_variants_give_same_postings() {
    let ledger = make_ledger(500);
    let (grow, sized) = (flatten_grow(&ledger), flatten_with_capacity(&ledger));
    assert_eq!(grow.len(), 1_500);
    assert_eq!(grow, sized);
    // with_capacity đặt đúng dung lượng cần, không dư.
    assert_eq!(sized.capacity(), 1_500);
}
