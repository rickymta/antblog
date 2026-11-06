//! Integration test: chỉ dùng API public, đi từ sao kê dạng văn bản tới báo cáo.

use ledger_lab::reconcile::{ledger_records, reconcile};
use ledger_lab::statement::parse_statement;
use ledger_lab::{AccountId, Finding, JournalEntry, Ledger, LedgerError, Money, Posting};

fn collect_via_psp(ledger: &mut Ledger, external_ref: &str, dong: i64) -> Result<(), LedgerError> {
    let entry = JournalEntry::try_new(
        &format!("psp:vixanh:{external_ref}"),
        vec![
            Posting::new(AccountId::new("customer:CU-001")?, Money::from_dong(-dong)),
            Posting::new(
                AccountId::new("psp:vixanh:clearing")?,
                Money::from_dong(dong),
            ),
        ],
    )?;
    ledger.post(entry)?;
    Ok(())
}

#[test]
fn perfect_match_has_no_findings() -> Result<(), Box<dyn std::error::Error>> {
    let mut ledger = Ledger::new();
    collect_via_psp(&mut ledger, "VX-1", 25_500)?;
    collect_via_psp(&mut ledger, "VX-2", 40_000)?;
    let psp = parse_statement("h\nVX-2,40000\nVX-1,25500\n")?;
    // Thứ tự dòng hai phía khác nhau vẫn khớp, vì khớp theo khóa chứ không theo vị trí.
    let report = reconcile(&ledger_records(&ledger)?, &psp)?;
    assert_eq!(report.matched, 2);
    assert!(report.findings.is_empty());
    Ok(())
}

#[test]
fn replayed_ledger_entry_is_not_double_counted() -> Result<(), Box<dyn std::error::Error>> {
    let mut ledger = Ledger::new();
    collect_via_psp(&mut ledger, "VX-1", 25_500)?;
    collect_via_psp(&mut ledger, "VX-1", 25_500)?; // ghi lại cùng key: replay, không thêm dòng
    let psp = parse_statement("h\nVX-1,25500\n")?;
    let report = reconcile(&ledger_records(&ledger)?, &psp)?;
    assert_eq!(report.matched, 1);
    Ok(())
}

#[test]
fn missing_on_our_side_is_reported() -> Result<(), Box<dyn std::error::Error>> {
    let ledger = Ledger::new();
    let psp = parse_statement("h\nVX-9,1000\n")?;
    let report = reconcile(&ledger_records(&ledger)?, &psp)?;
    assert_eq!(
        report.findings["VX-9"],
        Finding::MissingInLedger {
            psp: Money::from_dong(1_000)
        }
    );
    Ok(())
}

#[test]
fn collect_into_hashmap_keeps_last_duplicate_key() {
    use std::collections::HashMap;
    // Khác `ToDictionary` của LINQ (ném ArgumentException khi trùng khóa),
    // `collect` vào HashMap âm thầm giữ giá trị cuối: phải group trước khi collect.
    let lines = [("VX-1", 25_000), ("VX-1", 15_000)];
    let map: HashMap<&str, i64> = lines.into_iter().collect();
    assert_eq!(map.len(), 1);
    assert_eq!(map["VX-1"], 15_000);
}
