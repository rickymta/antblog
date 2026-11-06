//! Test tất định: thứ tự `HashMap`, đồng hồ giả và seed cố định.

use std::collections::{HashMap, HashSet};

use jiff::Timestamp;
use ledger_57_advanced_testing::clock::{Clock, StepClock};
use ledger_57_advanced_testing::fixture;
use ledger_57_advanced_testing::report::{compare, compare_unordered, new_run_id, reconcile};
use ledger_57_advanced_testing::testkit::{SplitMix64, seed_from_env, synthetic};

/// Tám ref chỉ có phía sổ cái, để mọi ref đều thành «thiếu bên PSP».
fn eight_missing() -> HashMap<String, i64> {
    (1..=8)
        .map(|i| (format!("vx_ch_{i:04}"), i64::from(i) * 1_000))
        .collect()
}

fn order_of(mismatches: &[ledger_57_advanced_testing::report::Mismatch]) -> Vec<String> {
    mismatches.iter().map(|m| m.external_ref.clone()).collect()
}

#[test]
fn unordered_compare_is_not_stable() {
    // Mỗi `HashMap` mới có khóa băm ngẫu nhiên riêng, nên thứ tự duyệt khác nhau giữa các map.
    // Chạy 30 lần với 8 phần tử: xác suất ra cùng một thứ tự cả 30 lần là (1/40320)^29, coi như không.
    let psp = HashMap::new();
    let orders: HashSet<Vec<String>> = (0..30)
        .map(|_| order_of(&compare_unordered(&eight_missing(), &psp)))
        .collect();
    assert!(
        orders.len() > 1,
        "mong thấy nhiều thứ tự, chỉ thấy {}",
        orders.len()
    );
}

#[test]
fn sorted_compare_is_stable() {
    let psp = HashMap::new();
    let orders: HashSet<Vec<String>> = (0..30)
        .map(|_| order_of(&compare(&eight_missing(), &psp)))
        .collect();
    assert_eq!(orders.len(), 1);
}

#[test]
fn step_clock_makes_duration_exact() {
    let start: Timestamp = "2026-11-05T17:00:00Z".parse().unwrap();
    let clock = StepClock::new(start, 250);
    let report = reconcile(&fixture::ledger(), fixture::STATEMENT, &clock, "run-0001");
    assert_eq!(report.duration_ms, 250);
    assert_eq!(report.generated_at, "2026-11-05T17:00:00.25Z");
    // Cùng đầu vào và đồng hồ giả mới: báo cáo giống hệt từng byte.
    let again = reconcile(
        &fixture::ledger(),
        fixture::STATEMENT,
        &StepClock::new(start, 250),
        "run-0001",
    );
    assert_eq!(report, again);
}

#[test]
fn step_clock_zero_step_stands_still() {
    let start: Timestamp = "2026-11-05T17:00:00Z".parse().unwrap();
    let clock = StepClock::new(start, 0);
    assert_eq!(clock.now(), clock.now());
}

#[test]
fn run_ids_differ_between_calls() {
    assert_ne!(new_run_id(), new_run_id());
}

#[test]
fn same_seed_same_stream() {
    let mut a = SplitMix64::new(42);
    let mut b = SplitMix64::new(42);
    let xs: Vec<u64> = (0..5).map(|_| a.next_u64()).collect();
    let ys: Vec<u64> = (0..5).map(|_| b.next_u64()).collect();
    assert_eq!(xs, ys);
    // Giá trị đầu của SplitMix64 với seed 42, cố định mãi mãi.
    assert_eq!(xs[0], 13_679_457_532_755_275_413);
}

#[test]
fn synthetic_counts_match_report_for_many_seeds() {
    // Seed của CI cố định; đặt LEDGER_TEST_SEED=<n> để chạy lại đúng ca vừa hỏng.
    let first = seed_from_env();
    for seed in first..first + 20 {
        let data = synthetic(seed, 300);
        let report = reconcile(
            &data.ledger,
            &data.statement,
            &StepClock::new("2026-11-05T00:00:00Z".parse().unwrap(), 1),
            "run-seeded",
        );
        let count = |k| report.mismatches.iter().filter(|m| m.kind == k).count();
        use ledger_57_advanced_testing::report::MismatchKind::*;
        assert_eq!(report.matched, data.matched, "seed={seed}");
        assert_eq!(
            count(MissingInLedger),
            data.missing_in_ledger,
            "seed={seed}"
        );
        assert_eq!(count(MissingInPsp), data.missing_in_psp, "seed={seed}");
        assert_eq!(count(AmountMismatch), data.amount_mismatch, "seed={seed}");
        assert!(report.bad_lines.is_empty(), "seed={seed}");
    }
}
