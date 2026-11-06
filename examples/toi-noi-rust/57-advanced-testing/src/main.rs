//! Lab bài 57: chạy các thí nghiệm kiểm thử nâng cao trong bộ nhớ.
//! `cargo run` in kết quả; `cargo run -- --verify` kiểm các giá trị kỳ vọng trong bài.
//! Mutation (`cargo mutants`) và fuzz (`cargo fuzz`) chạy ngoài chương trình này, xem README.

use std::collections::{HashMap, HashSet};
use std::panic::{self, catch_unwind};
use std::process::ExitCode;

use jiff::Timestamp;
use ledger_57_advanced_testing::clock::StepClock;
use ledger_57_advanced_testing::fixture;
use ledger_57_advanced_testing::report::{compare, compare_unordered, reconcile};
use ledger_57_advanced_testing::split::{SplitError, fare_for_km, split_fare};
use ledger_57_advanced_testing::statement::{ParseError, parse_amount, parse_line, parse_line_v1};
use ledger_57_advanced_testing::testkit::{DEFAULT_SEED, SplitMix64, synthetic};

/// Input tối thiểu cargo-fuzz tìm ra (7 byte), nhúng từ file hồi quy.
const CRASH_V1: &[u8] = include_bytes!("../tests/data/crash_v1_char_boundary.bin");

fn main() -> ExitCode {
    let verify = std::env::args().any(|a| a == "--verify");
    let mut failures = Vec::new();
    run(verify, &mut failures);
    if !verify {
        return ExitCode::SUCCESS;
    }
    if failures.is_empty() {
        println!("VERIFY OK");
        ExitCode::SUCCESS
    } else {
        for f in &failures {
            eprintln!("VERIFY FAIL: {f}");
        }
        ExitCode::FAILURE
    }
}

/// Ghi lại một kiểm tra; không dừng chương trình để in được hết các lỗi.
fn check(failures: &mut Vec<String>, name: &str, ok: bool) {
    if !ok {
        failures.push(name.to_owned());
    }
}

fn run(verify: bool, failures: &mut Vec<String>) {
    // 1. Chia tiền: chuyến TN-0001 và điểm giữa nửa đồng.
    let fare = fare_for_km(5);
    let tn = split_fare(fare, 20).expect("TN-0001 hợp lệ");
    let tie = split_fare(10, 15).expect("10 đồng 15% hợp lệ");
    println!("TN-0001: cước {fare} phí {} tài xế {}", tn.fee, tn.driver);
    println!(
        "điểm giữa: 10 đồng x 15% -> phí {} tài xế {}",
        tie.fee, tie.driver
    );
    check(
        failures,
        "TN-0001 25.500/5.100/20.400",
        (fare, tn.fee, tn.driver) == (25_500, 5_100, 20_400),
    );
    check(
        failures,
        "10 đồng 15% làm tròn lên 2/8",
        (tie.fee, tie.driver) == (2, 8),
    );
    check(
        failures,
        "phần trăm 101 bị từ chối",
        split_fare(1_000, 101) == Err(SplitError::PercentOutOfRange),
    );
    check(failures, "cước 0 hợp lệ", split_fare(0, 20).is_ok());
    check(
        failures,
        "tràn số trả Err",
        split_fare(i64::MAX, 20) == Err(SplitError::Overflow),
    );

    // 2. Input hồi quy từ fuzz: bản đã sửa trả lỗi, bản v1 panic (bắt bằng catch_unwind).
    let crash = std::str::from_utf8(CRASH_V1).expect("UTF-8");
    let fixed = parse_line(crash);
    let hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {})); // tắt thông báo panic để đầu ra gọn
    let v1_panics = catch_unwind(|| parse_line_v1(crash)).is_err();
    panic::set_hook(hook);
    println!("input hồi quy {CRASH_V1:?}");
    println!("  bản sửa: {fixed:?}; bản v1 panic: {v1_panics}");
    check(failures, "input hồi quy dài 7 byte", CRASH_V1.len() == 7);
    check(
        failures,
        "bản sửa trả BadAmount",
        fixed == Err(ParseError::BadAmount),
    );
    check(failures, "bản v1 panic", v1_panics);
    check(
        failures,
        "52.000 VND đọc được",
        parse_amount("52.000 VND") == Ok(52_000),
    );

    // 3. Thứ tự HashMap: bản chưa sắp ra nhiều thứ tự, bản sắp ra đúng một.
    let ledger = |_: usize| -> HashMap<String, i64> {
        (1..=8)
            .map(|i| (format!("vx_ch_{i:04}"), i64::from(i) * 1_000))
            .collect()
    };
    let psp = HashMap::new();
    let order = |v: Vec<ledger_57_advanced_testing::report::Mismatch>| -> Vec<String> {
        v.into_iter().map(|m| m.external_ref).collect()
    };
    let unordered: HashSet<_> = (0..30)
        .map(|i| order(compare_unordered(&ledger(i), &psp)))
        .collect();
    let sorted: HashSet<_> = (0..30).map(|i| order(compare(&ledger(i), &psp))).collect();
    println!(
        "30 lần so khớp 8 ref: bản chưa sắp thấy nhiều hơn một thứ tự = {}, bản đã sắp thấy {} thứ tự",
        unordered.len() > 1,
        sorted.len()
    );
    check(failures, "bản chưa sắp không tất định", unordered.len() > 1);
    check(failures, "bản đã sắp chỉ một thứ tự", sorted.len() == 1);

    // 4. Báo cáo với đồng hồ giả: giống hệt từng byte giữa hai lần chạy.
    let start: Timestamp = "2026-11-05T17:00:00Z".parse().expect("RFC 3339");
    let r1 = reconcile(
        &fixture::ledger(),
        fixture::STATEMENT,
        &StepClock::new(start, 250),
        "run-0001",
    );
    let r2 = reconcile(
        &fixture::ledger(),
        fixture::STATEMENT,
        &StepClock::new(start, 250),
        "run-0001",
    );
    print!("{}", r1.render_text());
    check(failures, "báo cáo lặp lại giống hệt", r1 == r2);
    check(failures, "duration 250 ms", r1.duration_ms == 250);
    check(
        failures,
        "khớp 2, lệch 3, hỏng 3",
        (r1.matched, r1.mismatches.len(), r1.bad_lines.len()) == (2, 3, 3),
    );

    // 5. Seed cố định: cùng seed cùng dữ liệu, kể cả số lệch cài sẵn.
    let mut rng = SplitMix64::new(42);
    let first = rng.next_u64();
    let a = synthetic(DEFAULT_SEED, 300);
    let b = synthetic(DEFAULT_SEED, 300);
    println!("SplitMix64(42) đầu tiên = {first}");
    println!(
        "seed {DEFAULT_SEED}: khớp {}, thiếu ta {}, thiếu PSP {}, lệch tiền {}",
        a.matched, a.missing_in_ledger, a.missing_in_psp, a.amount_mismatch
    );
    check(
        failures,
        "SplitMix64(42) đầu tiên",
        first == 13_679_457_532_755_275_413,
    );
    check(
        failures,
        "cùng seed cùng sao kê",
        a.statement == b.statement,
    );
    check(
        failures,
        "tổng ref bằng 300",
        a.matched + a.missing_in_ledger + a.missing_in_psp + a.amount_mismatch == 300,
    );
    if !verify {
        println!("(thêm -- --verify để kiểm tự động)");
    }
}
