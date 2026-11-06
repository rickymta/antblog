//! Integration test: chỉ dùng API public của crate `ledger_lab`, như một người dùng thư viện.

use ledger_lab::{
    AccountId, EntryId, JournalEntry, JournalEntryBuilder, Ledger, LedgerError, Money, Posting,
};

fn acct(raw: &str) -> AccountId {
    AccountId::new(raw).expect("mã tài khoản hợp lệ")
}

fn builder_entry(key: &str, lines: &[(&str, i64)]) -> Result<JournalEntry, LedgerError> {
    lines
        .iter()
        .fold(JournalEntryBuilder::new(key), |b, (raw, dong)| {
            b.posting(acct(raw), Money::from_dong(*dong))
        })
        .build()
}

#[test]
fn full_flow_capture_split_reverse_keeps_ledger_balanced() -> Result<(), LedgerError> {
    let mut ledger = Ledger::new();
    ledger.post(builder_entry(
        "trip:TN-0001:capture",
        &[
            ("customer:CU-001", -25_500),
            ("psp:vixanh:clearing", 25_500),
        ],
    )?)?;
    let split = ledger.post(builder_entry(
        "trip:TN-0001:split",
        &[
            ("psp:vixanh:clearing", -25_500),
            ("platform:revenue", 5_100),
            ("driver:DR-001", 20_400),
        ],
    )?)?;
    let split_id = split.id;
    assert_eq!(
        ledger.balance(&acct("driver:DR-001"))?,
        Money::from_dong(20_400)
    );

    let reversal = ledger.reverse(split_id)?;
    assert_eq!(reversal.id.to_string(), "JE-000003");
    assert_eq!(ledger.balance(&acct("driver:DR-001"))?, Money::ZERO);
    assert_eq!(ledger.len(), 3);
    assert_eq!(ledger.grand_total()?, Money::ZERO);
    Ok(())
}

#[test]
fn unbalanced_posting_set_cannot_become_an_entry() {
    let result = JournalEntry::try_new(
        "k",
        vec![
            Posting::new(acct("a"), Money::from_dong(-1)),
            Posting::new(acct("b"), Money::from_dong(2)),
        ],
    );
    assert_eq!(
        result,
        Err(LedgerError::Unbalanced {
            sum: Money::from_dong(1)
        })
    );
}

#[test]
fn reverse_of_unknown_id_is_an_error() {
    let mut ledger = Ledger::new();
    let id = EntryId::parse("JE-000042").expect("mã đúng khuôn dạng");
    assert!(matches!(
        ledger.reverse(id),
        Err(LedgerError::UnknownEntry { .. })
    ));
}

#[test]
fn correction_is_reversal_plus_new_entry_under_a_new_key() -> Result<(), LedgerError> {
    let capture = [
        ("customer:CU-001", -25_500),
        ("psp:vixanh:clearing", 25_500),
    ];
    let wrong = [
        ("psp:vixanh:clearing", -25_500),
        ("platform:revenue", 5_100),
        ("driver:DR-001", 20_400),
    ];
    let right = [
        ("psp:vixanh:clearing", -25_500),
        ("platform:revenue", 5_000),
        ("driver:DR-001", 20_500),
    ];
    let mut ledger = Ledger::new();
    ledger.post(builder_entry("trip:TN-0001:capture", &capture)?)?;
    let wrong_id = ledger
        .post(builder_entry("trip:TN-0001:split", &wrong)?)?
        .id;

    // Cùng key, nội dung khác: bị từ chối, sổ không đổi.
    let same_key = builder_entry("trip:TN-0001:split", &right)?;
    assert!(matches!(
        ledger.post(same_key),
        Err(LedgerError::IdempotencyConflict { .. })
    ));

    // Cách đúng: đảo bút toán sai (JE-000003), rồi ghi bút toán mới dưới key mới (JE-000004).
    assert_eq!(ledger.reverse(wrong_id)?.id.to_string(), "JE-000003");
    let fixed = ledger.post(builder_entry("trip:TN-0001:split-v2", &right)?)?;
    assert_eq!(fixed.id.to_string(), "JE-000004");

    assert_eq!(
        ledger.balance(&acct("platform:revenue"))?,
        Money::from_dong(5_000)
    );
    assert_eq!(
        ledger.balance(&acct("driver:DR-001"))?,
        Money::from_dong(20_500)
    );
    assert_eq!(ledger.balance(&acct("psp:vixanh:clearing"))?, Money::ZERO);
    assert_eq!(ledger.grand_total()?, Money::ZERO);
    Ok(())
}
