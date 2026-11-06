//! LOC-17: bám đường online (cửa sổ trượt, độ trễ quyết định, sửa lại điểm cũ), ca khó của đô thị Việt Nam, mất tín hiệu, ngân sách CPU.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc17_online
//! Tham số: --trips N (chuyến thường, mặc định 40), --per-kind N (ca khó mỗi loại, mặc định 12), --reps N (số lần nhiễu mỗi ca, mặc định 3),
//!          --drivers N (số tài xế xen kẽ khi đo CPU, mặc định 400), --seed S (báo cáo), --fixture

use geo_core::{LatLng, haversine_m};
use labs::Args;
use labs::p02::{self, EVAL_SEED, TUNE_SEED, World, p95};
use map_match::eval::{Score, canon, nearest_snap, score_trace};
use map_match::hmm::{MatchConfig, MatchCtx, MatchedPoint};
use map_match::online::{Decision, Lag, OnlineMatcher};
use map_match::route::Router;
use map_match::scenario::{Catalog, InEdges, Kind, PairKind, Scenario, Split, build, realize};
use map_match::trace::{DriveConfig, NoiseConfig, Trace};
use road_graph::{EdgeId, Mode};
use std::time::Instant;

fn run_online(
    ctx: &mut MatchCtx<'_>,
    tr: &Trace,
    lag: Lag,
) -> (Vec<Option<MatchedPoint>>, Vec<Decision>) {
    let mut m = OnlineMatcher::new(lag);
    let mut decisions = Vec::with_capacity(tr.obs.len());
    for o in &tr.obs {
        decisions.extend(m.push(ctx, *o));
    }
    decisions.extend(m.flush(tr.obs.last().map_or(0, |o| o.t_ms)));
    let mut pts = vec![None; tr.obs.len()];
    for d in &decisions {
        pts[d.seq as usize] = d.point;
    }
    (pts, decisions)
}

/// (loại, sai số nội suy theo đường, sai số nối thẳng, số khoảng, số lần gãy chuỗi)
type GapRow = (Kind, Vec<f64>, Vec<f64>, usize, usize);
/// (vết, kết quả snap, kết quả online, kết quả offline)
type Sample = (
    Trace,
    Vec<Option<MatchedPoint>>,
    Vec<Option<MatchedPoint>>,
    Vec<Option<MatchedPoint>>,
);

#[derive(Default)]
struct Agg {
    score: Score,
    revised: usize,
    first_wrong: usize,
    decisions: usize,
    lags_s: Vec<f64>,
}

fn online_score(
    w: &World,
    ctx: &mut MatchCtx<'_>,
    traces: &[Trace],
    lag: Lag,
    focus: impl Fn(&Trace) -> Option<(LatLng, f64)>,
) -> Agg {
    let mut a = Agg::default();
    for tr in traces {
        let (pts, ds) = run_online(ctx, tr, lag);
        let route = ctx.reconstruct_route(&pts);
        a.score.add(&score_trace(&w.g, tr, &pts, &route, focus(tr)));
        for d in &ds {
            a.decisions += 1;
            a.revised += usize::from(d.revisions > 0);
            if let (Some(f), Some(p)) = (d.first_guess, d.point)
                && canon(&w.g, f) != canon(&w.g, p.edge)
            {
                a.first_wrong += 1;
            }
            a.lags_s.push(d.lag_ms as f64 / 1000.0);
        }
    }
    a
}

fn row(name: &str, s: &Score) -> String {
    format!(
        "{name:<30} {:>8.2}% {:>8.2}% {:>8.2}% {:>8.2}% {:>7.3} {:>8}",
        s.edge_pct(),
        s.edge_near_pct(),
        s.dir_near_pct(),
        s.recall_pct(),
        s.rmf(),
        s.n
    )
}
const HEAD: &str =
    "                               cạnh chặt  cạnh nới  chiều nới  recall    RMF   số điểm";

fn main() {
    let args = Args::from_env();
    let n = args.u64_or("--trips", 40) as usize;
    let per_kind = args.u64_or("--per-kind", 12) as usize;
    let reps = args.u64_or("--reps", 3);
    let drivers = args.u64_or("--drivers", 400) as usize;
    let seed = args.u64_or("--seed", EVAL_SEED);
    let w = p02::world(&args);
    p02::header(
        "LOC-17: bám đường online",
        &w,
        &format!(
            "{n} chuyến thường, tối đa {per_kind} ca khó mỗi loại × {reps} lần nhiễu; seed báo cáo {seed}; tham số HMM = MatchConfig::default() (đã chọn ở LOC-16)"
        ),
    );
    let cfg = MatchConfig::default();
    let mode = Mode::Car;

    // 0. danh mục ca khó trên đồ thị
    let t = Instant::now();
    let cat = Catalog::new(&w.g, &w.idx, mode);
    let n_par = cat
        .pairs
        .iter()
        .filter(|p| p.kind == PairKind::Parallel)
        .count();
    println!(
        "-- 0. ca khó tìm được trên đồ thị (quét hình học và thẻ, {:.1} s) --",
        t.elapsed().as_secs_f64()
    );
    println!(
        "cầu vượt cắt đường ô tô bên dưới: {}; cặp đường song song (6–35 m, ≥100 m, cùng hạng): {n_par}; cặp đường chính + đường gom: {}; hầm cho ô tô (≥80 m): {}",
        cat.overpasses.len(),
        cat.pairs.len() - n_par,
        cat.tunnels.len()
    );
    let name = |e: EdgeId| w.g.edge(e).name().unwrap_or("(không tên)").to_string();
    for c in cat
        .overpasses
        .iter()
        .step_by((cat.overpasses.len() / 5).max(1))
        .take(5)
    {
        println!(
            "  cầu vượt: cạnh {} ({}) trên cạnh {} ({}) tại {:.6},{:.6}",
            c.bridge,
            name(c.bridge),
            c.lower,
            name(c.lower),
            c.point.lat(),
            c.point.lng()
        );
    }
    for p in cat
        .pairs
        .iter()
        .step_by((cat.pairs.len() / 5).max(1))
        .take(5)
    {
        println!(
            "  {:?}: cạnh {} ({}) và cạnh {} ({}), chạy chung {:.0} m, cách trung bình {:.1} m, {}",
            p.kind,
            p.a,
            name(p.a),
            p.b,
            name(p.b),
            p.overlap_m,
            p.gap_m,
            if p.same_dir {
                "cùng chiều"
            } else {
                "ngược chiều"
            }
        );
    }
    println!();

    // 1. quét độ trễ trên chuyến thường
    let drive1 = DriveConfig::default();
    let trips = p02::trips(&w, n, seed, &drive1);
    let sigma8 = NoiseConfig {
        sigma_m: 8.0,
        ..NoiseConfig::default()
    };
    let tr8 = p02::corrupt_all(&trips, &sigma8, seed);
    println!(
        "-- 1. độ trễ quyết định (chuyến thường, σ = 8 m, 1 Hz, {} điểm): chờ thêm L điểm thì chốt chắc hơn --",
        tr8.iter().map(|t| t.obs.len()).sum::<usize>()
    );
    println!("{HEAD}   %sửa  %đầu-sai  trễ p50/p95 (s)");
    let mut ctx = MatchCtx::new(&w.g, &w.idx, cfg.clone());
    let mut snap = Score::default();
    for tr in &tr8 {
        let (p, r) = nearest_snap(&w.g, &w.idx, mode, &tr.obs);
        snap.add(&score_trace(&w.g, tr, &p, &r, None));
    }
    println!("{}", row("snap về cạnh gần nhất", &snap));
    for lag in [0usize, 1, 2, 3, 5, 8, 12, 20] {
        let a = online_score(&w, &mut ctx, &tr8, Lag::Points(lag), |_| None);
        let mut l = a.lags_s.clone();
        l.sort_by(f64::total_cmp);
        println!(
            "{}  {:>5.1}%  {:>6.1}%   {:>4.1} / {:>4.1}",
            row(&format!("online, trễ {lag} điểm"), &a.score),
            100.0 * a.revised as f64 / a.decisions as f64,
            100.0 * a.first_wrong as f64 / a.decisions as f64,
            labs::percentile(&l, 50.0),
            labs::percentile(&l, 95.0)
        );
    }
    let mut off = Score::default();
    for tr in &tr8 {
        let r = ctx.match_offline(&tr.obs);
        off.add(&score_trace(&w.g, tr, &r.points, &r.route, None));
    }
    println!("{}", row("offline (nhìn cả chuyến)", &off));
    println!(
        "(%sửa: điểm có ước lượng tạm đổi cạnh ít nhất một lần trước khi chốt; %đầu-sai: cạnh gán lần đầu khác cạnh chốt)\n"
    );

    // 2. ca khó
    let mut router = Router::new(&w.g);
    let rev = InEdges::new(&w.g);
    let scenarios: Vec<Scenario> = build(
        &w.g,
        &cat,
        &rev,
        &mut router,
        mode,
        Split::Eval,
        per_kind,
        seed,
    );
    println!(
        "-- 2. ca khó (tập báo cáo: chỉ số lẻ trong danh mục; tập chẵn dành cho chọn tham số và chưa dùng; mỗi ca {reps} lần nhiễu σ = 6 m + vùng lệch 12 m quanh điểm khó; chấm các điểm trong 45 m quanh điểm khó, lộ trình chấm cả vết) --"
    );
    println!("{HEAD}");
    let noise6 = NoiseConfig::default();
    let mut gap_rows: Vec<GapRow> = Vec::new();
    for kind in Kind::HARD {
        let scs: Vec<&Scenario> = scenarios.iter().filter(|s| s.kind == kind).collect();
        if scs.is_empty() {
            println!("{}: không có ca nào trong đồ thị này", kind.label());
            continue;
        }
        let mut traces: Vec<(Trace, Option<(LatLng, f64)>)> = Vec::new();
        for (si, sc) in scs.iter().enumerate() {
            for r in 0..reps {
                let tr = realize(
                    &w.g,
                    sc,
                    &drive1,
                    &noise6,
                    12.0,
                    seed ^ ((si as u64) << 8 | r),
                );
                traces.push((tr, sc.focus.map(|f| (f, sc.focus_radius_m))));
            }
        }
        let mut s_snap = Score::default();
        let mut s_off = Score::default();
        let mut s_on3 = Score::default();
        let mut s_on8 = Score::default();
        let mut s_on0 = Score::default();
        let (mut interp, mut straight) = (Vec::<f64>::new(), Vec::<f64>::new());
        let (mut gaps, mut breaks) = (0usize, 0usize);
        for (tr, focus) in &traces {
            let (p, r) = nearest_snap(&w.g, &w.idx, mode, &tr.obs);
            s_snap.add(&score_trace(&w.g, tr, &p, &r, *focus));
            let off = ctx.match_offline(&tr.obs);
            breaks += off.breaks;
            s_off.add(&score_trace(&w.g, tr, &off.points, &off.route, *focus));
            for (lag, acc) in [(0usize, &mut s_on0), (3, &mut s_on3), (8, &mut s_on8)] {
                let (pts, _) = run_online(&mut ctx, tr, Lag::Points(lag));
                let route = ctx.reconstruct_route(&pts);
                acc.add(&score_trace(&w.g, tr, &pts, &route, *focus));
                if lag == 8 && matches!(kind, Kind::Tunnel | Kind::SignalLoss) {
                    for i in 0..tr.obs.len() - 1 {
                        let dt = (tr.obs[i + 1].t_ms - tr.obs[i].t_ms) as f64 / 1000.0;
                        if dt < 6.0 {
                            continue;
                        }
                        let (Some(a), Some(b)) = (pts[i], pts[i + 1]) else {
                            continue;
                        };
                        let (ti, tj) = (tr.obs_truth[i], tr.obs_truth[i + 1]);
                        let fr: Vec<f64> = (ti + 1..tj)
                            .map(|k| {
                                (tr.truth[k].t_ms - tr.truth[ti].t_ms) as f64
                                    / (tr.truth[tj].t_ms - tr.truth[ti].t_ms) as f64
                            })
                            .collect();
                        let ip = ctx.interpolate_gap(&a, &b, &fr);
                        if ip.len() != fr.len() {
                            continue;
                        }
                        gaps += 1;
                        for (k, (p, f)) in ip.iter().zip(&fr).enumerate() {
                            let t = tr.truth[ti + 1 + k].pos;
                            interp.push(haversine_m(*p, t));
                            let (o1, o2) = (tr.obs[i].pos, tr.obs[i + 1].pos);
                            let lin = LatLng::try_new(
                                o1.lat() + f * (o2.lat() - o1.lat()),
                                o1.lng() + f * (o2.lng() - o1.lng()),
                            )
                            .expect("hợp lệ");
                            straight.push(haversine_m(lin, t));
                        }
                    }
                }
            }
        }
        println!("{} ({} ca × {reps} nhiễu):", kind.label(), scs.len());
        println!("{}", row("  snap về cạnh gần nhất", &s_snap));
        println!("{}", row("  online, trễ 0 (tham lam)", &s_on0));
        println!("{}", row("  online, trễ 3 điểm", &s_on3));
        println!("{}", row("  online, trễ 8 điểm", &s_on8));
        println!("{}  (gãy chuỗi {breaks})", row("  offline", &s_off));
        if kind == Kind::Tunnel {
            // độ nhạy, KHÔNG dùng để chọn tham số: hầm dài hơn 90 s (mặc định gap_break_s) làm gãy chuỗi
            let mut c = cfg.clone();
            c.gap_break_s = 300.0;
            let mut ctx2 = MatchCtx::new(&w.g, &w.idx, c);
            let (mut s2, mut br2) = (Score::default(), 0usize);
            for (tr, focus) in &traces {
                let off = ctx2.match_offline(&tr.obs);
                br2 += off.breaks;
                s2.add(&score_trace(&w.g, tr, &off.points, &off.route, *focus));
            }
            println!(
                "{}  (gãy chuỗi {br2}; độ nhạy, không dùng chọn tham số)",
                row("  offline, gap_break_s = 300", &s2)
            );
        }
        if matches!(kind, Kind::Tunnel | Kind::SignalLoss) {
            gap_rows.push((kind, interp, straight, gaps, breaks));
        }
    }
    println!(
        "\n-- 2b. nội suy trong khoảng mất tín hiệu (≥ 6 s không có điểm), kết quả online trễ 8 điểm --"
    );
    println!(
        "{:<28} {:>7} {:>10} {:>26} {:>26}",
        "loại",
        "số khoảng",
        "số điểm",
        "sai số nội suy theo đường p50/p95/max (m)",
        "sai số nối thẳng p50/p95/max (m)"
    );
    for (kind, interp, straight, gaps, _) in gap_rows {
        if interp.is_empty() {
            println!(
                "{:<28} {gaps:>7} (không có điểm nào để nội suy)",
                kind.label()
            );
            continue;
        }
        let stat = |v: &[f64]| {
            let mut s = v.to_vec();
            s.sort_by(f64::total_cmp);
            format!(
                "{:.1} / {:.1} / {:.1}",
                labs::percentile(&s, 50.0),
                p95(v.to_vec()),
                s[s.len() - 1]
            )
        };
        println!(
            "{:<28} {gaps:>7} {:>10} {:>26} {:>26}",
            kind.label(),
            interp.len(),
            stat(&interp),
            stat(&straight)
        );
    }

    // 3. ngân sách CPU
    println!(
        "\n-- 3. ngân sách CPU theo tài xế (online, {drivers} tài xế xen kẽ từng điểm, σ = 8 m, 1 luồng) --"
    );
    println!("uptime: {}", p02::uptime());
    let dr_trips = p02::trips(&w, drivers.min(200), seed ^ 0xD71, &drive1);
    let mut dr_tr = p02::corrupt_all(&dr_trips, &sigma8, seed ^ 0xD71);
    while dr_tr.len() < drivers {
        let k = dr_tr.len() % dr_trips.len();
        dr_tr.push(dr_tr[k].clone());
    }
    println!(
        "{:>8} {:>12} {:>12} {:>10} {:>10} {:>11} {:>10} {:>12} {:>20}",
        "trễ",
        "max_settled",
        "µs/điểm",
        "p50 (µs)",
        "p99 (µs)",
        "tối đa (µs)",
        "cạnh nới",
        "chạm trần",
        "bộ nhớ/xe (TB, tối đa)"
    );
    let mut cpu_summary = Vec::new();
    for (lag, max_settled) in [
        (0usize, 4000usize),
        (3, 4000),
        (8, 4000),
        (20, 4000),
        (3, 4),
        (3, 8),
        (3, 16),
        (3, 50),
        (3, 200),
    ] {
        let mut c = cfg.clone();
        c.max_settled = max_settled;
        let mut ctx = MatchCtx::new(&w.g, &w.idx, c);
        let mut ms: Vec<OnlineMatcher> = (0..dr_tr.len())
            .map(|_| OnlineMatcher::new(Lag::Points(lag)))
            .collect();
        let longest = dr_tr.iter().map(|t| t.obs.len()).max().unwrap_or(0);
        let mut times: Vec<f64> = Vec::new();
        let mut pts: Vec<Vec<Option<MatchedPoint>>> =
            dr_tr.iter().map(|t| vec![None; t.obs.len()]).collect();
        let (mut mem_max, mut mem_sum, mut mem_n) = (0usize, 0usize, 0usize);
        for i in 0..longest {
            for (d, tr) in dr_tr.iter().enumerate() {
                if i >= tr.obs.len() {
                    continue;
                }
                let t0 = Instant::now();
                let out = ms[d].push(&mut ctx, tr.obs[i]);
                times.push(t0.elapsed().as_secs_f64() * 1e6);
                for dd in out {
                    pts[d][dd.seq as usize] = dd.point;
                }
                let m = ms[d].mem_bytes();
                mem_max = mem_max.max(m);
                mem_sum += m;
                mem_n += 1;
            }
        }
        let mut sc = Score::default();
        for (d, tr) in dr_tr.iter().enumerate() {
            let mut m = std::mem::replace(&mut ms[d], OnlineMatcher::new(Lag::Points(0)));
            for dd in m.flush(tr.obs.last().map_or(0, |o| o.t_ms)) {
                pts[d][dd.seq as usize] = dd.point;
            }
            sc.add(&score_trace(&w.g, tr, &pts[d], &[], None));
        }
        let mean = times.iter().sum::<f64>() / times.len() as f64;
        times.sort_by(f64::total_cmp);
        let mem_avg = mem_sum / mem_n.max(1);
        println!(
            "{lag:>8} {max_settled:>12} {mean:>12.2} {:>10.2} {:>10.2} {:>11.1} {:>9.2}% {:>12} {:>9} B, {mem_max} B",
            labs::percentile(&times, 50.0),
            labs::percentile(&times, 99.0),
            times[times.len() - 1],
            sc.edge_near_pct(),
            ctx.router.budget_hits,
            mem_avg
        );
        cpu_summary.push((lag, max_settled, mean, mem_avg));
    }
    println!(
        "tải tham chiếu của Tới Nơi: 25.000 tài xế, mỗi 4 giây một điểm = 6.250 điểm/giây (kiến trúc §14, LOC-09)."
    );
    for (lag, ms, mean, mem) in cpu_summary.iter().filter(|c| c.1 == 4000) {
        println!(
            "  trễ {lag} điểm, max_settled {ms}: 6.250 điểm/s × {mean:.2} µs = {:.4} lõi; 25.000 tài xế × {mem} B = {:.1} MB",
            6250.0 * mean / 1e6,
            25_000.0 * *mem as f64 / 1e6
        );
    }

    // 4. một cầu vượt cụ thể
    println!(
        "\n-- 4. một cầu vượt cụ thể: chạy trên cầu vượt bằng ô tô, 20 lần nhiễu σ = 6 m + lệch 12 m quanh giao điểm (minh họa một ca, không phải thống kê) --"
    );
    let flyover = cat.overpasses.iter().find(|c| {
        name(c.bridge).starts_with("Cầu vượt")
            && w.g.class[c.lower as usize] <= road_graph::HighwayClass::Tertiary as u8
            && f64::from(w.g.length_cm[c.bridge as usize]) / 100.0 >= 100.0
    });
    println!(
        "số giao cắt có tên cầu bắt đầu bằng «Cầu vượt»: {}",
        cat.overpasses
            .iter()
            .filter(|c| name(c.bridge).starts_with("Cầu vượt"))
            .count()
    );
    if let Some(c) = flyover {
        let mut rng = map_match::Rng::new(seed);
        let route =
            map_match::scenario::extend_route(&w.g, &rev, mode, c.bridge, 350.0, 350.0, &mut rng);
        let sc = Scenario {
            kind: Kind::Overpass,
            name: format!(
                "cạnh {} ({}) trên cạnh {} ({})",
                c.bridge,
                name(c.bridge),
                c.lower,
                name(c.lower)
            ),
            route,
            focus: Some(c.point),
            focus_radius_m: 45.0,
            pair: Some((c.bridge, c.lower)),
        };
        let focus = c.point;
        println!("{}", sc.name);
        let poly = |e: EdgeId| {
            w.g.edge_points(e)
                .iter()
                .map(|p| format!("{:.6},{:.6}", p.lat(), p.lng()))
                .collect::<Vec<_>>()
                .join(" ")
        };
        println!("cầu vượt: {}", poly(c.bridge));
        println!("đường dưới: {}", poly(c.lower));
        println!("giao điểm: {:.6},{:.6}", focus.lat(), focus.lng());
        let (mut s_snap, mut s_on3, mut s_on8, mut s_off) = (
            Score::default(),
            Score::default(),
            Score::default(),
            Score::default(),
        );
        let mut first: Option<Sample> = None;
        for r in 0..20u64 {
            let tr = realize(&w.g, &sc, &drive1, &noise6, 12.0, seed ^ (0xF1 << 8 | r));
            let fo = Some((focus, 45.0));
            let (sp, sr) = nearest_snap(&w.g, &w.idx, mode, &tr.obs);
            s_snap.add(&score_trace(&w.g, &tr, &sp, &sr, fo));
            let off = ctx.match_offline(&tr.obs);
            s_off.add(&score_trace(&w.g, &tr, &off.points, &off.route, fo));
            let (p3, _) = run_online(&mut ctx, &tr, Lag::Points(3));
            let r3 = ctx.reconstruct_route(&p3);
            s_on3.add(&score_trace(&w.g, &tr, &p3, &r3, fo));
            let (p8, _) = run_online(&mut ctx, &tr, Lag::Points(8));
            let r8 = ctx.reconstruct_route(&p8);
            s_on8.add(&score_trace(&w.g, &tr, &p8, &r8, fo));
            if r == 0 {
                first = Some((tr, sp, p3, off.points));
            }
        }
        println!("{HEAD}");
        println!("{}", row("snap về cạnh gần nhất", &s_snap));
        println!("{}", row("online, trễ 3 điểm", &s_on3));
        println!("{}", row("online, trễ 8 điểm", &s_on8));
        println!("{}", row("offline", &s_off));
        if let Some((tr, sp, on, off)) = first {
            let near = (0..tr.obs.len())
                .min_by(|&a, &b| {
                    haversine_m(tr.truth_of(a).pos, focus)
                        .total_cmp(&haversine_m(tr.truth_of(b).pos, focus))
                })
                .unwrap_or(0);
            println!(
                "lần nhiễu đầu tiên; cạnh cầu = {}, cạnh đường dưới = {}",
                c.bridge, c.lower
            );
            println!(
                "t_s,truth_edge,snap_edge,online3_edge,offline_edge,raw_lat,raw_lng,online3_lat,online3_lng,truth_lat,truth_lng"
            );
            for i in near.saturating_sub(10)..(near + 11).min(tr.obs.len()) {
                let f =
                    |m: &Option<MatchedPoint>| m.map_or("-".to_string(), |m| m.edge.to_string());
                let (o, t) = (on[i], tr.truth_of(i));
                println!(
                    "{},{},{},{},{},{:.7},{:.7},{},{},{:.7},{:.7}",
                    tr.obs[i].t_ms / 1000,
                    t.edge,
                    f(&sp[i]),
                    f(&on[i]),
                    f(&off[i]),
                    tr.obs[i].pos.lat(),
                    tr.obs[i].pos.lng(),
                    o.map_or("-".into(), |m| format!("{:.7}", m.pos.lat())),
                    o.map_or("-".into(), |m| format!("{:.7}", m.pos.lng())),
                    t.pos.lat(),
                    t.pos.lng()
                );
            }
        }
    } else {
        println!("không có cầu vượt nào đủ điều kiện");
    }
    let _ = TUNE_SEED;
    println!("\nuptime lúc kết thúc: {}", p02::uptime());
}
