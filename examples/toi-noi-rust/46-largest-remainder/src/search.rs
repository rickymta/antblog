//! Tìm phản ví dụ: bằng proptest (có shrinking) và bằng vét cạn trên không gian nhỏ.
//!
//! RNG của proptest cố định (ChaCha) để kết quả giống nhau mỗi lần chạy, nhờ đó `--verify`
//! kiểm được. Không ghi file `proptest-regressions` (`failure_persistence: None`).

use std::collections::{BTreeSet, HashMap};

use ledger_core::Money;
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, TestError, TestRng, TestRunner};
use rust_decimal::RoundingStrategy;

use crate::allocate::{Party, allocate};
use crate::naive::{drift, round_each};

/// Đầu vào cho phép chia ngây thơ: tổng nhỏ (để ca fail đọc được), 2 đến 4 bên, trọng số 1..=3.
fn naive_input() -> impl Strategy<Value = (i64, Vec<u32>)> {
    (0i64..=100, prop::collection::vec(1u32..=3, 2..=4))
}

fn runner_with(max_shrink_iters: u32) -> TestRunner {
    let config = Config {
        cases: 1_000,
        max_shrink_iters,
        failure_persistence: None,
        ..Config::default()
    };
    TestRunner::new_with_rng(config, TestRng::deterministic_rng(RngAlgorithm::ChaCha))
}

fn naive_counterexample(max_shrink_iters: u32) -> Option<(i64, Vec<u32>)> {
    let mut runner = runner_with(max_shrink_iters);
    let result = runner.run(&naive_input(), |(total, weights)| {
        let parts = round_each(total, &weights, RoundingStrategy::MidpointAwayFromZero)
            .expect("đầu vào hợp lệ");
        prop_assert_eq!(drift(total, &parts), 0);
        Ok(())
    });
    match result {
        Err(TestError::Fail(_, input)) => Some(input),
        _ => None,
    }
}

/// Phản ví dụ đầu tiên proptest tìm được cho «làm tròn từng phần», CHƯA thu nhỏ.
pub fn naive_raw_counterexample() -> Option<(i64, Vec<u32>)> {
    naive_counterexample(0)
}

/// Phản ví dụ sau khi proptest thu nhỏ.
pub fn naive_minimal_counterexample() -> Option<(i64, Vec<u32>)> {
    naive_counterexample(Config::default().max_shrink_iters)
}

fn shares_dong(total: i64, weights: &[u64]) -> Vec<i64> {
    let keys = ["p0", "p1", "p2", "p3", "p4", "p5", "p6", "p7"];
    let parties: Vec<Party> = weights
        .iter()
        .zip(keys)
        .map(|(&w, k)| Party::new(k, w))
        .collect();
    allocate(Money::from_dong(total), &parties)
        .expect("đầu vào hợp lệ")
        .iter()
        .map(|s| s.amount.dong())
        .collect()
}

/// Vét cạn 3 bên, trọng số 1..=`max_weight`, tổng 0..`max_total`: tăng tổng thêm 1 đồng mà
/// phần của một bên GIẢM (nghịch lý Alabama của largest remainder). Trả ca nhỏ nhất theo
/// (tổng, trọng số).
pub fn alabama_counterexample(max_total: i64, max_weight: u64) -> Option<AlabamaCase> {
    for total in 0..max_total {
        for a in 1..=max_weight {
            for b in 1..=max_weight {
                for c in 1..=max_weight {
                    let weights = [a, b, c];
                    let before = shares_dong(total, &weights);
                    let after = shares_dong(total + 1, &weights);
                    if let Some(i) = (0..3).find(|&i| after[i] < before[i]) {
                        return Some(AlabamaCase {
                            total,
                            weights,
                            before,
                            after,
                            party: i,
                        });
                    }
                }
            }
        }
    }
    None
}

/// Một ca Alabama: tổng `total` rồi `total + 1`, bên `party` bị giảm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlabamaCase {
    /// Tổng trước khi tăng.
    pub total: i64,
    /// Trọng số ba bên.
    pub weights: [u64; 3],
    /// Các phần khi chia `total`.
    pub before: Vec<i64>,
    /// Các phần khi chia `total + 1`.
    pub after: Vec<i64>,
    /// Chỉ số bên bị giảm.
    pub party: usize,
}

/// Cách làm SAI: phá hòa theo thứ tự duyệt `HashMap`. Mỗi `HashMap` mới có hạt giống băm riêng,
/// nên cùng dữ liệu có thể ra thứ tự khác. Trả bên nhận đồng lẻ trong `trials` lần thử.
pub fn hashmap_tiebreak_winners(trials: usize) -> BTreeSet<String> {
    let mut winners = BTreeSet::new();
    for _ in 0..trials {
        // Ba bên bằng nhau chia 100 đồng: mỗi bên 33, dư 1, phần dư hòa nhau.
        let weights: HashMap<String, u64> = ["a", "b", "c"]
            .iter()
            .map(|k| ((*k).to_string(), 1))
            .collect();
        if let Some(first) = weights.keys().next() {
            winners.insert(first.clone());
        }
    }
    winners
}

/// Chạy `cases` ca ngẫu nhiên (RNG cố định) cho hai bất biến của `allocate`: tổng các phần bằng
/// tổng gốc, và mỗi phần lệch phần chính xác dưới 1 đồng. `Err` chứa phản ví dụ đã thu nhỏ.
pub fn check_allocation_invariants(cases: u32) -> Result<(), String> {
    let config = Config {
        cases,
        failure_persistence: None,
        ..Config::default()
    };
    let mut runner =
        TestRunner::new_with_rng(config, TestRng::deterministic_rng(RngAlgorithm::ChaCha));
    let input = (
        prop_oneof![-100_000i64..=100_000, any::<i64>()],
        prop::collection::vec(0u32..=1_000, 1..=8)
            .prop_filter("tổng trọng số > 0", |w| w.iter().any(|&x| x > 0)),
    );
    runner
        .run(&input, |(total, weights)| {
            let parties: Vec<Party> = weights
                .iter()
                .enumerate()
                .map(|(i, &w)| Party::new(&format!("p{i}"), u64::from(w)))
                .collect();
            let shares = allocate(Money::from_dong(total), &parties).expect("đầu vào hợp lệ");
            let sum: i128 = shares.iter().map(|s| i128::from(s.amount.dong())).sum();
            prop_assert_eq!(sum, i128::from(total));
            let weight_sum: i128 = weights.iter().map(|&w| i128::from(w)).sum();
            for (share, &w) in shares.iter().zip(&weights) {
                let diff = i128::from(share.amount.dong()) * weight_sum
                    - i128::from(total) * i128::from(w);
                prop_assert!(diff.abs() < weight_sum);
            }
            Ok(())
        })
        .map_err(|e| e.to_string())
}
