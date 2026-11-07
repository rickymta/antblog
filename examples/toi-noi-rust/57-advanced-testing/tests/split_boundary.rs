//! Test thêm sau khi `cargo mutants` chỉ ra ba mutant sống sót: biên cước 0 và làm tròn nửa đồng.

use ledger_57_advanced_testing::split::{SplitError, fare_for_km, split_fare};

#[test]
fn zero_fare_is_valid_and_splits_to_zero() {
    // Diệt mutant `fare < 0` thành `fare <= 0`.
    let s = split_fare(0, 20).unwrap();
    assert_eq!((s.fee, s.driver), (0, 0));
}

#[test]
fn exact_half_rounds_up() {
    // 10 đồng × 15% = 1,5 đồng: đúng điểm giữa, làm tròn lên thành 2.
    // Diệt mutant `fee += 1` thành `fee -= 1` hoặc `fee *= 1`.
    let s = split_fare(10, 15).unwrap();
    assert_eq!((s.fee, s.driver), (2, 8));
}

#[test]
fn below_half_rounds_down() {
    // 9 đồng × 15% = 1,35 đồng: dưới nửa, làm tròn xuống thành 1.
    let s = split_fare(9, 15).unwrap();
    assert_eq!((s.fee, s.driver), (1, 8));
}

#[test]
fn percent_range_edges() {
    assert_eq!(split_fare(1_000, 0).unwrap().fee, 0);
    assert_eq!(split_fare(1_000, 100).unwrap().driver, 0);
    assert_eq!(split_fare(1_000, 101), Err(SplitError::PercentOutOfRange));
    assert_eq!(split_fare(1_000, -1), Err(SplitError::PercentOutOfRange));
}

#[test]
fn overflow_is_an_error_not_a_panic() {
    assert_eq!(split_fare(i64::MAX, 20), Err(SplitError::Overflow));
}

#[test]
fn fare_edges_around_two_km() {
    assert_eq!(fare_for_km(0), 12_000);
    assert_eq!(fare_for_km(2), 12_000);
    assert_eq!(fare_for_km(3), 16_500);
}
