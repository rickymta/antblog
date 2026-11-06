//! Bộ test «đường chính» viết đầu tiên: chuyến mẫu TN-0001 và hai lỗi hiển nhiên.

use ledger_57_advanced_testing::split::{SplitError, fare_for_km, split_fare};

#[test]
fn tn_0001_fare_and_split() {
    let fare = fare_for_km(5);
    assert_eq!(fare, 25_500);
    let s = split_fare(fare, 20).unwrap();
    assert_eq!((s.fee, s.driver), (5_100, 20_400));
}

#[test]
fn negative_fare_is_rejected() {
    assert_eq!(split_fare(-1, 20), Err(SplitError::NegativeFare));
}

#[test]
fn percent_over_100_is_rejected() {
    assert_eq!(split_fare(25_500, 150), Err(SplitError::PercentOutOfRange));
}
