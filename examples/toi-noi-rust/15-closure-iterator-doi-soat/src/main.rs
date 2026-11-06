//! Đối soát sổ cái Tới Nơi với sao kê VíXanh giả, hoàn toàn trong bộ nhớ.

use std::error::Error;
use std::process::ExitCode;

use ledger_lab::reconcile::{
    CLEARING, KEY_PREFIX, LazyProbe, Summary, at_least, lazy_probe, ledger_records, ledger_totals,
    positional_ref_clashes, psp_totals, reconcile, visit_common,
};
use ledger_lab::statement::parse_statement;
use ledger_lab::{
    AccountId, Finding, JournalEntry, Ledger, LedgerError, LedgerRecord, Money, Posting,
    StatementError,
};

/// Sao kê VíXanh giả: VX-1002 chia hai dòng, VX-1003 lệch 500 đồng, VX-1004 không có,
/// VX-1005 sổ ta chưa ghi.
const STATEMENT: &str = "\
external_ref,amount_vnd
VX-1001,25500
VX-1002,25000
VX-1002,15000
VX-1003,31000
VX-1005,52000
---
tổng: 5 dòng
";

/// Bút toán thu qua VíXanh: ví khách -X, clearing +X (dấu như bước capture của bài 13).
/// Key `psp:vixanh:<external_ref>` là quy ước riêng của lab đối soát này.
fn psp_entry(external_ref: &str, customer: &str, dong: i64) -> Result<JournalEntry, LedgerError> {
    JournalEntry::try_new(
        &format!("{KEY_PREFIX}{external_ref}"),
        vec![
            Posting::new(AccountId::new(customer)?, Money::from_dong(-dong)),
            Posting::new(AccountId::new(CLEARING)?, Money::from_dong(dong)),
        ],
    )
}

/// Sổ cái mẫu: 4 lần thu qua VíXanh và 1 bút toán chia tiền nội bộ (bước 2 của bài 13).
fn build_ledger() -> Result<Ledger, LedgerError> {
    let mut ledger = Ledger::new();
    ledger.post(psp_entry("VX-1001", "customer:CU-001", 25_500)?)?; // JE-000001
    ledger.post(JournalEntry::try_new(
        "trip:TN-0001:split", // JE-000002, bị `filter_map` bỏ qua
        vec![
            Posting::new(AccountId::new(CLEARING)?, Money::from_dong(-25_500)),
            Posting::new(AccountId::new("platform:revenue")?, Money::from_dong(5_100)),
            Posting::new(AccountId::new("driver:DR-001")?, Money::from_dong(20_400)),
        ],
    )?)?;
    ledger.post(psp_entry("VX-1002", "customer:CU-002", 40_000)?)?; // JE-000003
    ledger.post(psp_entry("VX-1003", "customer:CU-003", 31_500)?)?; // JE-000004
    ledger.post(psp_entry("VX-1004", "customer:CU-004", 18_000)?)?; // JE-000005
    Ok(ledger)
}

/// Ba loại closure: `Fn`, `FnMut`, `FnOnce`.
fn closure_kinds(records: &[LedgerRecord]) {
    // Fn: chỉ đọc biến bắt được (`min`), gọi bao nhiêu lần cũng được.
    let min = Money::from_dong(30_000);
    let is_big = |r: &LedgerRecord| r.amount >= min;
    println!(
        "Fn      : {} dòng sổ từ {min} trở lên",
        records.iter().filter(|r| is_big(r)).count()
    );

    // FnMut: sửa biến bắt được (`mismatched`) mỗi lần gọi.
    let ours = ledger_totals(records).expect("không tràn");
    let theirs = match parse_statement(STATEMENT) {
        Ok(psp) => psp_totals(&psp).expect("không tràn"),
        Err(e) => panic!("sao kê mẫu hợp lệ: {e}"),
    };
    let mut mismatched = 0;
    visit_common(&ours, &theirs, |_, l, p| {
        if l != p {
            mismatched += 1;
        }
    });
    println!("FnMut   : {mismatched} ref có ở cả hai phía nhưng lệch số tiền");

    // FnOnce: closure `move` trả chính `note` đi, nên chỉ gọi được một lần.
    let note = String::from("đã đối soát với sao kê VíXanh");
    let finish = move || note;
    println!("FnOnce  : {}", finish());
    // Lỗi E0382 nếu bỏ comment dòng dưới: `finish` đã bị gọi (và tiêu thụ) ở trên.
    // let again = finish();
}

/// Chứng minh lazy: in thứ tự thật của `inspect` khi gọi `next()`.
fn lazy_demo(records: &[LedgerRecord]) {
    let big = at_least(Money::from_dong(30_000));
    println!("Dựng chuỗi inspect → filter → map (chưa tiêu thụ):");
    let mut iter = records
        .iter()
        .inspect(|r| println!("    inspect {}", r.external_ref))
        .filter(|r| big(r))
        .map(|r| r.external_ref.as_str());
    println!("  dựng xong, chưa có dòng inspect nào ở trên");
    for call in 1..=3 {
        println!("  next() lần {call}:");
        println!("    -> {:?}", iter.next());
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let ledger = build_ledger()?;
    let records = ledger_records(&ledger)?;
    println!(
        "Phía sổ cái ({} bút toán, {} dòng qua VíXanh):",
        ledger.len(),
        records.len()
    );
    for r in &records {
        println!("  {} {:<8}{:>12}", r.entry_id, r.external_ref, r.amount);
    }

    let psp = parse_statement(STATEMENT)?;
    println!("Phía sao kê VíXanh ({} dòng):", psp.len());
    for r in &psp {
        println!("  {:<8}{:>12}", r.external_ref, r.amount);
    }

    let report = reconcile(&records, &psp)?;
    println!(
        "Kết quả đối soát: {} ref khớp, {} chênh lệch",
        report.matched,
        report.findings.len()
    );
    for (external_ref, finding) in &report.findings {
        println!("  {external_ref}: {finding}");
    }
    let Summary {
        missing_in_psp,
        missing_in_ledger,
        amount_mismatch,
    } = report.summary();
    println!(
        "Tóm tắt: thiếu bên PSP = {missing_in_psp}, thiếu bên ta = {missing_in_ledger}, lệch số tiền = {amount_mismatch}"
    );

    println!("--- Closure ---");
    closure_kinds(&records);

    println!("--- zip ghép theo vị trí (cách sai) ---");
    for (ours, theirs) in positional_ref_clashes(&records, &psp) {
        println!("  ghép {ours} với {theirs}");
    }

    println!("--- Lazy ---");
    lazy_demo(&records);
    let probe = lazy_probe(&records, Money::from_dong(30_000));
    println!(
        "inspect chạy: dựng = {}, next đầu tiên = {}, collect = {}",
        probe.after_build, probe.after_first, probe.after_collect
    );

    println!("--- Sao kê hỏng ---");
    match parse_statement("external_ref,amount_vnd\nVX-1,100\nVX-2,12a\n") {
        Ok(_) => println!("  không có lỗi?"),
        Err(e) => println!("  {e}"),
    }
    Ok(())
}

/// Kiểm các giá trị kỳ vọng in trong bài; trả danh sách lỗi (rỗng nếu đúng).
fn verify() -> Vec<String> {
    let mut failures = Vec::new();
    let mut check = |ok: bool, name: &str| {
        if !ok {
            failures.push(name.to_string());
        }
    };

    let Ok(ledger) = build_ledger() else {
        return vec!["dựng sổ cái mẫu".to_string()];
    };
    check(ledger.len() == 5, "sổ có 5 bút toán");
    let Ok(records) = ledger_records(&ledger) else {
        return vec!["lấy dòng qua VíXanh".to_string()];
    };
    check(
        records.len() == 4,
        "4 dòng qua VíXanh (bỏ bút toán chia tiền)",
    );
    check(
        records.first().map(|r| r.entry_id.as_str()) == Some("JE-000001"),
        "dòng đầu là JE-000001",
    );
    check(
        records.get(1).map(|r| r.entry_id.as_str()) == Some("JE-000003"),
        "dòng thứ hai là JE-000003 (JE-000002 bị bỏ)",
    );

    let Ok(psp) = parse_statement(STATEMENT) else {
        return vec!["đọc sao kê mẫu".to_string()];
    };
    check(psp.len() == 5, "sao kê 5 dòng, chân trang bị bỏ");

    let Ok(report) = reconcile(&records, &psp) else {
        return vec!["đối soát".to_string()];
    };
    check(report.matched == 2, "2 ref khớp (VX-1001, VX-1002)");
    check(
        report.findings.get("VX-1003")
            == Some(&Finding::AmountMismatch {
                ledger: Money::from_dong(31_500),
                psp: Money::from_dong(31_000),
            }),
        "VX-1003 lệch 500",
    );
    check(
        report.findings.get("VX-1004")
            == Some(&Finding::MissingInPsp {
                ledger: Money::from_dong(18_000),
            }),
        "VX-1004 thiếu bên PSP",
    );
    check(
        report.findings.get("VX-1005")
            == Some(&Finding::MissingInLedger {
                psp: Money::from_dong(52_000),
            }),
        "VX-1005 thiếu bên ta",
    );
    check(
        report
            .findings
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>()
            == ["VX-1003", "VX-1004", "VX-1005"],
        "chênh lệch sắp theo external_ref",
    );
    check(
        report.summary()
            == Summary {
                missing_in_psp: 1,
                missing_in_ledger: 1,
                amount_mismatch: 1,
            },
        "tóm tắt 1/1/1",
    );

    let clashes = positional_ref_clashes(&records, &psp);
    check(
        clashes == [("VX-1003", "VX-1002"), ("VX-1004", "VX-1003")],
        "zip ghép sai hai cặp",
    );

    check(
        lazy_probe(&records, Money::from_dong(30_000))
            == LazyProbe {
                after_build: 0,
                after_first: 2,
                after_collect: 4,
            },
        "lazy: 0 / 2 / 4 lần inspect",
    );

    check(
        parse_statement("h\nVX-1,100\nVX-2,12a\n")
            == Err(StatementError::BadAmount {
                line: 3,
                raw: "12a".to_string(),
            }),
        "sao kê hỏng báo dòng 3",
    );
    failures
}

fn main() -> ExitCode {
    if std::env::args().any(|arg| arg == "--verify") {
        let failures = verify();
        if failures.is_empty() {
            println!("VERIFY OK");
            return ExitCode::SUCCESS;
        }
        for f in &failures {
            eprintln!("VERIFY FAIL: {f}");
        }
        return ExitCode::FAILURE;
    }
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Lỗi: {e}");
            ExitCode::FAILURE
        }
    }
}
