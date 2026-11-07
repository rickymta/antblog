//! Kiểm thử mô phỏng trên fixture: tất định từng byte, bảo toàn số khách, vị trí xe tiến dọc tuyến, chiến lược cắm vào được.

use road_graph::fixture::{compile_fixture, generate};
use route_engine::engine::{Engine, EngineConfig};
use route_engine::turn::{Goal, Start};
use simulator::strategy::{Assignment, Ctx, CustomerView, DriverView, PendingView, PickupChoice};
use simulator::world::{Router, generate as gen_world};
use simulator::{
    AtPin, MatchStrategy, NearestStraightLine, Outcome, PickupStrategy, RunOptions, Scenario, run,
};
use std::sync::OnceLock;

fn engine() -> &'static Engine {
    static E: OnceLock<Engine> = OnceLock::new();
    E.get_or_init(|| Engine::new(compile_fixture(&generate()), EngineConfig::default()))
}

fn scenario() -> Scenario {
    Scenario::from_toml(include_str!("../../../../fixtures/scenario-khu-mau.toml")).unwrap()
}

fn baseline(sc: &Scenario, trace: bool) -> simulator::SimOutput {
    run(
        engine(),
        sc,
        Box::new(AtPin),
        Box::new(NearestStraightLine),
        RunOptions { trace },
    )
}

#[test]
fn same_seed_gives_byte_identical_output() {
    let sc = scenario();
    let (a, b) = (baseline(&sc, true), baseline(&sc, true));
    assert_eq!(a.report(), b.report());
    assert_eq!(a.trips_csv(), b.trips_csv());
    assert_eq!(a.trace_csv(), b.trace_csv());
    assert_eq!(a.summary_json(), b.summary_json());
    assert!(
        a.summary.completed > 5,
        "kịch bản phải có chuyến hoàn thành: {}",
        a.report()
    );
}

#[test]
fn another_seed_gives_a_different_run() {
    let sc = scenario();
    let mut other = sc.clone();
    other.seed += 1;
    assert_ne!(
        baseline(&sc, false).trips_csv(),
        baseline(&other, false).trips_csv()
    );
}

#[test]
fn trace_does_not_change_the_numbers() {
    let sc = scenario();
    assert_eq!(
        baseline(&sc, false).trips_csv(),
        baseline(&sc, true).trips_csv()
    );
}

#[test]
fn every_request_ends_in_exactly_one_outcome() {
    let sc = scenario();
    let out = baseline(&sc, false);
    let s = &out.summary;
    assert_eq!(out.trips.len(), s.requests);
    assert_eq!(
        s.completed + s.cancelled_after_assign + s.cancelled_no_driver + s.cancelled_no_route,
        s.requests
    );
    let mut ids: Vec<u32> = out.trips.iter().map(|t| t.customer).collect();
    ids.dedup();
    assert_eq!(ids.len(), s.requests, "mỗi khách đúng một bản ghi");
    for t in out.trips.iter().filter(|t| t.outcome == Outcome::Completed) {
        assert!(t.pickup_eta_s >= 0.0 && t.wait_s >= t.pickup_eta_s - 1e-9);
        assert!(t.meet_gap_s >= 0.0 && t.deadhead_m >= 0.0 && t.trip_m > 0.0 && t.trip_s > 0.0);
        assert!(t.walk_m >= 0.0 && t.driver.is_some());
    }
    assert!((0.0..=1.0).contains(&s.cancel_rate));
    assert!(
        (0.0..=1.0).contains(&s.driver_busy_share),
        "{}",
        s.driver_busy_share
    );
}

#[test]
fn drivers_advance_along_the_route_monotonically() {
    use simulator::sim::ActiveRoute;
    let (e, sc) = (engine(), scenario());
    let w = gen_world(e, &sc);
    let router = Router::new(e, &sc);
    let (d, c) = (&w.drivers[0], &w.customers[0]);
    let r = router
        .route_car(0, Start::Pos(d.pos), Goal::Pos(c.dropoff[0]))
        .expect("có đường");
    let tm = router.turn_model(0);
    let a = ActiveRoute::build(e, &tm, &r, 10_000, 1.0);
    // tại t0 ở điểm xuất phát, tại cuối ở điểm đích
    let start = a.position_at(10_000);
    assert_eq!(start.edge, d.pos.edge);
    assert!((start.frac - d.pos.frac).abs() < 1e-4);
    let end = a.position_at(10_000 + a.total_ms + 5_000);
    assert_eq!(end, a.end_position());
    // quãng đường đi được không giảm, và tới đúng tổng độ dài ở cuối
    let mut last = -1.0;
    for k in 0..=50u64 {
        let t = 10_000 + a.total_ms * k / 50;
        let dist = a.distance_until(t);
        assert!(dist >= last - 1e-6, "k={k}: {dist} < {last}");
        last = dist;
    }
    assert!(
        (last - a.length_m()).abs() < 1.0,
        "{last} vs {}",
        a.length_m()
    );
    // thời gian thực tế tỉ lệ với nhiễu
    let slow = ActiveRoute::build(e, &tm, &r, 0, 1.5);
    assert!(
        (slow.total_ms as f64 - 1.5 * a.total_ms as f64).abs() <= 2.0 + 0.001 * a.total_ms as f64
    );
}

struct NeverAssign;
impl MatchStrategy for NeverAssign {
    fn name(&self) -> &str {
        "never"
    }
    fn assign(&mut self, _: &Ctx, _: &[PendingView], _: &[DriverView]) -> Vec<Assignment> {
        Vec::new()
    }
}

struct NoPickup;
impl PickupStrategy for NoPickup {
    fn name(&self) -> &str {
        "none"
    }
    fn choose(&mut self, _: &Ctx, _: &CustomerView) -> Option<PickupChoice> {
        None
    }
}

#[test]
fn strategies_plug_in_without_touching_the_simulator() {
    let sc = scenario();
    let out = run(
        engine(),
        &sc,
        Box::new(AtPin),
        Box::new(NeverAssign),
        RunOptions::default(),
    );
    assert_eq!(out.match_strategy, "never");
    assert_eq!(out.summary.completed, 0);
    assert_eq!(out.summary.cancelled_no_driver, out.summary.requests);
    assert!((out.summary.cancel_rate - 1.0).abs() < 1e-12);

    let out = run(
        engine(),
        &sc,
        Box::new(NoPickup),
        Box::new(NearestStraightLine),
        RunOptions::default(),
    );
    assert_eq!(out.pickup_strategy, "none");
    assert_eq!(out.summary.cancelled_no_route, out.summary.requests);
}

#[test]
fn cancel_model_off_means_no_cancellations_when_drivers_are_plentiful() {
    let mut sc = scenario();
    sc.cancel.max_prob = 0.0;
    sc.drivers = 60;
    let out = baseline(&sc, false);
    assert_eq!(out.summary.cancelled_after_assign, 0);
    assert_eq!(out.summary.cancelled_no_driver, 0);
}

#[test]
fn more_drivers_never_lengthen_the_median_pickup_eta_much() {
    let mut sc = scenario();
    sc.cancel.max_prob = 0.0;
    sc.drivers = 4;
    let few = baseline(&sc, false);
    sc.drivers = 40;
    let many = baseline(&sc, false);
    assert!(
        many.summary.pickup_eta_s.p50 <= few.summary.pickup_eta_s.p50 + 1.0,
        "{} vs {}",
        many.summary.pickup_eta_s.p50,
        few.summary.pickup_eta_s.p50
    );
}

#[test]
fn synthetic_peak_traffic_makes_trips_slower_than_free_flow() {
    use simulator::scenario::Traffic;
    let mut free = scenario();
    free.cancel.max_prob = 0.0;
    free.driver.speed_jitter = 0.0;
    free.drivers = 40;
    let mut peak = free.clone();
    peak.traffic = Traffic::Synthetic;
    peak.start_hour = 8;
    peak.traffic_radius_m = Some(2_000.0); // phủ cả khu mẫu
    let (a, b) = (baseline(&free, false), baseline(&peak, false));
    assert_eq!(
        a.summary.completed, b.summary.completed,
        "cùng khách, cùng tài xế rảnh nên cùng số chuyến hoàn thành"
    );
    let mean_trip_s = |o: &simulator::SimOutput| {
        let t: Vec<f64> = o
            .trips
            .iter()
            .filter(|t| t.outcome == Outcome::Completed)
            .map(|t| t.trip_s)
            .collect();
        t.iter().sum::<f64>() / t.len() as f64
    };
    assert!(
        mean_trip_s(&b) > mean_trip_s(&a),
        "giờ cao điểm tổng hợp phải chậm hơn: {} vs {}",
        mean_trip_s(&b),
        mean_trip_s(&a)
    );
}
