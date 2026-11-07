//! Điểm hẹn động và điểm trả trên fixture: chính sách tĩnh không bao giờ đổi, kết quả tất định, điểm trả cắm vào simulator
//! mà vẫn giữ đủ khách.

use geo_core::haversine_m;
use pickup::dropoff::DropoffPolicy;
use pickup::dynamic::{DynConfig, DynScenario, Kind, Policy, Simulation, consent_probability};
use pickup::objective::ObjectiveWeights;
use pickup::rank::DriverRef;
use pickup::{JConfig, JPickup};
use road_graph::fixture::{compile_fixture, generate};
use route_engine::engine::{Engine, EngineConfig};
use simulator::world::generate as gen_world;
use simulator::{NearestStraightLine, RunOptions, Scenario, run};
use std::sync::OnceLock;

fn engine() -> &'static Engine {
    static E: OnceLock<Engine> = OnceLock::new();
    E.get_or_init(|| Engine::new(compile_fixture(&generate()), EngineConfig::default()))
}

fn scenario() -> Scenario {
    Scenario::from_toml(include_str!("../../../../fixtures/scenario-khu-mau.toml")).unwrap()
}

fn pairs(kind: Kind, k: usize) -> Vec<DynScenario> {
    let e = engine();
    let w = gen_world(e, &scenario());
    let mut out = Vec::new();
    for (i, c) in w.customers.iter().enumerate() {
        let d = &w.drivers[i % w.drivers.len()];
        let p = d.pos;
        if haversine_m(p.point(&e.g), c.view.true_pos) < 400.0 {
            continue;
        }
        out.push(DynScenario {
            cust0: c.view.true_pos,
            pin: c.view.pin,
            dest: c.view.dropoff,
            driver0: DriverRef {
                pos: p,
                latlng: p.point(&e.g),
            },
            kind,
            seed: i as u64,
        });
        if out.len() >= k {
            break;
        }
    }
    out
}

#[test]
fn static_policy_never_changes_the_meeting_point_and_runs_are_deterministic() {
    let e = engine();
    let mut sim = Simulation::new(e, ObjectiveWeights::default(), DynConfig::default());
    for sc in pairs(
        Kind::Both {
            walk: 0.5,
            delay_s: 90.0,
        },
        6,
    ) {
        let a = sim.run(&sc, Policy::Static);
        let b = sim.run(&sc, Policy::Static);
        assert_eq!(a.changes, 0);
        assert_eq!(a.replans, 0);
        assert_eq!(a.t_pick_s.to_bits(), b.t_pick_s.to_bits());
    }
}

#[test]
fn hysteresis_changes_no_more_often_than_naive_under_gps_noise() {
    let e = engine();
    let mut sim = Simulation::new(e, ObjectiveWeights::default(), DynConfig::default());
    let (mut naive, mut hyst) = (0u32, 0u32);
    for sc in pairs(Kind::NoisyGps(12.0), 8) {
        naive += sim.run(&sc, Policy::Naive).changes;
        hyst += sim.run(&sc, Policy::Hysteresis).changes;
    }
    assert!(hyst <= naive, "hysteresis {hyst} > naive {naive}");
}

#[test]
fn consent_probability_is_bounded_and_decreasing() {
    assert!((consent_probability(0.0) - 0.95).abs() < 1e-12);
    assert!(consent_probability(60.0) < consent_probability(20.0));
    assert!(consent_probability(5_000.0) >= 0.05);
}

#[test]
fn dropoff_policy_keeps_every_customer_and_is_deterministic() {
    let sc = scenario();
    let mut cfg = JConfig::new("drop", ObjectiveWeights::default());
    cfg.pickup_j = false;
    cfg.dropoff = Some(DropoffPolicy::default());
    let go = || {
        run(
            engine(),
            &sc,
            Box::new(JPickup::new(cfg.clone())),
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
}
