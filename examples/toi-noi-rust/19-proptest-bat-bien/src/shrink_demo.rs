//! Tìm phản ví dụ nhỏ nhất của bản `split_floor_only` (cố ý sai) bằng `TestRunner`.
//!
//! Dùng RNG xác định (ChaCha cố định) để kết quả giống nhau mỗi lần chạy, nhờ đó `--verify`
//! kiểm được. Không ghi file `proptest-regressions` (`failure_persistence: None`).

use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, TestError, TestRng, TestRunner};

use crate::money::Money;
use crate::split::split_floor_only;
use crate::strategies;

/// Runner xác định: cùng seed, cùng dãy dữ liệu, cùng kết quả shrink.
pub fn deterministic_runner(cases: u32) -> TestRunner {
    runner_with(cases, Config::default().max_shrink_iters)
}

fn runner_with(cases: u32, max_shrink_iters: u32) -> TestRunner {
    let config = Config {
        cases,
        max_shrink_iters,
        failure_persistence: None,
        ..Config::default()
    };
    TestRunner::new_with_rng(config, TestRng::deterministic_rng(RngAlgorithm::ChaCha))
}

/// Chạy bất biến «tổng các phần = tổng gốc» trên bản sai; trả đầu vào đã thu nhỏ.
pub fn minimal_counterexample() -> Option<(i64, Vec<u32>)> {
    counterexample(Config::default().max_shrink_iters)
}

/// Phản ví dụ đầu tiên, CHƯA thu nhỏ (`max_shrink_iters = 0`): dữ liệu thô proptest sinh ra.
pub fn raw_counterexample() -> Option<(i64, Vec<u32>)> {
    counterexample(0)
}

fn counterexample(max_shrink_iters: u32) -> Option<(i64, Vec<u32>)> {
    let mut runner = runner_with(1_000, max_shrink_iters);
    let result = runner.run(&strategies::split_input(), |(total, weights)| {
        let parts = split_floor_only(Money::from_dong(total), &weights).expect("đầu vào hợp lệ");
        let sum: i128 = parts.iter().map(|m| i128::from(m.dong())).sum();
        prop_assert_eq!(sum, i128::from(total));
        Ok(())
    });
    match result {
        Err(TestError::Fail(_, input)) => Some(input),
        _ => None,
    }
}
