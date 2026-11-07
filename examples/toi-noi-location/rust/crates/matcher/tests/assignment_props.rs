//! Tính chất của phân công (proptest): hợp lệ, Hungarian không tệ hơn tham lam, khớp vét cạn, cắt tỉa không rẻ hơn tối ưu đầy đủ.

use matcher::assign::{brute_force, greedy_fifo, greedy_global, hungarian, total_cost};
use matcher::cost::{Dense, FORBIDDEN};
use matcher::policy::CostPolicy;
use matcher::prune::{PruneConfig, Sparse, candidates, components, solve_sparse_hungarian};
use proptest::prelude::*;

fn matrix(max_dim: usize, forbid: bool) -> impl Strategy<Value = Dense> {
    (1..=max_dim, 1..=max_dim).prop_flat_map(move |(r, c)| {
        proptest::collection::vec(
            prop_oneof![
                85 => 1i64..5000,
                15 => Just(if forbid { FORBIDDEN } else { 77 }),
            ],
            r * c,
        )
        .prop_map(move |data| Dense {
            rows: r,
            cols: c,
            data,
        })
    })
}

fn valid(costs: &Dense, pairs: &[(usize, usize)]) -> bool {
    let (mut rs, mut cs) = (vec![false; costs.rows], vec![false; costs.cols]);
    pairs.iter().all(|&(r, c)| {
        let ok =
            r < costs.rows && c < costs.cols && !rs[r] && !cs[c] && costs.get(r, c) < FORBIDDEN / 2;
        rs[r] = true;
        cs[c] = true;
        ok
    })
}

proptest! {
    /// Mỗi khách và mỗi tài xế tối đa một lần, không có cặp bị cấm, cho cả ba thuật toán.
    #[test]
    fn assignment_is_valid_each_driver_at_most_once(c in matrix(8, true)) {
        prop_assert!(valid(&c, &hungarian(&c)));
        prop_assert!(valid(&c, &greedy_fifo(&c)));
        prop_assert!(valid(&c, &greedy_global(&c)));
    }

    /// Chi phí lô (Hungarian) ≤ chi phí tham lam trên cùng ma trận (khi mọi ô được phép, hai bên ghép đủ min(hàng, cột) cặp).
    #[test]
    fn batch_cost_never_exceeds_greedy_on_the_same_matrix(c in matrix(9, false)) {
        let h = hungarian(&c);
        let (g1, g2) = (greedy_fifo(&c), greedy_global(&c));
        prop_assert_eq!(h.len(), c.rows.min(c.cols));
        prop_assert_eq!(g1.len(), h.len());
        prop_assert_eq!(g2.len(), h.len());
        prop_assert!(total_cost(&c, &h) <= total_cost(&c, &g1));
        prop_assert!(total_cost(&c, &h) <= total_cost(&c, &g2));
    }

    /// Có ô bị cấm: Hungarian ghép nhiều cặp nhất có thể, rồi rẻ nhất; luôn không kém tham lam theo thứ tự (số cặp, rồi chi phí).
    #[test]
    fn with_forbidden_cells_hungarian_dominates_greedy_lexicographically(c in matrix(8, true)) {
        let h = hungarian(&c);
        for g in [greedy_fifo(&c), greedy_global(&c)] {
            prop_assert!(h.len() >= g.len());
            if h.len() == g.len() {
                prop_assert!(total_cost(&c, &h) <= total_cost(&c, &g));
            }
        }
    }

    /// Khớp vét cạn trên ma trận nhỏ (cả số cặp lẫn tổng chi phí), kể cả ma trận chữ nhật và có ô bị cấm.
    #[test]
    fn hungarian_matches_brute_force(c in matrix(5, true)) {
        let (h, b) = (hungarian(&c), brute_force(&c));
        prop_assert_eq!(h.len(), b.len());
        prop_assert_eq!(total_cost(&c, &h), total_cost(&c, &b));
    }

    /// Cắt tỉa (ma trận thưa, tách vùng) không bao giờ rẻ hơn tối ưu đầy đủ khi ghép cùng số cặp, và không ghép nhiều hơn.
    #[test]
    fn pruned_cost_is_never_below_the_full_optimum(
        c in matrix(8, false),
        keep in proptest::collection::vec(any::<bool>(), 64),
    ) {
        let mut sp = Sparse { rows: c.rows, cols: c.cols, edges: Vec::new() };
        for r in 0..c.rows {
            for k in 0..c.cols {
                if keep[(r * 8 + k) % keep.len()] {
                    sp.edges.push((r, k, c.get(r, k)));
                }
            }
        }
        let full = hungarian(&c);
        let sol = solve_sparse_hungarian(&sp);
        let pruned_cost: i64 = sol.pairs.iter().map(|&(r, k)| c.get(r, k)).sum();
        prop_assert!(sol.pairs.len() <= full.len());
        if sol.pairs.len() == full.len() {
            prop_assert!(pruned_cost >= total_cost(&c, &full));
        }
        // các vùng độc lập hợp lại đúng tập cạnh: mỗi khách và mỗi xe thuộc tối đa một vùng
        let comps = components(&sp);
        let mut seen_r = vec![false; c.rows];
        let mut seen_c = vec![false; c.cols];
        for (rs, cs) in &comps {
            for &r in rs { prop_assert!(!seen_r[r]); seen_r[r] = true; }
            for &k in cs { prop_assert!(!seen_c[k]); seen_c[k] = true; }
        }
    }

    /// Giải theo vùng cho đúng tối ưu của ma trận thưa viết dưới dạng đặc (vùng độc lập không làm mất tối ưu).
    #[test]
    fn solving_by_components_equals_solving_the_sparse_matrix_densely(
        c in matrix(7, false),
        keep in proptest::collection::vec(any::<bool>(), 49),
    ) {
        let mut sp = Sparse { rows: c.rows, cols: c.cols, edges: Vec::new() };
        let mut dense = Dense::new(c.rows, c.cols, FORBIDDEN);
        for r in 0..c.rows {
            for k in 0..c.cols {
                if keep[(r * 7 + k) % keep.len()] {
                    sp.edges.push((r, k, c.get(r, k)));
                    dense.set(r, k, c.get(r, k));
                }
            }
        }
        let by_comp = solve_sparse_hungarian(&sp);
        let direct = hungarian(&dense);
        prop_assert_eq!(by_comp.pairs.len(), direct.len());
        let cost: i64 = by_comp.pairs.iter().map(|&(r, k)| c.get(r, k)).sum();
        prop_assert_eq!(cost, total_cost(&dense, &direct));
    }

    /// Chính sách: chi phí không giảm khi ETA tăng (các số hạng khác cố định, trọng số không âm), và cặp vượt giới hạn cứng bị cấm.
    #[test]
    fn policy_is_monotone_in_eta_and_respects_the_hard_limit(
        eta in 0i64..2_000_000,
        extra in 0i64..500_000,
        wait in 0i64..600_000,
        idle in 0i64..600_000,
        p in 0.0f64..1.0,
    ) {
        let pol = CostPolicy {
            risk_ms: 600_000.0,
            age_bonus: 0.5,
            idle_bonus: 0.1,
            soft_pickup_ms: 300_000,
            soft_w: 1.0,
            hard_pickup_ms: Some(900_000),
            ..CostPolicy::eta_only()
        };
        match (pol.cost(eta, wait, idle, p), pol.cost(eta + extra, wait, idle, p)) {
            (Some(a), Some(b)) => prop_assert!(b >= a),
            (_, None) => prop_assert!(eta + extra > 900_000),
            (None, Some(_)) => prop_assert!(false, "ETA lớn hơn mà lại được phép"),
        }
        prop_assert_eq!(pol.cost(900_001, wait, idle, p), None);
    }
}

#[test]
fn h3_candidates_are_sorted_bounded_and_subset() {
    use geo_core::LatLng;
    let ll = |lat: f64, lng: f64| LatLng::try_new(lat, lng).unwrap();
    let drivers: Vec<LatLng> = (0..40)
        .map(|i| {
            ll(
                10.77 + 0.002 * (i % 8) as f64,
                106.69 + 0.002 * (i / 8) as f64,
            )
        })
        .collect();
    let pickups = [ll(10.775, 106.695), ll(10.80, 106.72)];
    let cfg = PruneConfig {
        res: 8,
        k_ring: 1,
        k_nearest: 5,
        k_ring_max: 3,
        leftover: false,
    };
    let c = candidates(&drivers, &pickups, cfg);
    assert_eq!(c.len(), 2);
    for (j, list) in c.iter().enumerate() {
        assert!(list.len() <= 5);
        assert!(list.iter().all(|&i| i < drivers.len()));
        let d: Vec<f64> = list
            .iter()
            .map(|&i| geo_core::haversine_m(drivers[i], pickups[j]))
            .collect();
        assert!(d.windows(2).all(|w| w[0] <= w[1]), "sắp theo khoảng cách");
    }
    assert!(!c[0].is_empty(), "điểm đón gần đàn xe phải có ứng viên");
}
