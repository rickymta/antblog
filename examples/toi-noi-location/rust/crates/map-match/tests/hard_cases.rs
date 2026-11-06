//! Bám đường HMM phải thắng snap về cạnh gần nhất trên các ca khó của fixture (cầu vượt, hai chiều đại lộ có dải phân cách),
//! trên vết tổng hợp có seed cố định. Không cần mạng. Đây là kiểm tra hồi quy, không phải chứng minh tổng quát:
//! nhiễu do chính test sinh ra và các ngưỡng được đặt sau khi xem một lần chạy, rộng hơn kết quả đo để không dễ vỡ.

use map_match::eval::{Score, nearest_snap, score_trace};
use map_match::hmm::{MatchConfig, MatchCtx};
use map_match::online::{Lag, OnlineMatcher};
use map_match::route::Router;
use map_match::scenario::{Catalog, InEdges, Kind, Scenario, Split, build, realize};
use map_match::trace::{DriveConfig, NoiseConfig};
use road_graph::fixture::{compile_fixture, generate};
use road_graph::snap::SnapIndex;
use road_graph::{Graph, Mode};

struct Env {
    g: Graph,
    idx: SnapIndex,
    scenarios: Vec<Scenario>,
}

fn env() -> Env {
    let g = compile_fixture(&generate());
    let idx = SnapIndex::build(&g);
    let cat = Catalog::new(&g, &idx, Mode::Car);
    let rev = InEdges::new(&g);
    let mut router = Router::new(&g);
    let scenarios = build(&g, &cat, &rev, &mut router, Mode::Car, Split::All, 8, 7);
    Env { g, idx, scenarios }
}

fn noise() -> NoiseConfig {
    NoiseConfig {
        sigma_m: 6.0,
        ..NoiseConfig::default()
    }
}

#[test]
fn hmm_beats_nearest_snap_on_fixture_hard_cases() {
    let e = env();
    let hard: Vec<&Scenario> = e
        .scenarios
        .iter()
        .filter(|s| matches!(s.kind, Kind::Overpass | Kind::Underpass | Kind::Parallel))
        .collect();
    assert!(
        hard.len() >= 3,
        "fixture phải cho ít nhất 3 ca khó, có {}",
        hard.len()
    );
    let (mut snap, mut off, mut on) = (Score::default(), Score::default(), Score::default());
    let mut ctx = MatchCtx::new(&e.g, &e.idx, MatchConfig::default());
    for (i, sc) in hard.iter().enumerate() {
        for r in 0..6u64 {
            let tr = realize(
                &e.g,
                sc,
                &DriveConfig::default(),
                &noise(),
                10.0,
                100 + ((i as u64) << 8) + r,
            );
            let focus = sc.focus.map(|f| (f, sc.focus_radius_m));
            let (p, rt) = nearest_snap(&e.g, &e.idx, Mode::Car, &tr.obs);
            snap.add(&score_trace(&e.g, &tr, &p, &rt, focus));
            let m = ctx.match_offline(&tr.obs);
            off.add(&score_trace(&e.g, &tr, &m.points, &m.route, focus));
            let mut om = OnlineMatcher::new(Lag::Points(5));
            let mut pts = vec![None; tr.obs.len()];
            for o in &tr.obs {
                for d in om.push(&mut ctx, *o) {
                    pts[d.seq as usize] = d.point;
                }
            }
            for d in om.flush(0) {
                pts[d.seq as usize] = d.point;
            }
            let route = ctx.reconstruct_route(&pts);
            on.add(&score_trace(&e.g, &tr, &pts, &route, focus));
        }
    }
    println!(
        "snap {:.1}%/{:.1}%  offline {:.1}%/{:.1}%  online(5) {:.1}%/{:.1}%  RMF {:.2} / {:.2} / {:.2}  ({} điểm)",
        snap.edge_pct(),
        snap.edge_near_pct(),
        off.edge_pct(),
        off.edge_near_pct(),
        on.edge_pct(),
        on.edge_near_pct(),
        snap.rmf(),
        off.rmf(),
        on.rmf(),
        snap.n
    );
    assert!(
        off.edge_pct() >= snap.edge_pct() + 5.0,
        "offline {} vs snap {}",
        off.edge_pct(),
        snap.edge_pct()
    );
    assert!(
        on.edge_pct() >= snap.edge_pct() + 3.0,
        "online {} vs snap {}",
        on.edge_pct(),
        snap.edge_pct()
    );
    assert!(
        off.rmf() < snap.rmf(),
        "lộ trình HMM phải gần thật hơn dãy cạnh rời rạc của snap"
    );
}

#[test]
fn matching_is_deterministic_for_a_fixed_seed() {
    let e = env();
    let sc = e
        .scenarios
        .iter()
        .find(|s| s.kind == Kind::Overpass)
        .expect("có cầu vượt");
    let tr = realize(&e.g, sc, &DriveConfig::default(), &noise(), 10.0, 5);
    let mut ctx = MatchCtx::new(&e.g, &e.idx, MatchConfig::default());
    let a = ctx.match_offline(&tr.obs);
    let b = ctx.match_offline(&tr.obs);
    assert_eq!(a.route, b.route);
    assert!(
        a.points
            .iter()
            .zip(&b.points)
            .all(|(x, y)| x.map(|m| m.edge) == y.map(|m| m.edge))
    );
}
