//! Binary crate: dòng tiền hai bước của chuyến TN-0001, bút toán đảo và số dư projection.

use std::process::ExitCode;

use ledger_lab::{
    AccountId, Flow, JournalEntry, JournalEntryBuilder, Ledger, LedgerError, Money, kind_of, naive,
};

const CUSTOMER: &str = "customer:CU-001";
const CLEARING: &str = "psp:vixanh:clearing";
const REVENUE: &str = "platform:revenue";
const DRIVER: &str = "driver:DR-001";

fn acct(raw: &str) -> Result<AccountId, LedgerError> {
    AccountId::new(raw)
}

/// Bước 1, capture: khách trả 25.500, tiền nằm ở tài khoản clearing của VíXanh.
fn capture_entry() -> Result<JournalEntry, LedgerError> {
    JournalEntryBuilder::new("trip:TN-0001:capture")
        .posting(acct(CUSTOMER)?, Money::from_dong(-25_500))
        .posting(acct(CLEARING)?, Money::from_dong(25_500))
        .build()
}

/// Bước 2, chia: clearing trả ra 25.500, nền tảng nhận 5.100 và tài xế nhận 20.400.
fn split_entry() -> Result<JournalEntry, LedgerError> {
    JournalEntryBuilder::new("trip:TN-0001:split")
        .posting(acct(CLEARING)?, Money::from_dong(-25_500))
        .posting(acct(REVENUE)?, Money::from_dong(5_100))
        .posting(acct(DRIVER)?, Money::from_dong(20_400))
        .build()
}

/// In một bút toán: dòng tiền vào (số dương) trước, dòng tiền ra (số âm) sau.
fn print_entry(index: usize, entry: &JournalEntry) {
    let id = format!("JE-{:06}", index + 1);
    match entry.reverses() {
        Some(orig) => println!("{id}  {}  (đảo {orig})", entry.idempotency_key()),
        None => println!("{id}  {}", entry.idempotency_key()),
    }
    // `sort_by_key` ổn định nên trong mỗi nhóm giữ thứ tự gốc.
    let mut lines: Vec<_> = entry.postings().iter().collect();
    lines.sort_by_key(|p| p.amount.dong() <= 0);
    for p in lines {
        let label = match p.flow() {
            Some(Flow::In) => "Vào",
            Some(Flow::Out) => "Ra",
            None => "--",
        };
        // Nhãn đã nói chiều, nên in số tiền không dấu.
        let magnitude = Money::from_dong(p.amount.dong().saturating_abs());
        println!("  {label:<3}  {:<20}{:>12}", p.account, magnitude);
    }
}

fn print_balances(ledger: &Ledger) -> Result<(), LedgerError> {
    for (account, amount) in ledger.balances()? {
        let kind = kind_of(&account).map_or("?", |k| k.label());
        println!("  {:<20}{:<11}{:>12}", account, kind, amount);
    }
    println!("Tổng toàn sổ: {}", ledger.grand_total()?);
    Ok(())
}

fn run() -> Result<(), LedgerError> {
    let mut ledger = Ledger::new();

    println!("== Bước 1 và 2: capture rồi chia ==");
    ledger.post(capture_entry()?)?;
    let split_id = ledger.post(split_entry()?)?.id;
    for (i, e) in ledger.entries().iter().enumerate() {
        print_entry(i, e);
    }
    println!("Số dư (projection từ posting):");
    print_balances(&ledger)?;

    println!();
    println!("== Bút toán đảo cho {split_id} ==");
    let reversal = ledger.reverse(split_id)?;
    println!("{} replayed = {}", reversal.id, reversal.replayed);
    let again = ledger.reverse(split_id)?;
    println!("{} replayed = {} (đảo lần hai)", again.id, again.replayed);
    println!("Số bút toán trong sổ: {}", ledger.len());
    print_entry(2, &ledger.entries()[2]);
    println!("Số dư sau khi đảo:");
    print_balances(&ledger)?;

    println!();
    println!("== Bút toán lệch bị từ chối ==");
    let bad = JournalEntryBuilder::new("trip:TN-0001:split-sai")
        .posting(acct(CLEARING)?, Money::from_dong(-25_500))
        .posting(acct(REVENUE)?, Money::from_dong(5_100))
        .posting(acct(DRIVER)?, Money::from_dong(20_000))
        .build();
    match bad {
        Ok(_) => println!("không thể xảy ra"),
        Err(e) => println!("Lỗi: {e}"),
    }

    println!();
    println!("== Vì sao không lưu một cột balance ==");
    let (stored, projected) = naive::lost_update(Money::from_dong(20_400));
    println!("Ô số dư sau hai lần cộng 20.400 xen kẽ: {stored}");
    println!("Projection từ hai posting:              {projected}");
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
    let cap = ledger.post(capture_entry().expect("bút toán capture cân"));
    check(
        cap.map(|p| p.id.to_string()) == Ok("JE-000001".to_string()),
        "capture là JE-000001",
    );
    let split = ledger.post(split_entry().expect("bút toán chia cân"));
    let split_id = split.map(|p| p.id);
    check(
        split_id.as_ref().map(|id| id.to_string()) == Ok("JE-000002".to_string()),
        "chia là JE-000002",
    );

    let bal = |l: &Ledger, raw: &str| l.balance(&AccountId::new(raw).expect("mã hợp lệ"));
    check(
        bal(&ledger, CUSTOMER) == Ok(Money::from_dong(-25_500)),
        "khách -25.500",
    );
    check(bal(&ledger, CLEARING) == Ok(Money::ZERO), "clearing về 0");
    check(
        bal(&ledger, REVENUE) == Ok(Money::from_dong(5_100)),
        "nền tảng 5.100",
    );
    check(
        bal(&ledger, DRIVER) == Ok(Money::from_dong(20_400)),
        "tài xế 20.400",
    );
    check(
        ledger.grand_total() == Ok(Money::ZERO),
        "tổng toàn sổ bằng 0",
    );

    // Bút toán đảo: JE-000003 trỏ về JE-000002, số dư về như sau bước 1.
    if let Ok(id) = split_id {
        let (rid, rev_of) = match ledger.reverse(id) {
            Ok(p) => (p.id.to_string(), p.entry.reverses()),
            Err(e) => (format!("lỗi {e}"), None),
        };
        check(rid == "JE-000003", "đảo là JE-000003");
        check(rev_of == Some(id), "JE-000003 trỏ về JE-000002");
        let replay = ledger.reverse(id).map(|p| p.replayed);
        check(replay == Ok(true), "đảo lần hai là replay");
        check(ledger.len() == 3, "sổ có 3 bút toán, không xoá bản gốc");
        check(
            bal(&ledger, DRIVER) == Ok(Money::ZERO),
            "tài xế về 0 sau đảo",
        );
        check(
            bal(&ledger, CLEARING) == Ok(Money::from_dong(25_500)),
            "clearing 25.500 sau đảo",
        );
        check(
            ledger.grand_total() == Ok(Money::ZERO),
            "tổng toàn sổ vẫn 0 sau đảo",
        );
    } else {
        check(false, "không có mã bút toán chia");
    }

    // Bút toán lệch 500 đồng bị từ chối, cả bằng try_new lẫn builder.
    let bad = JournalEntryBuilder::new("k")
        .posting(
            AccountId::new(CLEARING).expect("mã hợp lệ"),
            Money::from_dong(-25_500),
        )
        .posting(
            AccountId::new(REVENUE).expect("mã hợp lệ"),
            Money::from_dong(5_100),
        )
        .posting(
            AccountId::new(DRIVER).expect("mã hợp lệ"),
            Money::from_dong(20_000),
        )
        .build();
    check(
        bad == Err(LedgerError::Unbalanced {
            sum: Money::from_dong(-400),
        }),
        "lệch 400 đồng bị từ chối",
    );

    let (stored, projected) = naive::lost_update(Money::from_dong(20_400));
    check(
        stored == Money::from_dong(20_400) && projected == Money::from_dong(40_800),
        "cột balance mất một lần cập nhật, projection đúng",
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
