//! Kiểm thử LOC-48 trên fixture (không mạng): máy trạng thái hysteresis bằng chuỗi tay, và bất biến bằng proptest.

use geo_core::LatLng;
use matcher::deviation::{
    Keep, MatchedObs, MonitorConfig, MonitorEvent, Notifier, NotifyBudget, PlannedRoute,
    RematchDecision, RematchPolicy, RematchState, RouteMonitor, Track,
};
use proptest::prelude::*;
use road_graph::fixture::{compile_fixture, generate};
use road_graph::{EdgeId, Graph};
use std::sync::OnceLock;

fn graph() -> &'static Graph {
    static G: OnceLock<Graph> = OnceLock::new();
    G.get_or_init(|| compile_fixture(&generate()))
}

/// Tuyến mẫu: một đường đi liền mạch của fixture dài ít nhất 6 cạnh (đi ngẫu nhiên tất định, không quay đầu).
fn plan_edges() -> Vec<EdgeId> {
    let g = graph();
    for start in 0..g.node_count() as u32 {
        let mut edges: Vec<EdgeId> = Vec::new();
        let mut node = start;
        while edges.len() < 8 {
            let next = g.out_edges(node).find(|&e| {
                edges.last().is_none_or(|&l| g.twin[l as usize] != e) && !edges.contains(&e)
            });
            let Some(e) = next else { break };
            edges.push(e);
            node = g.head[e as usize];
        }
        if edges.len() >= 6 {
            return edges;
        }
    }
    panic!("fixture không có đường đủ dài");
}

fn point_of(e: EdgeId) -> LatLng {
    let pts = graph().edge_points(e);
    pts[pts.len() / 2]
}

/// Cạnh nằm xa tuyến (cách polyline hơn 100 m) để làm điểm «ngoài tuyến».
fn off_edges(route: &PlannedRoute) -> Vec<EdgeId> {
    let g = graph();
    (0..g.edge_count() as u32)
        .filter(|&e| !route.contains(e) && route.dist_to_m(point_of(e)) > 100.0)
        .collect()
}

fn obs(t_s: i64, e: Option<EdgeId>, pos: LatLng) -> MatchedObs {
    MatchedObs {
        now_ms: t_s * 1000,
        t_ms: t_s * 1000,
        edge: e,
        pos,
    }
}

fn route() -> PlannedRoute {
    PlannedRoute::new(graph(), plan_edges())
}

#[test]
fn needs_k_consecutive_off_points_and_an_isolated_glitch_is_ignored() {
    let r = route();
    let off = off_edges(&r)[0];
    let on = r.edges[2];
    let cfg = MonitorConfig {
        k_off: 3,
        off_dist_m: 30.0,
        ..MonitorConfig::default()
    };
    let mut m = RouteMonitor::new(cfg, r.clone());
    // hai điểm ngoài rồi một điểm trên: bộ đếm về 0, không báo
    for (t, e) in [(0, off), (1, off), (2, on), (3, off), (4, off)] {
        assert_eq!(m.observe(&obs(t, Some(e), point_of(e))), None);
    }
    assert_eq!(m.state(), Track::OnRoute);
    // điểm ngoài thứ ba liên tiếp (t = 5) mới báo
    let ev = m.observe(&obs(5, Some(off), point_of(off)));
    assert!(matches!(
        ev,
        Some(MonitorEvent::Alert {
            off_points: 3,
            first_off_t_ms: 3000,
            ..
        })
    ));
    assert_eq!(m.state(), Track::OffRoute);
    assert_eq!(m.alerts(), 1);
}

#[test]
fn recovery_needs_k_on_points_and_counts_toggles() {
    let r = route();
    let off = off_edges(&r)[0];
    let on = r.edges[1];
    let cfg = MonitorConfig {
        k_off: 1,
        k_on: 3,
        off_dist_m: 30.0,
        ..MonitorConfig::default()
    };
    let mut m = RouteMonitor::new(cfg, r);
    assert!(matches!(
        m.observe(&obs(0, Some(off), point_of(off))),
        Some(MonitorEvent::Alert { .. })
    ));
    // hai điểm trên tuyến rồi một điểm ngoài: chưa tính là quay lại
    for t in 1..=2 {
        assert_eq!(m.observe(&obs(t, Some(on), point_of(on))), None);
    }
    assert_eq!(m.observe(&obs(3, Some(off), point_of(off))), None);
    assert_eq!(m.state(), Track::OffRoute);
    for t in 4..=5 {
        assert_eq!(m.observe(&obs(t, Some(on), point_of(on))), None);
    }
    assert!(matches!(
        m.observe(&obs(6, Some(on), point_of(on))),
        Some(MonitorEvent::Recovered { .. })
    ));
    assert_eq!(m.toggles(), 2);
}

#[test]
fn a_close_parallel_road_is_swallowed_by_the_distance_threshold() {
    let r = route();
    let g = graph();
    // một cạnh không thuộc tuyến nhưng điểm khớp nằm ngay trên tuyến (đường song song sát bên, hoặc chiều ngược)
    let twin_of_plan = g.twin[r.edges[2] as usize];
    assert_ne!(twin_of_plan, road_graph::NONE);
    // điểm khớp cách tim đường của tuyến khoảng 10 m (đường song song sát bên)
    let base = point_of(r.edges[2]);
    let on_street = LatLng::try_new(base.lat() + 0.00009, base.lng()).unwrap();
    assert!(r.dist_to_m(on_street) > 5.0 && r.dist_to_m(on_street) < 30.0);
    let mut m = RouteMonitor::new(MonitorConfig::naive(), r.clone());
    // ngây thơ (ngưỡng 0 m) báo ngay vì cạnh khác tuyến, dù cách tuyến 0 m
    assert!(m.observe(&obs(0, Some(twin_of_plan), on_street)).is_some());
    let mut m = RouteMonitor::new(
        MonitorConfig {
            k_off: 1,
            off_dist_m: 30.0,
            ..MonitorConfig::default()
        },
        r,
    );
    assert_eq!(m.observe(&obs(0, Some(twin_of_plan), on_street)), None);
}

#[test]
fn cooldown_delays_and_budget_caps_alerts() {
    let r = route();
    let off = off_edges(&r)[0];
    let on = r.edges[1];
    let cfg = MonitorConfig {
        k_off: 1,
        k_on: 1,
        off_dist_m: 30.0,
        cooldown_ms: 30_000,
        max_alerts: 2,
    };
    let mut m = RouteMonitor::new(cfg, r);
    let mut alerts = Vec::new();
    // ra/vào tuyến mỗi 5 s trong 120 s
    for t in 0..24 {
        let (e, p) = if t % 2 == 0 {
            (off, point_of(off))
        } else {
            (on, point_of(on))
        };
        if let Some(MonitorEvent::Alert { now_ms, .. }) = m.observe(&obs(t * 5, Some(e), p)) {
            alerts.push(now_ms);
        }
    }
    assert_eq!(alerts.len(), 2, "trần 2 lần báo");
    assert!(alerts[1] - alerts[0] >= 30_000, "{alerts:?}");
}

#[test]
fn unmatched_points_do_not_count_either_way() {
    let r = route();
    let off = off_edges(&r)[0];
    let cfg = MonitorConfig {
        k_off: 2,
        off_dist_m: 30.0,
        ..MonitorConfig::default()
    };
    let mut m = RouteMonitor::new(cfg, r);
    assert_eq!(m.observe(&obs(0, Some(off), point_of(off))), None);
    assert_eq!(m.observe(&obs(1, None, point_of(off))), None);
    assert!(m.observe(&obs(2, Some(off), point_of(off))).is_some());
}

#[test]
fn adopt_resets_state_but_keeps_the_alert_budget() {
    let r = route();
    let off = off_edges(&r)[0];
    let mut m = RouteMonitor::new(
        MonitorConfig {
            k_off: 1,
            ..MonitorConfig::default()
        },
        r.clone(),
    );
    assert!(m.observe(&obs(0, Some(off), point_of(off))).is_some());
    m.adopt(PlannedRoute::new(graph(), vec![off]));
    assert_eq!(m.state(), Track::OnRoute);
    assert_eq!(m.alerts(), 1);
    // trên tuyến mới thì cạnh `off` là trên tuyến
    assert_eq!(m.observe(&obs(1, Some(off), point_of(off))), None);
}

#[test]
fn rematch_requires_worse_eta_margin_distance_and_changes_once() {
    let p = RematchPolicy::guarded();
    let mut st = RematchState::default();
    // ETA không xấu đi so với gốc
    assert_eq!(
        st.decide(&p, 0, 300.0, 330.0, 100.0),
        RematchDecision::Keep(Keep::NotWorse)
    );
    // sắp tới nơi
    assert_eq!(
        st.decide(&p, 0, 40.0, 80.0, 10.0),
        RematchDecision::Keep(Keep::TooCloseToArrival)
    );
    // tài xế khác chỉ nhanh hơn 100 s (< 120 s và < 25% của 600 s = 150 s)
    assert_eq!(
        st.decide(&p, 0, 300.0, 600.0, 500.0),
        RematchDecision::Keep(Keep::MarginTooSmall)
    );
    // nhanh hơn 120 s: chưa đủ biên max(120, 25% × 600 = 150)
    assert_eq!(
        st.decide(&p, 1000, 300.0, 600.0, 480.0),
        RematchDecision::Keep(Keep::MarginTooSmall)
    );
    // nhanh hơn 220 s ≥ 150: đổi
    assert_eq!(
        st.decide(&p, 2000, 300.0, 600.0, 380.0),
        RematchDecision::Switch
    );
    // chỉ một lần đổi mỗi chuyến
    assert_eq!(
        st.decide(&p, 400_000, 300.0, 900.0, 100.0),
        RematchDecision::Keep(Keep::MaxChanges)
    );
    assert_eq!(st.changes(), 1);
}

#[test]
fn notifier_enforces_gap_cap_and_minimum_eta_change() {
    let b = NotifyBudget::default();
    let mut n = Notifier::new(b);
    assert!(n.driver_route_update(0));
    assert!(!n.driver_route_update(10_000), "chưa đủ 30 s");
    assert!(n.driver_route_update(40_000));
    assert!(n.driver_route_update(80_000));
    assert!(!n.driver_route_update(200_000), "quá trần 3");
    assert!(!n.customer_eta_update(0, 20.0), "đổi 20 s < 60 s");
    assert!(n.customer_eta_update(0, -90.0), "độ lớn 90 s");
    assert!(n.customer_eta_update(40_000, 120.0));
    assert!(!n.customer_eta_update(90_000, 300.0), "quá trần 2");
}

// ---------------------------------------------------------------- proptest

#[derive(Debug, Clone)]
struct Step {
    on: bool,
    dt_s: i64,
    unmatched: bool,
}

fn steps() -> impl Strategy<Value = Vec<Step>> {
    proptest::collection::vec(
        (any::<bool>(), 1i64..8, prop::bool::weighted(0.1)).prop_map(|(on, dt_s, unmatched)| {
            Step {
                on,
                dt_s,
                unmatched,
            }
        }),
        1..200,
    )
}

fn config() -> impl Strategy<Value = MonitorConfig> {
    (1usize..6, 0.0f64..60.0, 1usize..5, 0i64..60, 1usize..5).prop_map(
        |(k_off, off_dist_m, k_on, cd_s, max_alerts)| MonitorConfig {
            k_off,
            off_dist_m,
            k_on,
            cooldown_ms: cd_s * 1000,
            max_alerts,
        },
    )
}

proptest! {
    /// Không bao giờ báo quá `max_alerts` lần, hai lần báo không gần nhau hơn cooldown.
    #[test]
    fn alerts_respect_budget_and_cooldown(cfg in config(), seq in steps()) {
        let r = route();
        let offs = off_edges(&r);
        let mut m = RouteMonitor::new(cfg, r.clone());
        let mut t = 0i64;
        let mut alerts = Vec::new();
        for (i, s) in seq.iter().enumerate() {
            t += s.dt_s;
            let e = if s.on { r.edges[i % r.edges.len()] } else { offs[i % offs.len()] };
            let o = obs(t, (!s.unmatched).then_some(e), point_of(e));
            if let Some(MonitorEvent::Alert { now_ms, .. }) = m.observe(&o) {
                alerts.push(now_ms);
            }
        }
        prop_assert!(alerts.len() <= cfg.max_alerts);
        prop_assert_eq!(alerts.len(), m.alerts());
        for w in alerts.windows(2) {
            prop_assert!(w[1] - w[0] >= cfg.cooldown_ms, "{:?} cooldown {}", alerts, cfg.cooldown_ms);
        }
    }

    /// Chuỗi luôn nằm trên tuyến: không báo, không đổi trạng thái.
    #[test]
    fn staying_on_route_never_alerts(cfg in config(), n in 1usize..300, noise_m in 0.0f64..1.0) {
        let r = route();
        let mut m = RouteMonitor::new(cfg, r.clone());
        for i in 0..n {
            let e = r.edges[i % r.edges.len()];
            let mut p = point_of(e);
            // dịch rất nhẹ: cạnh vẫn thuộc tuyến nên luôn là «trên tuyến» bất kể khoảng cách
            p = LatLng::try_new(p.lat() + noise_m * 1e-6, p.lng()).unwrap();
            prop_assert_eq!(m.observe(&obs(i as i64, Some(e), p)), None);
        }
        prop_assert_eq!(m.alerts(), 0);
        prop_assert_eq!(m.toggles(), 0);
        prop_assert_eq!(m.state(), Track::OnRoute);
    }

    /// Chính sách ghép lại không bao giờ đổi quá `max_changes` lần, dù ETA đọc nhảy lung tung.
    #[test]
    fn rematch_never_swaps_more_than_max_changes(
        readings in proptest::collection::vec((0.0f64..2000.0, 0.0f64..2000.0, 0.0f64..2000.0), 1..80),
        max_changes in 1u32..4,
    ) {
        let p = RematchPolicy { max_changes, ..RematchPolicy::guarded() };
        let mut st = RematchState::default();
        let mut switches = 0u32;
        for (i, (orig, cur, alt)) in readings.into_iter().enumerate() {
            if st.decide(&p, i as i64 * 15_000, orig, cur, alt) == RematchDecision::Switch {
                switches += 1;
            }
        }
        prop_assert!(switches <= max_changes);
        prop_assert_eq!(switches, st.changes());
    }

    /// Thông báo không vượt trần và không dày hơn khoảng cách tối thiểu.
    #[test]
    fn notifier_never_exceeds_caps(times in proptest::collection::vec((1i64..60, -300.0f64..300.0), 1..60)) {
        let b = NotifyBudget::default();
        let mut n = Notifier::new(b);
        let (mut t, mut last_d): (i64, Option<i64>) = (0, None);
        for (dt, delta) in times {
            t += dt * 1000;
            if n.driver_route_update(t) {
                if let Some(l) = last_d { prop_assert!(t - l >= b.min_gap_ms); }
                last_d = Some(t);
            }
            n.customer_eta_update(t, delta);
        }
        prop_assert!(n.driver_sent() <= b.max_driver);
        prop_assert!(n.customer_sent() <= b.max_customer);
    }
}
