//! Binary crate: dùng thư viện qua tên crate `ledger_lab`.

use std::fmt::Debug;
use std::process::ExitCode;

use ledger_lab::shrink_demo;
use ledger_lab::strategies;
use ledger_lab::{AccountId, JournalEntry, Ledger, LedgerError, Money, Posting, split};
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use proptest::test_runner::TestError;

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

fn show(parts: &[Money]) -> String {
    let texts: Vec<String> = parts.iter().map(Money::to_string).collect();
    texts.join(", ")
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Example test dưới dạng chương trình: một bút toán, ghi hai lần.
    let mut ledger = Ledger::new();
    let first = ledger.post(capture_trip_entry()?)?.replayed;
    let second = ledger.post(capture_trip_entry()?)?.replayed;
    println!(
        "Ghi 2 lần cùng key: replayed = {first} rồi {second}, sổ có {} bút toán, tổng = {}",
        ledger.len(),
        ledger.grand_total()?
    );

    // 2. Chia tiền không mất đồng.
    let fare = Money::from_dong(25_500);
    println!(
        "Chia {fare} theo [80, 20]: {}",
        show(&split(fare, &[80, 20])?)
    );
    let ten = Money::from_dong(10);
    println!(
        "Chia {ten} theo [1, 1, 1]: {}",
        show(&split(ten, &[1, 1, 1])?)
    );

    // 3. Strategy sinh bút toán cân: xem một mẫu ngẫu nhiên (RNG cố định).
    let mut runner = shrink_demo::deterministic_runner(1);
    let sample = strategies::balanced_postings()
        .new_tree(&mut runner)
        .map_err(|e| e.to_string())?
        .current();
    println!("Một bút toán sinh ngẫu nhiên ({} posting):", sample.len());
    for p in &sample {
        println!("  {:<20}{:>22}", p.account.to_string(), p.amount);
    }

    // 4. Shrinking: bản split sai, phản ví dụ thô và phản ví dụ đã thu nhỏ.
    if let (Some((raw_total, raw_w)), Some((total, w))) = (
        shrink_demo::raw_counterexample(),
        shrink_demo::minimal_counterexample(),
    ) {
        println!("split_floor_only, phản ví dụ thô: total = {raw_total}, weights = {raw_w:?}");
        println!("split_floor_only, sau shrinking:  total = {total}, weights = {w:?}");
    }
    Ok(())
}

/// Chạy một property qua `TestRunner` xác định; trả `Err(mô tả)` nếu bị phá.
fn check_property<S: Strategy>(
    strategy: &S,
    test: impl Fn(S::Value) -> Result<(), TestCaseError>,
) -> Result<(), String>
where
    S::Value: Debug,
{
    let mut runner = shrink_demo::deterministic_runner(1_000);
    match runner.run(strategy, test) {
        Ok(()) => Ok(()),
        Err(TestError::Fail(reason, input)) => Err(format!("{reason}; đầu vào nhỏ nhất {input:?}")),
        Err(other) => Err(other.to_string()),
    }
}

/// Ghi dãy bút toán (key `k-i`) vào sổ mới.
fn post_all(sequence: &[Vec<Posting>]) -> Result<Ledger, LedgerError> {
    let mut ledger = Ledger::new();
    for (i, postings) in sequence.iter().enumerate() {
        ledger.post(JournalEntry::try_new(&format!("k-{i}"), postings.clone())?)?;
    }
    Ok(ledger)
}

fn dong_of(parts: Result<Vec<Money>, ledger_lab::SplitError>) -> Vec<i64> {
    parts
        .map(|p| p.iter().map(|m| m.dong()).collect())
        .unwrap_or_default()
}

/// Kiểm các giá trị kỳ vọng in trong bài; trả danh sách lỗi (rỗng nếu đúng).
fn verify() -> Vec<String> {
    let mut failures = Vec::new();
    let mut check = |ok: bool, name: &str| {
        if !ok {
            failures.push(name.to_string());
        }
    };

    // Example test.
    check(
        dong_of(split(Money::from_dong(25_500), &[80, 20])) == [20_400, 5_100],
        "25.500 chia 80/20 = 20.400 + 5.100",
    );
    check(
        dong_of(split(Money::from_dong(10), &[1, 1, 1])) == [4, 3, 3],
        "10 chia 3 đều = 4, 3, 3",
    );
    let mut ledger = Ledger::new();
    let replays: Vec<bool> = (0..2)
        .map(|_| {
            capture_trip_entry()
                .ok()
                .and_then(|e| ledger.post(e).ok().map(|p| p.replayed))
                .unwrap_or(false)
        })
        .collect();
    check(replays == [false, true], "ghi lần hai là replay");
    check(ledger.len() == 1, "chỉ một bút toán sau khi ghi lại");

    // Property test: 1000 ca mỗi bất biến, RNG cố định.
    let split_ok = check_property(&strategies::split_input(), |(total, weights)| {
        let parts = split(Money::from_dong(total), &weights).expect("đầu vào hợp lệ");
        let sum: i128 = parts.iter().map(|m| i128::from(m.dong())).sum();
        prop_assert_eq!(sum, i128::from(total));
        Ok(())
    });
    check(split_ok.is_ok(), "1000 ca: tổng các phần = tổng gốc");
    if let Err(e) = split_ok {
        eprintln!("  {e}");
    }

    let ledger_ok = check_property(&strategies::entry_sequence(), |sequence| {
        let ledger = post_all(&sequence).expect("strategy chỉ sinh bút toán cân");
        prop_assert_eq!(ledger.grand_total(), Ok(Money::ZERO));
        Ok(())
    });
    check(ledger_ok.is_ok(), "1000 ca: tổng toàn sổ bằng 0");
    if let Err(e) = ledger_ok {
        eprintln!("  {e}");
    }

    // Shrinking: bản sai bị bắt và phản ví dụ nhỏ nhất đúng như bài ghi.
    check(
        shrink_demo::minimal_counterexample() == Some((1, vec![1, 1])),
        "shrinking ra (total = 1, weights = [1, 1])",
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
