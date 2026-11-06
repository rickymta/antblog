//! Kiểm thử các điểm cắm P05 của simulator (cửa sổ lô, ghép nối chuyến, gợi ý di chuyển): cộng thêm, không đổi baseline.

use geo_core::haversine_m;
use road_graph::fixture::{compile_fixture, generate};
use route_engine::engine::{Engine, EngineConfig};
use route_engine::position::Position;
use simulator::strategy::{
    Assignment, Ctx, DriverView, FinishingView, PendingView, RepositionSuggestion, Repositioner,
};
use simulator::{
    AtPin, Extras, MatchStrategy, NearestStraightLine, Outcome, RunOptions, Scenario, run, run_with,
};
use std::sync::OnceLock;

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

fn base(sc: &Scenario) -> simulator::SimOutput {
    run(
        engine(),
        sc,
        Box::new(AtPin),
        Box::new(NearestStraightLine),
        RunOptions::default(),
    )
}

/// Mã băm nhật ký chuyến của baseline trên fixture, ghi lại **trước** khi thêm các điểm cắm P05
/// (output `loc30_mo_phong --fixture`: «38 dòng, FNV-1a = 1e0c1a5b30a19582»).
#[test]
fn baseline_is_byte_identical_to_the_one_recorded_before_p05_hooks() {
    let out = base(&scenario());
    assert_eq!(out.trips.len(), 38);
    assert_eq!(format!("{:016x}", fnv(&out)), "1e0c1a5b30a19582");
    assert_eq!(out.extras, simulator::ExtraStats::default());
}

#[test]
fn run_with_default_extras_equals_run() {
    let sc = scenario();
    let a = base(&sc);
    let b = run_with(
        engine(),
        &sc,
        Box::new(AtPin),
        Box::new(NearestStraightLine),
        RunOptions::default(),
        Extras::default(),
    );
    assert_eq!(a.report(), b.report());
    assert_eq!(a.trips_csv(), b.trips_csv());
}

/// Ghép tham lam theo chim bay nhưng gom lô `ms` mili giây.
struct Windowed(u64);
impl MatchStrategy for Windowed {
    fn name(&self) -> &str {
        "windowed"
    }
    fn assign(&mut self, _: &Ctx, p: &[PendingView], d: &[DriverView]) -> Vec<Assignment> {
        simulator::strategy::nearest_straight_line(p, d)
    }
    fn window_ms(&mut self, _: &Ctx, _: &[PendingView], _: &[DriverView]) -> Option<u64> {
        Some(self.0)
    }
}

fn one_outcome_per_request(out: &simulator::SimOutput) {
    let s = &out.summary;
    assert_eq!(out.trips.len(), s.requests);
    assert_eq!(
        s.completed + s.cancelled_after_assign + s.cancelled_no_driver + s.cancelled_no_route,
        s.requests
    );
    let mut ids: Vec<u32> = out.trips.iter().map(|t| t.customer).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), s.requests, "mỗi khách đúng một bản ghi");
}

#[test]
fn batch_window_delays_assignment_and_is_deterministic() {
    let sc = scenario();
    let run_w = |ms| {
        run(
            engine(),
            &sc,
            Box::new(AtPin),
            Box::new(Windowed(ms)),
            RunOptions { trace: true },
        )
    };
    let (a, b) = (run_w(20_000), run_w(20_000));
    assert_eq!(a.trips_csv(), b.trips_csv());
    assert!(a.extras.batches > 0);
    one_outcome_per_request(&a);
    // cửa sổ 20 s: không khách nào được gán trước khi đặt xe 20 s, nên thời gian chờ trung bình dài hơn ghép ngay
    let now = base(&sc);
    assert!(
        a.summary.wait_s.mean >= now.summary.wait_s.mean,
        "{} vs {}",
        a.summary.wait_s.mean,
        now.summary.wait_s.mean
    );
}

/// Chain: khi hết tài xế rảnh, xếp khách cho tài xế sắp trả khách gần nhất (chim bay) trong tầm 10 phút.
struct Chainer;
impl MatchStrategy for Chainer {
    fn name(&self) -> &str {
        "chain-test"
    }
    fn assign(&mut self, _: &Ctx, p: &[PendingView], d: &[DriverView]) -> Vec<Assignment> {
        simulator::strategy::nearest_straight_line(p, d)
    }
    fn chain_horizon_ms(&self) -> Option<u64> {
        Some(600_000)
    }
    fn assign_chain(
        &mut self,
        _: &Ctx,
        p: &[PendingView],
        idle: &[DriverView],
        fin: &[FinishingView],
    ) -> Vec<Assignment> {
        let mut out = simulator::strategy::nearest_straight_line(p, idle);
        let used: Vec<u32> = out.iter().map(|a| a.customer).collect();
        let mut taken = vec![false; fin.len()];
        for pv in p.iter().filter(|x| !used.contains(&x.customer)) {
            let best = fin
                .iter()
                .enumerate()
                .filter(|(i, _)| !taken[*i])
                .min_by(|a, b| {
                    haversine_m(a.1.end_latlng, pv.pickup.stop_point)
                        .total_cmp(&haversine_m(b.1.end_latlng, pv.pickup.stop_point))
                });
            if let Some((i, f)) = best {
                taken[i] = true;
                out.push(Assignment {
                    customer: pv.customer,
                    driver: f.id,
                });
            }
        }
        out
    }
}

#[test]
fn chain_queues_customers_behind_busy_drivers_and_serves_them() {
    let mut sc = scenario();
    sc.drivers = 3;
    sc.cancel.max_prob = 0.0;
    let out = run(
        engine(),
        &sc,
        Box::new(AtPin),
        Box::new(Chainer),
        RunOptions { trace: true },
    );
    one_outcome_per_request(&out);
    assert!(
        out.extras.chained > 0,
        "cần có khách xếp hàng: {:?}",
        out.extras
    );
    // không huỷ nên mọi khách xếp hàng đều được đón; mỗi tài xế chỉ phục vụ một khách tại một thời điểm
    assert_eq!(out.extras.chain_cancelled_in_queue, 0);
    let starts = out.trace.iter().filter(|e| e.kind == "chain_start").count();
    assert_eq!(starts, out.extras.chained);
    for t in out.trips.iter().filter(|t| t.outcome == Outcome::Completed) {
        assert!(t.wait_s >= t.pickup_eta_s - 1e-9);
    }
    // thời gian chờ (từ lúc đặt) của khách xếp hàng gồm cả phần còn lại của chuyến trước, nên pickup_eta_s ≥ 0 và wait_s ≥ eta
    let base_out = base(&sc);
    one_outcome_per_request(&base_out);
}

/// Gợi ý mọi tài xế rảnh về một vị trí cố định (nút đầu của cạnh 0).
struct Everyone(Position);
impl Repositioner for Everyone {
    fn name(&self) -> &str {
        "everyone-to-one-place"
    }
    fn interval_ms(&self) -> u64 {
        60_000
    }
    fn suggest(
        &mut self,
        _: &Ctx,
        idle: &[DriverView],
        _: &[PendingView],
    ) -> Vec<RepositionSuggestion> {
        idle.iter()
            .map(|d| RepositionSuggestion {
                driver: d.id,
                target: self.0,
            })
            .collect()
    }
}

#[test]
fn reposition_moves_idle_drivers_and_counts_cruise_distance() {
    let sc = scenario();
    let target = Position::new(0, 0.5);
    let with = run_with(
        engine(),
        &sc,
        Box::new(AtPin),
        Box::new(NearestStraightLine),
        RunOptions::default(),
        Extras {
            repositioner: Some(Box::new(Everyone(target))),
            reposition_accept_prob: 1.0,
        },
    );
    one_outcome_per_request(&with);
    assert!(with.extras.reposition_accepted > 0);
    assert!(with.extras.cruise_m > 0.0);
    // xác suất nghe theo 0: không ai đi, kết quả y hệt baseline
    let none = run_with(
        engine(),
        &sc,
        Box::new(AtPin),
        Box::new(NearestStraightLine),
        RunOptions::default(),
        Extras {
            repositioner: Some(Box::new(Everyone(target))),
            reposition_accept_prob: 0.0,
        },
    );
    assert_eq!(none.extras.reposition_accepted, 0);
    assert_eq!(none.trips_csv(), base(&sc).trips_csv());
}
