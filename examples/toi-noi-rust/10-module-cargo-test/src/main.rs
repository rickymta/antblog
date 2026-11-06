//! Binary crate: dùng thư viện qua tên crate `ledger_lab`, không `mod` lại các file kia.

use std::process::ExitCode;

use ledger_lab::{AccountId, JournalEntry, Ledger, LedgerError, Money, Posting};

/// Bút toán thu cước chuyến TN-0001: khách -25.500, nền tảng +5.100, tài xế +20.400.
fn capture_trip_entry() -> Result<JournalEntry, LedgerError> {
    JournalEntry::try_new(
        "trip:TN-0001:capture",
        vec![
            Posting::new(
                AccountId::new("customer:CU-001")?,
                Money::from_dong(-25_500),
            ),
            Posting::new(AccountId::new("platform:revenue")?, Money::from_dong(5_100)),
            Posting::new(AccountId::new("driver:DR-001")?, Money::from_dong(20_400)),
        ],
    )
}

const ACCOUNTS: [&str; 3] = ["customer:CU-001", "platform:revenue", "driver:DR-001"];

fn run() -> Result<(), LedgerError> {
    let mut ledger = Ledger::new();
    if ledger.is_empty() {
        println!("Sổ cái mới, chưa có bút toán.");
    }

    let posted = ledger.post(capture_trip_entry()?)?;
    println!(
        "{} {} ({} posting, replayed = {})",
        posted.id,
        posted.entry.idempotency_key(),
        posted.entry.postings().len(),
        posted.replayed
    );

    // Ghi lần hai cùng key: không tạo bản sao.
    let again = ledger.post(capture_trip_entry()?)?;
    println!("{} ghi lại: replayed = {}", again.id, again.replayed);
    println!("Số bút toán trong sổ: {}", ledger.len());

    println!("Số dư:");
    for raw in ACCOUNTS {
        let balance = ledger.balance(&AccountId::new(raw)?)?;
        println!("  {raw:<18}{balance:>12}");
    }
    println!("Tổng toàn sổ: {}", ledger.grand_total()?);
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

    let mut ledger = Ledger::new();
    let entry = capture_trip_entry().expect("bút toán mẫu cân");
    let posted = ledger.post(entry).expect("ghi lần đầu");
    check(posted.id == "JE-000001", "mã bút toán JE-000001");
    check(!posted.replayed, "lần đầu không phải replay");

    let again = ledger
        .post(capture_trip_entry().expect("bút toán mẫu cân"))
        .expect("ghi lại cùng key");
    check(again.replayed, "lần hai là replay");
    check(ledger.len() == 1, "chỉ một bút toán sau khi ghi lại");

    let balance_of = |raw: &str| {
        ledger
            .balance(&AccountId::new(raw).expect("mã hợp lệ"))
            .expect("không tràn")
    };
    check(
        balance_of("customer:CU-001") == Money::from_dong(-25_500),
        "khách -25.500",
    );
    check(
        balance_of("platform:revenue") == Money::from_dong(5_100),
        "nền tảng 5.100",
    );
    check(
        balance_of("driver:DR-001") == Money::from_dong(20_400),
        "tài xế 20.400",
    );
    check(
        ledger.grand_total() == Ok(Money::ZERO),
        "tổng toàn sổ bằng 0",
    );

    let a = AccountId::new("a").expect("mã hợp lệ");
    let b = AccountId::new("b").expect("mã hợp lệ");
    let unbalanced = JournalEntry::try_new(
        "k1",
        vec![
            Posting::new(a.clone(), Money::from_dong(-100)),
            Posting::new(b.clone(), Money::from_dong(90)),
        ],
    );
    check(
        unbalanced
            == Err(LedgerError::Unbalanced {
                sum: Money::from_dong(-10),
            }),
        "bút toán lệch 10 đồng bị từ chối",
    );
    let overflow = JournalEntry::try_new(
        "k2",
        vec![
            Posting::new(a, Money::from_dong(i64::MAX)),
            Posting::new(b, Money::from_dong(1)),
        ],
    );
    check(
        overflow == Err(LedgerError::Overflow),
        "tràn i64 bị báo Overflow",
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
