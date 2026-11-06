//! Tính chất kiểm bằng proptest trên fixture (không cần mạng): Kalman bám đường thẳng không nhiễu, HMM trên vết sạch,
//! online với độ trễ vô hạn trùng offline, mỗi điểm được chốt đúng một lần theo thứ tự, lộ trình liền mạch khi không gãy chuỗi.

use geo_core::{LatLng, destination, haversine_m};
use map_match::eval::{canon, score_trace};
use map_match::hmm::{MatchConfig, MatchCtx, MatchedPoint};
use map_match::kalman::{KalmanConfig, KalmanFilter};
use map_match::online::{Lag, OnlineMatcher};
use map_match::route::Router;
use map_match::scenario::random_trip;
use map_match::trace::{DriveConfig, NoiseConfig, Trace, corrupt, drive};
use map_match::{Obs, Rng};
use proptest::prelude::*;
use road_graph::fixture::{compile_fixture, generate};
use road_graph::snap::SnapIndex;
use road_graph::{Graph, Mode};

fn setup() -> (Graph, SnapIndex) {
    let g = compile_fixture(&generate());
    let idx = SnapIndex::build(&g);
    (g, idx)
}

fn trace(g: &Graph, seed: u64, noise: &NoiseConfig) -> Option<Trace> {
    let mut rng = Rng::new(seed);
    let mut router = Router::new(g);
    let route = random_trip(g, &mut router, Mode::Car, &mut rng, 600.0, 1500.0)?;
    let truth = drive(g, &route, &DriveConfig::default(), &mut rng);
    Some(corrupt(&route, truth, noise, &mut rng))
}

fn run_online(
    ctx: &mut MatchCtx<'_>,
    tr: &Trace,
    lag: usize,
) -> (Vec<Option<MatchedPoint>>, Vec<u64>) {
    let mut m = OnlineMatcher::new(Lag::Points(lag));
    let mut out = Vec::new();
    for o in &tr.obs {
        out.extend(m.push(ctx, *o));
    }
    out.extend(m.flush(0));
    let mut pts = vec![None; tr.obs.len()];
    for d in &out {
        pts[d.seq as usize] = d.point;
    }
    (pts, out.iter().map(|d| d.seq).collect())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn kalman_follows_noise_free_straight_line(
        speed in 1.0f64..25.0, bearing in 0.0f64..360.0, dt in 1i64..4,
    ) {
        let start = LatLng::try_new(10.77, 106.69).unwrap();
        let mut kf = KalmanFilter::new(KalmanConfig::default());
        let mut worst: f64 = 0.0;
        for k in 0..50i64 {
            let truth = destination(start, bearing, speed * (k * dt) as f64);
            let o = Obs { accuracy_m: Some(5.0), ..Obs::new(k * dt * 1000, truth) };
            let (e, _) = kf.update(&o).unwrap();
            if k >= 15 {
                worst = worst.max(haversine_m(e.pos, truth));
            }
        }
        prop_assert!(worst < 1.0, "sai số lớn nhất sau khởi động {worst}");
    }

    #[test]
    fn clean_trace_is_matched_to_the_true_road(seed in 0u64..10_000) {
        let (g, idx) = setup();
        let clean = NoiseConfig {
            sigma_m: 0.5, tau_s: 0.0, p_outlier: 0.0, p_drop: 0.0, accuracy_miscal: 0.0, ..NoiseConfig::default()
        };
        let Some(tr) = trace(&g, seed, &clean) else { return Ok(()) };
        let mut ctx = MatchCtx::new(&g, &idx, MatchConfig::default());
        let r = ctx.match_offline(&tr.obs);
        let s = score_trace(&g, &tr, &r.points, &r.route, None);
        prop_assert!(s.edge_near_pct() >= 95.0, "cạnh nới {}", s.edge_near_pct());
        prop_assert!(s.recall_pct() >= 95.0, "recall {}", s.recall_pct());
    }

    #[test]
    fn online_with_unbounded_lag_equals_offline(seed in 0u64..10_000) {
        let (g, idx) = setup();
        let Some(tr) = trace(&g, seed, &NoiseConfig::default()) else { return Ok(()) };
        let mut ctx = MatchCtx::new(&g, &idx, MatchConfig::default());
        let off = ctx.match_offline(&tr.obs);
        let (on, _) = run_online(&mut ctx, &tr, usize::MAX / 2);
        for (a, b) in off.points.iter().zip(&on) {
            prop_assert_eq!(a.map(|m| (m.edge, m.offset_m.to_bits())), b.map(|m| (m.edge, m.offset_m.to_bits())));
        }
    }

    #[test]
    fn online_decides_every_point_once_in_order(seed in 0u64..10_000, lag in 0usize..12) {
        let (g, idx) = setup();
        let Some(tr) = trace(&g, seed, &NoiseConfig::default()) else { return Ok(()) };
        let mut ctx = MatchCtx::new(&g, &idx, MatchConfig::default());
        let (_, seqs) = run_online(&mut ctx, &tr, lag);
        let want: Vec<u64> = (0..tr.obs.len() as u64).collect();
        prop_assert_eq!(seqs, want);
    }

    #[test]
    fn offline_route_is_contiguous_when_chain_never_breaks(seed in 0u64..10_000) {
        let (g, idx) = setup();
        let Some(tr) = trace(&g, seed, &NoiseConfig::default()) else { return Ok(()) };
        let mut ctx = MatchCtx::new(&g, &idx, MatchConfig::default());
        let r = ctx.match_offline(&tr.obs);
        if r.breaks == 0 {
            for w in r.route.windows(2) {
                prop_assert_eq!(g.head[w[0] as usize], g.tail[w[1] as usize], "cạnh {} → {}", w[0], w[1]);
            }
        }
        // mọi cạnh được gán đều đi được bằng ô tô
        for m in r.points.iter().flatten() {
            prop_assert!(g.modes[m.edge as usize] & Mode::CAR_BIT != 0);
            let _ = canon(&g, m.edge);
        }
    }
}
