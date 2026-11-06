//! Integration test: chỉ dùng API public của crate `ledger_lab`, như một người dùng thư viện.

use ledger_lab::{AccountId, JournalEntry, Ledger, LedgerError, Money, Posting};

fn acct(raw: &str) -> AccountId {
    AccountId::new(raw).expect("mã tài khoản hợp lệ")
}

fn entry(key: &str, lines: &[(&str, i64)]) -> Result<JournalEntry, LedgerError> {
    let postings = lines
        .iter()
        .map(|(raw, dong)| Posting::new(acct(raw), Money::from_dong(*dong)))
        .collect();
    JournalEntry::try_new(key, postings)
}

#[test]
fn topup_then_capture_keeps_ledger_balanced() -> Result<(), LedgerError> {
    let mut ledger = Ledger::new();
    // VíXanh nạp 25.500 vào ví khách, sau đó thu cước chuyến TN-0001.
    ledger.post(entry(
        "psp:vixanh:topup:1",
        &[
            ("psp:vixanh:clearing", -25_500),
            ("customer:CU-001", 25_500),
        ],
    )?)?;
    let posted = ledger.post(entry(
        "trip:TN-0001:capture",
        &[
            ("customer:CU-001", -25_500),
            ("platform:revenue", 5_100),
            ("driver:DR-001", 20_400),
        ],
    )?)?;
    assert_eq!(posted.id, "JE-000002");

    assert_eq!(ledger.balance(&acct("customer:CU-001"))?, Money::ZERO);
    assert_eq!(
        ledger.balance(&acct("driver:DR-001"))?,
        Money::from_dong(20_400)
    );
    assert_eq!(ledger.grand_total()?, Money::ZERO);
    // Chỉ hiện khi chạy `cargo test -- --nocapture`.
    println!("tổng toàn sổ = {}", ledger.grand_total()?);
    Ok(())
}

#[test]
fn replay_does_not_double_count() -> Result<(), LedgerError> {
    let mut ledger = Ledger::new();
    let lines = [("customer:CU-001", -25_500), ("driver:DR-001", 25_500)];
    ledger.post(entry("trip:TN-0001:capture", &lines)?)?;
    let again = ledger.post(entry("trip:TN-0001:capture", &lines)?)?;
    assert!(again.replayed);
    assert_eq!(ledger.len(), 1);
    assert_eq!(
        ledger.balance(&acct("driver:DR-001"))?,
        Money::from_dong(25_500)
    );
    Ok(())
}

#[test]
fn same_key_with_other_amount_is_conflict() -> Result<(), LedgerError> {
    let mut ledger = Ledger::new();
    ledger.post(entry("k", &[("a", -10), ("b", 10)])?)?;
    let result = ledger.post(entry("k", &[("a", -20), ("b", 20)])?);
    assert!(matches!(
        result,
        Err(LedgerError::IdempotencyConflict { .. })
    ));
    assert_eq!(ledger.balance(&acct("b"))?, Money::from_dong(10));
    Ok(())
}

#[test]
fn unbalanced_entry_never_reaches_the_ledger() {
    let result = entry("k", &[("a", -10), ("b", 9)]);
    assert_eq!(
        result,
        Err(LedgerError::Unbalanced {
            sum: Money::from_dong(-1)
        })
    );
}
