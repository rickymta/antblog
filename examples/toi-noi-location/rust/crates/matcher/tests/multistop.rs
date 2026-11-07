//! LOC-50: proptest cho nhiều điểm dừng có ràng buộc lấy trước giao. Ma trận ngẫu nhiên, bất đối xứng, không cần là metric:
//! các bất biến dưới đây không dựa vào bất đẳng thức tam giác.

use matcher::multistop::{Instance, cheapest_insertion, exact_dp, local_search, solve};
use proptest::prelude::*;
use std::time::Duration;

fn instance(pairs: usize, seed_costs: Vec<u32>) -> Instance {
    let m = 2 * pairs + 1;
    let mut cost: Vec<i64> = seed_costs
        .iter()
        .take(m * m)
        .map(|&c| i64::from(c))
        .collect();
    for i in 0..m {
        cost[i * m + i] = 0;
    }
    Instance::new(pairs, cost).unwrap()
}

fn arb_instance(max_pairs: usize) -> impl Strategy<Value = Instance> {
    (1..=max_pairs).prop_flat_map(|p| {
        let m = 2 * p + 1;
        proptest::collection::vec(1u32..5_000, m * m).prop_map(move |c| instance(p, c))
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(96))]

    /// Chèn và tìm kiếm cục bộ luôn cho tuyến hợp lệ: bắt đầu ở 0, mỗi đỉnh đúng một lần, lấy trước giao.
    #[test]
    fn routes_are_valid_permutations_with_precedence(inst in arb_instance(8)) {
        let ins = cheapest_insertion(&inst);
        prop_assert!(inst.is_feasible(&ins));
        prop_assert_eq!(ins[0], 0);
        let sol = solve(&inst, None);
        prop_assert!(inst.is_feasible(&sol.route));
        prop_assert_eq!(sol.route.len(), inst.nodes());
        prop_assert_eq!(sol.cost, inst.route_cost(&sol.route));
    }

    /// Tìm kiếm cục bộ chỉ nhận bước giảm chi phí nên không bao giờ tệ hơn bước chèn.
    #[test]
    fn local_search_never_worse_than_insertion(inst in arb_instance(8)) {
        let ins = cheapest_insertion(&inst);
        let (ls, _, timed_out) = local_search(&inst, ins.clone(), None);
        prop_assert!(!timed_out);
        prop_assert!(inst.is_feasible(&ls));
        prop_assert!(inst.route_cost(&ls) <= inst.route_cost(&ins));
    }

    /// Tối ưu chính xác không lớn hơn chi phí của heuristic, và tuyến của nó cũng hợp lệ.
    #[test]
    fn dp_optimum_is_a_lower_bound_of_the_heuristics(inst in arb_instance(5)) {
        let (route, opt) = exact_dp(&inst).unwrap();
        prop_assert!(inst.is_feasible(&route));
        prop_assert_eq!(opt, inst.route_cost(&route));
        let sol = solve(&inst, None);
        prop_assert!(opt <= sol.cost);
        prop_assert!(opt <= sol.insertion_cost);
    }

    /// Kết quả tìm kiếm cục bộ đã là cực tiểu cục bộ: chạy lại từ nó không đổi gì.
    #[test]
    fn local_search_result_is_a_fixed_point(inst in arb_instance(6)) {
        let sol = solve(&inst, None);
        let (again, moves, _) = local_search(&inst, sol.route.clone(), None);
        prop_assert_eq!(moves, 0);
        prop_assert_eq!(again, sol.route);
    }

    /// Hết ngân sách (kể cả 0) vẫn trả tuyến hợp lệ; tất định: hai lần chạy không ngân sách cho cùng tuyến.
    #[test]
    fn budgeted_solve_is_valid_and_unbudgeted_is_deterministic(inst in arb_instance(6)) {
        let quick = solve(&inst, Some(Duration::ZERO));
        prop_assert!(inst.is_feasible(&quick.route));
        let (a, b) = (solve(&inst, None), solve(&inst, None));
        prop_assert_eq!(a.route, b.route);
    }
}
