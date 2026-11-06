//! Kiểm thử các điểm cắm P06 của simulator (`Truth`, `Observer`, `run_hooked`): cộng thêm, không đổi baseline.

use road_graph::EdgeId;
use road_graph::fixture::{compile_fixture, generate};
use route_engine::engine::{Engine, EngineConfig};
use simulator::{
    AtPin, Extras, Hooks, LegInfo, LegKind, NearestStraightLine, Observer, Outcome, RunOptions,
    Scenario, Truth, run, run_hooked,
};
use std::sync::{Arc, Mutex, OnceLock};

fn engine() -> &'static Engine {
    static E: OnceLock<Engine> = OnceLock::new();
    E.get_or_init(|| Engine::new(compile_fixture(&generate()), EngineConfig::default()))
}

fn scenario() -> Scenario {
    Scenario::from_toml(include_str!("../../../../fixtures/scenario-khu-mau.toml")).unwrap()
}

fn fnv(out: &simulator::SimOutput) -> u64 {
    simulator::metrics::fnv1a(out.trips_csv().as_bytes())
}

fn hooked(sc: &Scenario, hooks: Hooks<'_>) -> simulator::SimOutput {
    run_hooked(
        engine(),
        sc,
        Box::new(AtPin),
        Box::new(NearestStraightLine),
        RunOptions::default(),
        Extras::default(),
        hooks,
    )
}

struct Const(f64);
impl Truth for Const {
    fn edge_factor(&self, _: &Engine, _: EdgeId, _: u64) -> f64 {
        self.0
    }
}

#[derive(Default)]
struct Log {
    legs: Vec<(LegKind, u32, u64, u64, u64, u64)>,
    cancelled: Vec<(u32, u64)>,
}
struct Watch(Arc<Mutex<Log>>);
impl Observer for Watch {
    fn leg_started(&mut self, leg: &LegInfo) {
        self.0.lock().unwrap().legs.push((
            leg.kind,
            leg.customer,
            leg.start_ms,
            leg.route.planned_ms,
            leg.route.total_ms,
            leg.route.noiseless_ms,
        ));
    }
    fn leg_cancelled(&mut self, customer: u32, at_ms: u64) {
        self.0.lock().unwrap().cancelled.push((customer, at_ms));
    }
}

#[test]
fn default_hooks_and_unit_truth_leave_the_baseline_byte_identical() {
    let sc = scenario();
    let base = run(
        engine(),
        &sc,
        Box::new(AtPin),
        Box::new(NearestStraightLine),
        RunOptions::default(),
    );
    // mã băm đã ghi trước khi thêm điểm cắm P05 và P06 (xem p05_hooks.rs)
    assert_eq!(format!("{:016x}", fnv(&base)), "1e0c1a5b30a19582");
    let a = hooked(&sc, Hooks::default());
    assert_eq!(a.report(), base.report());
    let b = hooked(
        &sc,
        Hooks {
            truth: Some(Box::new(Const(1.0))),
            observer: None,
        },
    );
    assert_eq!(fnv(&b), fnv(&base), "hệ số 1,0 không được đổi một byte");
}

#[test]
fn truth_slows_the_world_but_not_what_the_engine_believes() {
    let sc = scenario();
    let log = Arc::new(Mutex::new(Log::default()));
    let slow = hooked(
        &sc,
        Hooks {
            truth: Some(Box::new(Const(1.5))),
            observer: Some(Box::new(Watch(Arc::clone(&log)))),
        },
    );
    let fast = hooked(&sc, Hooks::default());
    assert!(slow.summary.completed > 0 && fast.summary.completed > 0);
    let log = log.lock().unwrap();
    let completed = slow
        .trips
        .iter()
        .filter(|t| t.outcome == Outcome::Completed)
        .count();
    let onboard = log
        .legs
        .iter()
        .filter(|l| l.0 == LegKind::Onboard)
        .count();
    assert_eq!(onboard, completed, "mỗi chuyến hoàn thành có đúng một chặng có khách");
    assert!(log.legs.iter().any(|l| l.0 == LegKind::Pickup));
    for &(_, _, _, planned, _, noiseless) in &log.legs {
        // thời gian «thật không nhiễu» = 1,5 × ước lượng của route-engine (sai số làm tròn từng cạnh)
        let ratio = noiseless as f64 / planned.max(1) as f64;
        assert!((ratio - 1.5).abs() < 0.01, "ratio {ratio}");
    }
    assert!(slow.summary.trip_m.n > 0);
    // chạy chậm hơn thì thời gian chặng có khách dài hơn trung bình
    let mean_s = |o: &simulator::SimOutput| {
        let v: Vec<f64> = o
            .trips
            .iter()
            .filter(|t| t.outcome == Outcome::Completed)
            .map(|t| t.trip_s)
            .collect();
        v.iter().sum::<f64>() / v.len() as f64
    };
    assert!(mean_s(&slow) > 1.2 * mean_s(&fast));
}
