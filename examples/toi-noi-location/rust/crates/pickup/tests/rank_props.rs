//! Bất biến của bộ xếp hạng trên đồ thị fixture (không cần mạng, không cần bản đồ thật):
//! kết quả không bao giờ tệ hơn điểm ghim theo `J`, ứng viên hợp lệ, tất định.

use geo_core::{LatLng, destination};
use pickup::candidates::stop_violation;
use pickup::objective::ObjectiveWeights;
use pickup::rank::{DriverBelief, DriverRef, RankConfig, Ranker, Request};
use proptest::prelude::*;
use road_graph::fixture::{compile_fixture, generate};
use route_engine::engine::{Engine, EngineConfig};
use route_engine::position::Position;
use route_engine::profile::Profile;
use route_engine::turn::{EdgeSearcher, TurnCosts};
use std::cell::RefCell;
use std::sync::OnceLock;

fn engine() -> &'static Engine {
    static E: OnceLock<Engine> = OnceLock::new();
    E.get_or_init(|| Engine::new(compile_fixture(&generate()), EngineConfig::default()))
}

fn drivers(e: &Engine, seeds: &[u64]) -> Vec<DriverRef> {
    let m = e.g.edge_count() as u64;
    let view = e.view(Profile::Car);
    seeds
        .iter()
        .filter_map(|s| {
            let edge = (s % m) as u32;
            (view.w[edge as usize] != route_engine::INF).then(|| {
                let pos = Position::new(edge, 0.4);
                DriverRef {
                    pos,
                    latlng: pos.point(&e.g),
                }
            })
        })
        .collect()
}

fn rank_one(ranker: &mut Ranker, req: &Request, belief: &DriverBelief) -> pickup::rank::Ranked {
    let e = engine();
    let tm = e.turn_model(Profile::Car, TurnCosts::urban());
    let vmax = e.costs(Profile::Car).vmax_cm_per_unit;
    let es = RefCell::new(EdgeSearcher::new(e.g.edge_count()));
    let mut route = |f, t| es.borrow_mut().route(&tm, &e.xy, vmax, f, t, true).0;
    ranker.rank(e, req, belief, &mut route)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn result_is_never_worse_than_the_pin_under_j(
        bearing in 0.0f64..360.0, dist in 0.0f64..400.0,
        pin_noise in 0.0f64..40.0, pin_bearing in 0.0f64..360.0,
        dest_bearing in 0.0f64..360.0, dest_dist in 600.0f64..1500.0,
        dseeds in proptest::collection::vec(0u64..100_000, 0..6),
        stage2 in any::<bool>(),
    ) {
        let e = engine();
        let c = e.g.center();
        let from = destination(c, bearing, dist);
        let pin = destination(from, pin_bearing, pin_noise);
        let dest = destination(from, dest_bearing, dest_dist);
        let cfg = RankConfig { stage2, ..RankConfig::default() };
        let mut ranker = Ranker::new(e, ObjectiveWeights::default(), cfg);
        let ds = drivers(e, &dseeds);
        let belief = if ds.is_empty() { DriverBelief::Prior { n: 8, radius_m: 800.0 } } else { DriverBelief::Live(ds) };
        let r = rank_one(&mut ranker, &Request { from, pin, dest }, &belief);
        if let (Some(best), Some(pin_s)) = (r.top.first(), r.pin)
            && pin_s.cand.violation.is_none()
            && !r.stats.pin_forced
            && pin_s.j.is_finite()
        {
            prop_assert!(best.j <= pin_s.j + 1e-9, "best {} > pin {}", best.j, pin_s.j);
        }
        // J không giảm dọc danh sách, vị trí khác nhau
        for w in r.top.windows(2) {
            prop_assert!(w[0].j <= w[1].j + 1e-9);
            prop_assert!(geo_core::haversine_m(w[0].cand.point, w[1].cand.point) >= 1.0);
        }
        // ứng viên không phải ghim luôn qua ràng buộc cứng và nằm trong ngưỡng đi bộ ước lượng
        for s in &r.top {
            if !s.cand.is_pin {
                prop_assert!(stop_violation(e, s.cand.pos.edge, 0.0).is_none());
                prop_assert!(s.cand.walk_est_m <= 250.0 + 1e-6);
            }
            prop_assert!(s.terms.walk_s >= 0.0 && s.terms.penalty_s >= 0.0);
        }
        if stage2 && !r.top.is_empty() {
            prop_assert!(r.top[0].exact);
        }
    }

    #[test]
    fn ranking_is_deterministic(bearing in 0.0f64..360.0, dist in 0.0f64..300.0, dseed in 0u64..100_000) {
        let e = engine();
        let from = destination(e.g.center(), bearing, dist);
        let req = Request { from, pin: from, dest: destination(from, 45.0, 900.0) };
        let belief = DriverBelief::Live(drivers(e, &[dseed, dseed + 7]));
        let mut r1 = Ranker::new(e, ObjectiveWeights::default(), RankConfig::default());
        let mut r2 = Ranker::new(e, ObjectiveWeights::default(), RankConfig::default());
        let (a, b) = (rank_one(&mut r1, &req, &belief), rank_one(&mut r2, &req, &belief));
        let key = |r: &pickup::rank::Ranked| r.top.iter().map(|s| (s.cand.pos.edge, s.j.to_bits())).collect::<Vec<_>>();
        prop_assert_eq!(key(&a), key(&b));
    }
}

#[test]
fn hard_constraint_excludes_where_penalty_would_only_discourage() {
    // ví dụ tay: cùng một vị trí, cấm dừng làm ứng viên biến mất, phạt phía đường chỉ làm J lớn hơn
    let e = engine();
    let c: LatLng = e.g.center();
    let mut ranker = Ranker::new(e, ObjectiveWeights::default(), RankConfig::default());
    let req = Request {
        from: c,
        pin: c,
        dest: destination(c, 60.0, 1000.0),
    };
    let r = rank_one(&mut ranker, &req, &DriverBelief::Unknown);
    assert!(!r.all.is_empty());
    for s in &r.all {
        if !s.cand.is_pin {
            assert!(stop_violation(e, s.cand.pos.edge, 20.0).is_none());
        }
    }
}
