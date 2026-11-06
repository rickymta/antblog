//! Integration test: chỉ dùng API public của `ledger_core`, như một người dùng thư viện.

use ledger_core::{
    AccountId, JournalEntry, Ledger, LedgerError, Money, PaymentEvent, PaymentState, Posting,
    TripCharge, capture_entry, default_policy, fare_for_km, transition,
};

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

fn trip_0001() -> TripCharge {
    TripCharge {
        trip_id: "TN-0001".to_string(),
        customer: acct("customer:CU-001"),
        driver: acct("driver:DR-001"),
        fare: fare_for_km(5),
    }
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
    let posted = ledger.post(capture_entry(&trip_0001(), &default_policy())?)?;
    assert_eq!(posted.id.to_string(), "JE-000002");

    assert_eq!(ledger.balance(&acct("customer:CU-001"))?, Money::ZERO);
    assert_eq!(
        ledger.balance(&acct("platform:revenue"))?,
        Money::from_dong(5_100)
    );
    assert_eq!(
        ledger.balance(&acct("driver:DR-001"))?,
        Money::from_dong(20_400)
    );
    assert_eq!(ledger.grand_total()?, Money::ZERO);
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

#[test]
fn reversal_restores_balances_and_keeps_history() -> Result<(), LedgerError> {
    let mut ledger = Ledger::new();
    let id = ledger
        .post(capture_entry(&trip_0001(), &default_policy())?)?
        .id;
    ledger.reverse(id)?;
    assert_eq!(ledger.len(), 2);
    assert_eq!(ledger.balance(&acct("driver:DR-001"))?, Money::ZERO);
    // Lịch sử còn nguyên: bút toán gốc vẫn đọc được qua `entries()`.
    assert_eq!(
        ledger.entries()[0].postings()[0].amount,
        Money::from_dong(-25_500)
    );
    Ok(())
}

#[test]
fn ledger_survives_a_json_roundtrip_entry_by_entry() -> Result<(), Box<dyn std::error::Error>> {
    let mut ledger = Ledger::new();
    let id = ledger
        .post(capture_entry(&trip_0001(), &default_policy())?)?
        .id;
    ledger.reverse(id)?;
    let json = serde_json::to_string(ledger.entries())?;
    let entries: Vec<JournalEntry> = serde_json::from_str(&json)?;

    let mut rebuilt = Ledger::new();
    for e in entries {
        rebuilt.post(e)?;
    }
    assert_eq!(rebuilt.balances()?, ledger.balances()?);
    assert_eq!(rebuilt.len(), 2);
    Ok(())
}

#[test]
fn payment_state_rejects_skipping_authorization() {
    assert!(transition(PaymentState::Initiated, PaymentEvent::Capture).is_err());
    assert_eq!(
        transition(PaymentState::Initiated, PaymentEvent::Authorize),
        Ok(PaymentState::Authorized)
    );
}
