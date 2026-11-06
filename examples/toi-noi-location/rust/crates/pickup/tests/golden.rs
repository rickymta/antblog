//! Chiến lược P04 cắm vào simulator mà không làm đổi baseline: cùng kịch bản, cùng seed ra đúng từng byte.

use pickup::dropoff::{BaselineLogged, DropLog};
use pickup::{JConfig, JPickup, ObjectiveWeights};
use road_graph::fixture::{compile_fixture, generate};
use route_engine::engine::{Engine, EngineConfig};
use simulator::metrics::fnv1a;
use simulator::{AtPin, NearestStraightLine, RunOptions, Scenario, run};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;

fn engine() -> &'static Engine {
    static E: OnceLock<Engine> = OnceLock::new();
    E.get_or_init(|| Engine::new(compile_fixture(&generate()), EngineConfig::default()))
}

fn scenario() -> Scenario {
    Scenario::from_toml(include_str!("../../../../fixtures/scenario-khu-mau.toml")).unwrap()
}

fn hash(out: &simulator::SimOutput) -> u64 {
    fnv1a(out.trips_csv().as_bytes())
}

/// Mã băm nhật ký chuyến của baseline trên fixture, ghi lúc thêm các điểm cắm P04. Đổi con số này nghĩa là baseline đã đổi.
const BASELINE_FIXTURE_FNV: u64 = 0x1e0c_1a5b_30a1_9582;

#[test]
fn baseline_is_byte_identical_after_the_p04_hooks() {
    let sc = scenario();
    let out = run(
        engine(),
        &sc,
        Box::new(AtPin),
        Box::new(NearestStraightLine),
        RunOptions::default(),
    );
    assert_eq!(
        hash(&out),
        BASELINE_FIXTURE_FNV,
        "baseline đã đổi: {:016x}",
        hash(&out)
    );
}

#[test]
fn logged_baseline_changes_nothing() {
    let sc = scenario();
    let plain = run(
        engine(),
        &sc,
        Box::new(AtPin),
        Box::new(NearestStraightLine),
        RunOptions::default(),
    );
    let log = Rc::new(RefCell::new(DropLog::default()));
    let logged = run(
        engine(),
        &sc,
        Box::new(BaselineLogged {
            log: log.clone(),
            gate_radius_m: 80.0,
        }),
        Box::new(NearestStraightLine),
        RunOptions::default(),
    );
    assert_eq!(plain.trips_csv(), logged.trips_csv());
    assert_eq!(log.borrow().entries.len(), plain.summary.completed);
}

#[test]
fn j_strategy_is_deterministic_and_conserves_customers() {
    let sc = scenario();
    let go = || {
        run(
            engine(),
            &sc,
            Box::new(JPickup::new(JConfig::new("j", ObjectiveWeights::default()))),
            Box::new(NearestStraightLine),
            RunOptions::default(),
        )
    };
    let (a, b) = (go(), go());
    assert_eq!(a.trips_csv(), b.trips_csv());
    let s = &a.summary;
    assert_eq!(
        s.completed + s.cancelled_after_assign + s.cancelled_no_driver + s.cancelled_no_route,
        s.requests
    );
    assert!(s.completed > 5);
}

#[test]
fn walking_dominated_weights_never_walk_more_than_baseline_on_average() {
    // với a rất lớn, J chỉ còn quan tâm đi bộ: quãng đi bộ trung bình không thể lớn hơn baseline (ghim)
    let sc = scenario();
    let base = run(
        engine(),
        &sc,
        Box::new(AtPin),
        Box::new(NearestStraightLine),
        RunOptions::default(),
    );
    let w = ObjectiveWeights::default().with_ratio(1e6);
    let out = run(
        engine(),
        &sc,
        Box::new(JPickup::new(JConfig::new("walk-first", w))),
        Box::new(NearestStraightLine),
        RunOptions::default(),
    );
    assert!(
        out.summary.walk_m.mean <= base.summary.walk_m.mean + 5.0,
        "{} > {}",
        out.summary.walk_m.mean,
        base.summary.walk_m.mean
    );
}
