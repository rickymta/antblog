//! Cắm chiến lược của `matcher` vào simulator trên fixture: tất định từng byte, mỗi khách đúng một kết cục, chain, cửa sổ lô,
//! gợi ý di chuyển và chống bầy đàn. Không dùng mạng.

use geo_core::h3::{CellIndex, cell_of, resolution};
use matcher::batch::{AdaptiveWindow, Algo, BatchMatcher, ChainConfig, MatcherConfig, Window};
use matcher::policy::CostPolicy;
use matcher::prune::PruneConfig;
use matcher::reposition::{DemandReposition, DemandTable, Mode, RepoConfig};
use road_graph::fixture::{compile_fixture, generate};
use route_engine::engine::{Engine, EngineConfig};
use rustc_hash::FxHashMap;
use simulator::world::generate as gen_world;
use simulator::{
    AtPin, Extras, NearestStraightLine, RunOptions, Scenario, SimOutput, run, run_with,
};
use std::sync::OnceLock;

fn engine() -> &'static Engine {
    static E: OnceLock<Engine> = OnceLock::new();
    E.get_or_init(|| Engine::new(compile_fixture(&generate()), EngineConfig::default()))
}

fn scenario() -> Scenario {
    Scenario::from_toml(include_str!("../../../../fixtures/scenario-khu-mau.toml")).unwrap()
}

fn with(cfg: MatcherConfig, sc: &Scenario) -> SimOutput {
    let (m, _) = BatchMatcher::new(cfg);
    run(
        engine(),
        sc,
        Box::new(AtPin),
        Box::new(m),
        RunOptions { trace: true },
    )
}

fn one_outcome_each(o: &SimOutput) {
    let s = &o.summary;
    assert_eq!(o.trips.len(), s.requests);
    assert_eq!(
        s.completed + s.cancelled_after_assign + s.cancelled_no_driver + s.cancelled_no_route,
        s.requests
    );
}

fn all_configs() -> Vec<MatcherConfig> {
    let mut v = vec![
        MatcherConfig::nearest_eta(),
        MatcherConfig::new("h-immediate", Algo::Hungarian, Window::Immediate),
        MatcherConfig::new("g-w10", Algo::FifoGreedy, Window::Fixed(10_000)),
        MatcherConfig::new("h-w10", Algo::Hungarian, Window::Fixed(10_000)),
        MatcherConfig::new("gg-w5", Algo::GlobalGreedy, Window::Fixed(5_000)),
        MatcherConfig::new(
            "h-adaptive",
            Algo::Hungarian,
            Window::Adaptive(AdaptiveWindow {
                k: 2.0,
                min_ms: 2_000,
                max_ms: 20_000,
                horizon_s: 300.0,
            }),
        ),
    ];
    let mut pr = MatcherConfig::new("h-pruned", Algo::Hungarian, Window::Fixed(10_000));
    pr.prune = Some(PruneConfig {
        res: 8,
        k_ring: 1,
        k_nearest: 3,
        k_ring_max: 4,
        leftover: true,
    });
    v.push(pr);
    let mut pol = MatcherConfig::new("h-policy", Algo::Hungarian, Window::Immediate);
    pol.policy = CostPolicy {
        risk_ms: 600_000.0,
        age_bonus: 0.5,
        idle_bonus: 0.1,
        soft_pickup_ms: 120_000,
        soft_w: 1.0,
        hard_pickup_ms: Some(900_000),
        ..CostPolicy::eta_only()
    };
    v.push(pol);
    let mut ch = MatcherConfig::new("h-chain", Algo::Hungarian, Window::Immediate);
    ch.chain = Some(ChainConfig {
        horizon_ms: 120_000,
        risk_k: 1.0,
    });
    v.push(ch);
    v
}

#[test]
fn every_strategy_is_deterministic_and_every_request_has_one_outcome() {
    let mut sc = scenario();
    sc.drivers = 6; // đủ khan hiếm để lô có nhiều hơn một khách
    for cfg in all_configs() {
        let (a, b) = (with(cfg.clone(), &sc), with(cfg.clone(), &sc));
        assert_eq!(a.trips_csv(), b.trips_csv(), "{} không tất định", cfg.name);
        assert_eq!(a.trace_csv(), b.trace_csv());
        one_outcome_each(&a);
        for t in &a.trips {
            // mỗi tài xế chỉ có một chuyến tại một thời điểm: kiểm gián tiếp qua nhật ký (assign/chain_start ↔ trip_ends/cancel)
            assert!(t.deadhead_m >= 0.0);
        }
    }
}

#[test]
fn a_driver_never_works_two_customers_at_once() {
    let mut sc = scenario();
    sc.drivers = 4;
    for cfg in all_configs() {
        let out = with(cfg.clone(), &sc);
        let mut busy = vec![false; sc.drivers];
        for e in &out.trace {
            let Some(d) = e.driver.map(|d| d as usize) else {
                continue;
            };
            match e.kind {
                "assign" | "chain_start" => {
                    assert!(!busy[d], "{}: tài xế {d} bị gán khi đang bận", cfg.name);
                    busy[d] = true;
                }
                "trip_ends" | "cancel" => busy[d] = false,
                _ => {}
            }
        }
    }
}

#[test]
fn nearest_eta_with_no_window_is_a_valid_drop_in_for_the_baseline() {
    let sc = scenario();
    let base = run(
        engine(),
        &sc,
        Box::new(AtPin),
        Box::new(NearestStraightLine),
        RunOptions::default(),
    );
    let eta = with(MatcherConfig::nearest_eta(), &sc);
    assert_eq!(
        base.summary.requests, eta.summary.requests,
        "cùng dãy khách"
    );
    one_outcome_each(&eta);
}

#[test]
fn chain_serves_customers_that_would_otherwise_wait() {
    let mut sc = scenario();
    sc.drivers = 3;
    sc.cancel.max_prob = 0.0;
    let mut cfg = MatcherConfig::new("h-chain", Algo::Hungarian, Window::Immediate);
    cfg.chain = Some(ChainConfig {
        horizon_ms: 600_000,
        risk_k: 0.0,
    });
    let out = with(cfg, &sc);
    assert!(out.extras.chained > 0, "{:?}", out.extras);
    one_outcome_each(&out);
}

fn table_from_world(sc: &Scenario) -> DemandTable {
    let res = resolution(8);
    let w = gen_world(engine(), sc);
    let mut m: FxHashMap<CellIndex, f64> = FxHashMap::default();
    for c in &w.customers {
        *m.entry(cell_of(c.view.pin, res)).or_default() += 1.0;
    }
    DemandTable::from_per_hour(engine(), 8, &m)
}

fn repo_run(
    sc: &Scenario,
    table: &DemandTable,
    mode: Mode,
) -> (SimOutput, matcher::reposition::RepoStats) {
    let (m, _) = BatchMatcher::new(MatcherConfig::new(
        "h-immediate",
        Algo::Hungarian,
        Window::Immediate,
    ));
    let (rep, h) = DemandReposition::new(
        "t",
        table.clone(),
        RepoConfig {
            mode,
            interval_ms: 60_000,
            ..RepoConfig::default()
        },
    );
    let out = run_with(
        engine(),
        sc,
        Box::new(AtPin),
        Box::new(m),
        RunOptions { trace: true },
        Extras {
            repositioner: Some(Box::new(rep)),
            reposition_accept_prob: 1.0,
        },
    );
    let st = h.lock().unwrap().clone();
    (out, st)
}

/// Kiểm thử chống bầy đàn: nếu mọi tài xế đều được gợi ý về ô nóng nhất thì tập trung gần tuyệt đối; luật lợi kỳ vọng giảm dần thì không.
#[test]
fn everyone_to_one_hotspot_is_herding_and_the_marginal_rule_avoids_it() {
    let mut sc = scenario();
    sc.drivers = 30;
    let table = table_from_world(&sc);
    assert!(table.cells.len() >= 2, "bảng nhu cầu phải có từ hai ô");
    let (herd_out, herd) = repo_run(&sc, &table, Mode::HerdToTop);
    let (marg_out, marg) = repo_run(&sc, &table, Mode::Marginal);
    one_outcome_each(&herd_out);
    one_outcome_each(&marg_out);
    assert!(herd.suggested() > 0 && marg.suggested() > 0);
    // bầy đàn: một ô nhận gần như mọi gợi ý
    assert!(herd.top_cell_share() > 0.99, "{herd:?}");
    // luật chống bầy đàn: không nhiều xe hơn về cùng một ô trong một lần gọi, và đích đến đều hơn
    assert!(marg.max_burst() <= herd.max_burst(), "{marg:?} vs {herd:?}");
    assert!(marg.hhi() <= herd.hhi() + 1e-12);
    assert!(marg.top_cell_share() <= herd.top_cell_share());
}

#[test]
fn zero_compliance_means_the_baseline_even_with_a_repositioner() {
    let sc = scenario();
    let table = table_from_world(&sc);
    let (m, _) = BatchMatcher::new(MatcherConfig::nearest_eta());
    let (rep, _) = DemandReposition::new("t", table.clone(), RepoConfig::default());
    let with_rep = run_with(
        engine(),
        &sc,
        Box::new(AtPin),
        Box::new(m),
        RunOptions::default(),
        Extras {
            repositioner: Some(Box::new(rep)),
            reposition_accept_prob: 0.0,
        },
    );
    let (m2, _) = BatchMatcher::new(MatcherConfig::nearest_eta());
    let without = run(
        engine(),
        &sc,
        Box::new(AtPin),
        Box::new(m2),
        RunOptions::default(),
    );
    assert_eq!(with_rep.trips_csv(), without.trips_csv());
}
