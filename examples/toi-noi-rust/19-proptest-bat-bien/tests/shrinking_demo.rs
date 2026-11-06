//! Shrinking: cố ý kiểm bất biến trên bản `split_floor_only` sai.

use ledger_lab::strategies;
use ledger_lab::{Money, shrink_demo, split::split_floor_only};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

proptest! {
    // Mặc định (`SourceParallel`) tìm lib.rs/main.rs ngược lên cây thư mục, có thể ra ngoài crate.
    // `WithSource` luôn ghi cạnh file test: `tests/shrinking_demo.proptest-regressions`.
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
            "proptest-regressions",
        ))),
        ..ProptestConfig::default()
    })]

    // Cố ý FAIL, nên mặc định bị bỏ qua. Chạy: `cargo test -- --ignored`.
    // Lần fail đầu proptest ghi seed vào `tests/shrinking_demo.proptest-regressions`.
    #[test]
    #[ignore = "cố ý fail để xem shrinking; chạy bằng --ignored"]
    fn floor_only_split_keeps_total((total, weights) in strategies::split_input()) {
        let parts = split_floor_only(Money::from_dong(total), &weights).unwrap();
        let sum: i128 = parts.iter().map(|m| i128::from(m.dong())).sum();
        prop_assert_eq!(sum, i128::from(total));
    }
}

// Test xanh: kiểm rằng bất biến bị phá VÀ phản ví dụ đã thu nhỏ còn rất nhỏ.
#[test]
fn shrinking_reaches_a_tiny_counterexample() {
    let (total, weights) = shrink_demo::minimal_counterexample().expect("bản sai phải bị bắt");
    println!("phản ví dụ nhỏ nhất: total = {total}, weights = {weights:?}");
    assert!(total.abs() <= 3, "total chưa thu nhỏ: {total}");
    assert!(weights.len() <= 3, "weights chưa thu nhỏ: {weights:?}");
}
