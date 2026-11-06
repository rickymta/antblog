//! Binary crate: dùng thư viện qua tên crate `ledger_lab`.

use std::process::ExitCode;

use ledger_lab::{
    AccountId, EntryId, JournalEntry, Km, Ledger, LedgerError, Money, MoneyError, Posting, Usd,
    Vnd, trip_fare,
};

/// Bút toán thu cước chuyến TN-0001: khách -25.500, nền tảng +5.100, tài xế +20.400.
fn capture_trip_entry() -> Result<JournalEntry, LedgerError> {
    // `parse()?`: lỗi `IdError` tự đổi thành `LedgerError` nhờ `impl From`.
    JournalEntry::try_new(
        "trip:TN-0001:capture",
        vec![
            Posting::new("customer:CU-001".parse()?, Money::from_dong(-25_500)),
            Posting::new("platform:revenue".parse()?, Money::from_dong(5_100)),
            Posting::new("driver:DR-001".parse()?, Money::from_dong(20_400)),
        ],
    )
}

fn run() -> Result<(), LedgerError> {
    println!("== Mã tài khoản: parse có kiểm tra ==");
    for raw in [
        "customer:CU-001",
        "psp:vixanh:clearing",
        "customer",
        " driver:DR-001",
        "customer::CU-001",
        "",
    ] {
        match raw.parse::<AccountId>() {
            Ok(id) => println!("  {raw:?} -> hợp lệ: {id}"),
            Err(e) => println!("  {raw:?} -> {e}"),
        }
    }

    println!("== Tiền: kiểu riêng, cộng trừ có kiểm tra ==");
    let platform_fee = Money::from_dong(5_100);
    let driver_share = Money::from_dong(20_400);
    let fare = (platform_fee + driver_share)?;
    println!("  {platform_fee} + {driver_share} = {fare}");
    println!("  cước chuyến 5 km = {}", trip_fare(Km::new(5)));
    let max = Money::from_dong(i64::MAX);
    match max + Money::from_dong(1) {
        Ok(sum) => println!("  tổng = {sum}"),
        Err(e) => println!("  {max} + 1 VND -> lỗi: {e}"),
    }
    let wide = i128::from(i64::MAX) + 1;
    match Money::<Vnd>::try_from(wide) {
        Ok(m) => println!("  {wide} -> {m}"),
        Err(e) => println!("  {wide} -> lỗi: {e}"),
    }
    let usd = Money::<Usd>::new(100);
    println!("  Debug: {fare:?} và {usd:?} là hai kiểu khác nhau");

    println!("== Mã bút toán ==");
    let first = EntryId::try_from(1)?;
    println!("  {first} có số thứ tự {}", first.sequence());
    for raw in ["JE-000042", "JE-42", "TN-000001"] {
        match raw.parse::<EntryId>() {
            Ok(id) => println!("  {raw:?} -> {id}"),
            Err(e) => println!("  {raw:?} -> {e}"),
        }
    }

    println!("== Sổ cái ==");
    let mut ledger = Ledger::new();
    let posted = ledger.post(capture_trip_entry()?)?;
    println!(
        "  {} {} ({} posting, replayed = {})",
        posted.id,
        posted.entry.idempotency_key(),
        posted.entry.postings().len(),
        posted.replayed
    );
    let again = ledger.post(capture_trip_entry()?)?;
    println!("  {} ghi lại: replayed = {}", again.id, again.replayed);
    println!("  Số dư (xếp theo mã tài khoản):");
    for (account, balance) in ledger.balances()? {
        println!("    {account:<18}{balance:>12}");
    }
    println!("  Tổng toàn sổ: {}", ledger.grand_total()?);
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

    // Money: cộng, trừ, tràn, TryFrom.
    let fee = Money::from_dong(5_100);
    let driver = Money::from_dong(20_400);
    check(
        fee + driver == Ok(Money::from_dong(25_500)),
        "5.100 + 20.400 = 25.500",
    );
    check(
        driver - fee == Ok(Money::from_dong(15_300)),
        "20.400 - 5.100 = 15.300",
    );
    check(
        Money::from_dong(i64::MAX) + Money::from_dong(1) == Err(MoneyError::Overflow),
        "tràn khi cộng báo Overflow",
    );
    check(
        Money::<Vnd>::try_from(i128::from(i64::MAX) + 1) == Err(MoneyError::OutOfRange),
        "i128 vượt i64 bị từ chối",
    );
    check(
        trip_fare(Km::new(5)) == Money::from_dong(25_500),
        "cước 5 km = 25.500",
    );
    check(
        Money::from_dong(25_500).to_string() == "25.500 VND",
        "hiển thị 25.500 VND",
    );
    check(
        size_of::<Money>() == size_of::<i64>(),
        "phantom type không tốn byte",
    );

    // AccountId.
    check(
        "customer:CU-001".parse::<AccountId>().is_ok(),
        "customer:CU-001 hợp lệ",
    );
    check(
        "psp:vixanh:clearing".parse::<AccountId>().is_ok(),
        "psp:vixanh:clearing hợp lệ",
    );
    check(
        "customer".parse::<AccountId>().is_err(),
        "thiếu dấu hai chấm bị từ chối",
    );
    check(
        " driver:DR-001".parse::<AccountId>().is_err(),
        "khoảng trắng đầu bị từ chối",
    );

    // EntryId.
    let id1 = EntryId::try_from(1);
    check(
        id1.as_ref().map(ToString::to_string) == Ok("JE-000001".to_string()),
        "EntryId 1 in ra JE-000001",
    );
    check(
        "JE-000001".parse::<EntryId>() == id1,
        "parse JE-000001 ra EntryId 1",
    );
    check(
        "JE-1".parse::<EntryId>().is_err(),
        "JE-1 không phải dạng chuẩn",
    );
    check(EntryId::try_from(0).is_err(), "số thứ tự 0 bị từ chối");

    // Ledger.
    let mut ledger = Ledger::new();
    let entry = capture_trip_entry().expect("bút toán mẫu cân");
    let posted = ledger.post(entry).expect("ghi lần đầu");
    check(
        posted.id.to_string() == "JE-000001",
        "mã bút toán JE-000001",
    );
    check(!posted.replayed, "lần đầu không phải replay");
    let again = ledger
        .post(capture_trip_entry().expect("bút toán mẫu cân"))
        .expect("ghi lại cùng key");
    check(again.replayed, "lần hai là replay");
    check(ledger.len() == 1, "chỉ một bút toán sau khi ghi lại");
    let balance_of = |raw: &str| ledger.balance(&raw.parse::<AccountId>().expect("mã hợp lệ"));
    check(
        balance_of("customer:CU-001") == Ok(Money::from_dong(-25_500)),
        "khách -25.500",
    );
    check(
        balance_of("platform:revenue") == Ok(Money::from_dong(5_100)),
        "nền tảng 5.100",
    );
    check(
        balance_of("driver:DR-001") == Ok(Money::from_dong(20_400)),
        "tài xế 20.400",
    );
    check(
        ledger.grand_total() == Ok(Money::ZERO),
        "tổng toàn sổ bằng 0",
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
