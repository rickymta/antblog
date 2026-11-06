//! LOC-48: lệch tuyến, định tuyến lại và ghép lại. Phát hiện lệch tuyến từ bám đường online với hysteresis, thông báo có
//! ngân sách, và chính sách đổi tài xế có biên. **Thí nghiệm độc lập, không cắm vào simulator** (simulator chưa có mô hình
//! lệch tuyến); vết GPS, nhiễu, lệch tuyến và nhiễu ETA đều do lab sinh ra («trên dữ liệu mô phỏng»).
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc48_lech_tuyen_dinh_tuyen_lai
//! Tham số: --tune N (số seed chọn tham số, mặc định 100), --eval N (số seed báo cáo, mặc định 200), --hard N (ca đường song song), --fixture
//!
//! Định tuyến lại phía server hay phía app?
//! - App tự định tuyến lại (điều hướng của tài xế): phản ứng tức thì, không tốn CPU server, chạy được khi mất mạng, nhưng
//!   server **không biết** tuyến mới nên ETA gửi cho khách cũ dần, và không thể tính chuyện đổi tài xế.
//! - Server định tuyến lại: ETA đúng cho khách, đủ dữ kiện để ghép lại, nhưng tốn một truy vấn mỗi lần lệch và chỉ
//!   biết lệch sau độ trễ phát hiện (bám đường + hysteresis). Lab đo chi phí một truy vấn như vậy.
//!
//! Lab giả định cách lai: app định tuyến ngay, server phát hiện độc lập từ GPS để cập nhật ETA và cân nhắc ghép lại.
//! Việc lệch tuyến của tài xế thật (cố ý, hay do điều hướng chỉ khác) không phân biệt được từ vết GPS.

use geo_core::{LatLng, haversine_m};
use labs::p03::{uptime, vn};
use labs::{Args, die, lab_root, load_graph, percentile};
use map_match::Rng;
use map_match::hmm::{MatchConfig, MatchCtx};
use map_match::online::{Lag, OnlineMatcher};
use map_match::route::Router;
use map_match::scenario::{Catalog, InEdges, PairKind, extend_route, random_trip};
use map_match::trace::{DriveConfig, NoiseConfig, Trace, corrupt, drive};
use matcher::deviation::{
    MatchedObs, MonitorConfig, MonitorEvent, Notifier, NotifyBudget, PlannedRoute, RematchDecision,
    RematchPolicy, RematchState, RouteMonitor,
};
use rayon::prelude::*;
use road_graph::{EdgeId, Graph, Mode, NONE};
use route_engine::engine::{Engine, EngineConfig};
use route_engine::profile::Profile;
use simulator::Scenario;
use std::time::Instant;

const TUNE_SEED0: u64 = 48_001;
const EVAL_SEED0: u64 = 48_501;
/// Lệch tuyến «đáng kể»: đoạn lạc tuyến liên tục dài ít nhất chừng này (mét). Ngắn hơn là rẽ nhầm một ô phố rồi quay lại.
const MATERIAL_M: f64 = 150.0;
const LAGS: [(&str, Lag); 2] = [("trễ 0", Lag::Points(0)), ("trễ 3", Lag::Points(3))];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Dev,
    Straight,
    Hard,
}

struct Div {
    t_ms: i64,
    off_len_m: f64,
}

struct Case {
    kind: Kind,
    plan: Vec<EdgeId>,
    dest: u32,
    div: Option<Div>,
    /// Lệch nặng: chạy tiếp sai hướng 400–900 m trước khi định tuyến lại.
    severe: bool,
    truth_t: Vec<i64>,
    truth_cum: Vec<f64>,
    truth_pos: Vec<LatLng>,
    /// Điểm đã bám đường như máy chủ thấy, theo từng độ trễ trong `LAGS`.
    dec: [Vec<MatchedObs>; 2],
}

impl Case {
    fn cum_at(&self, t_ms: i64) -> f64 {
        let i = self.truth_t.partition_point(|&t| t <= t_ms);
        if i == 0 { 0.0 } else { self.truth_cum[i - 1] }
    }
    fn pos_at(&self, t_ms: i64) -> LatLng {
        let i = self.truth_t.partition_point(|&t| t <= t_ms);
        self.truth_pos[i.saturating_sub(1).min(self.truth_pos.len() - 1)]
    }
    fn material(&self) -> bool {
        self.div.as_ref().is_some_and(|d| d.off_len_m >= MATERIAL_M)
    }
}

struct Noise {
    name: &'static str,
    cfg: NoiseConfig,
}

fn noises() -> (Noise, Noise) {
    let standard = Noise {
        name: "chuẩn (σ 6 m, τ 8 s)",
        cfg: NoiseConfig::default(),
    };
    let rough = Noise {
        name: "thô (σ 12 m, 2% điểm nhảy xa, 3% rớt điểm)",
        cfg: NoiseConfig {
            sigma_m: 12.0,
            p_outlier: 0.02,
            p_drop: 0.03,
            ..NoiseConfig::default()
        },
    };
    (standard, rough)
}

fn cum_dist(truth: &[map_match::trace::TruthPoint]) -> Vec<f64> {
    let mut acc = 0.0;
    let mut out = Vec::with_capacity(truth.len());
    for i in 0..truth.len() {
        if i > 0 {
            acc += haversine_m(truth[i - 1].pos, truth[i].pos);
        }
        out.push(acc);
    }
    out
}

/// Lộ trình thực tế: theo kế hoạch tới cạnh `j`, rẽ sang cạnh khác, rồi điều hướng định tuyến lại tới đích.
/// `severe`: tài xế chạy tiếp theo hướng sai 400–900 m (bỏ qua điều hướng) rồi mới định tuyến lại; ngược lại điều hướng
/// định tuyến lại ngay sau chỗ rẽ nhầm (thường chỉ lạc một hai ô phố).
fn deviate(
    g: &Graph,
    router: &mut Router,
    inedges: &InEdges,
    plan: &[EdgeId],
    severe: bool,
    rng: &mut Rng,
) -> Option<(Vec<EdgeId>, usize)> {
    let n = plan.len();
    if n < 4 {
        return None;
    }
    let dest = g.head[plan[n - 1] as usize];
    for _ in 0..30 {
        let lo = (0.15 * n as f64).ceil() as usize;
        let hi = ((0.7 * n as f64).floor() as usize).max(lo + 1);
        let j = lo + rng.usize(hi - lo);
        if j + 1 >= n {
            continue;
        }
        let node = g.head[plan[j] as usize];
        let alts: Vec<EdgeId> = g
            .out_edges(node)
            .filter(|&e| {
                g.modes[e as usize] & Mode::Car.bit() != 0
                    && e != plan[j + 1]
                    && g.twin[plan[j] as usize] != e
                    && !plan.contains(&e)
            })
            .collect();
        if alts.is_empty() {
            continue;
        }
        let alt = alts[rng.usize(alts.len())];
        let lead: Vec<EdgeId> = if severe {
            let over = rng.range(400.0, 900.0);
            extend_route(g, inedges, Mode::Car, alt, 0.0, over, rng)
        } else {
            vec![alt]
        };
        let last = *lead.last().expect("không rỗng");
        let from = g.head[last as usize];
        let path = if from == dest {
            Vec::new()
        } else {
            match router.shortest_path(g, Mode::Car, from, dest) {
                Some((p, _)) => p,
                None => continue,
            }
        };
        // không chấp nhận quay đầu ngay sau khi rẽ (ngõ cụt): đó là quay xe, không phải đi đường khác
        if !severe
            && path
                .first()
                .is_some_and(|&p| p == g.twin[alt as usize] && p != NONE)
        {
            continue;
        }
        let mut actual: Vec<EdgeId> = plan[..=j].to_vec();
        actual.extend(lead);
        actual.extend(path);
        return Some((actual, j + 1));
    }
    None
}

fn run_decisions(ctx: &mut MatchCtx<'_>, tr: &Trace, lag: Lag) -> Vec<MatchedObs> {
    let mut m = OnlineMatcher::new(lag);
    let mut out = Vec::with_capacity(tr.obs.len());
    let push = |now: i64, ds: Vec<map_match::online::Decision>, out: &mut Vec<MatchedObs>| {
        for d in ds {
            if let Some(p) = d.point {
                out.push(MatchedObs {
                    now_ms: now,
                    t_ms: d.t_ms,
                    edge: Some(p.edge),
                    pos: p.pos,
                });
            }
        }
    };
    for o in &tr.obs {
        let ds = m.push(ctx, *o);
        push(o.t_ms, ds, &mut out);
    }
    let last = tr.obs.last().map_or(0, |o| o.t_ms);
    let ds = m.flush(last);
    push(last, ds, &mut out);
    out
}

struct Builder<'a> {
    eng: &'a Engine,
    cat: &'a Catalog,
    inedges: &'a InEdges,
    fixture: bool,
}

impl Builder<'_> {
    fn bounds(&self) -> (f64, f64) {
        if self.fixture {
            (400.0, 1_500.0)
        } else {
            (1_500.0, 6_000.0)
        }
    }

    #[allow(clippy::too_many_arguments)] // dựng một ca cần đủ ngữ cảnh (đồ thị, bám đường, luồng ngẫu nhiên); tách struct không làm rõ thêm
    fn make(
        &self,
        kind: Kind,
        plan: Vec<EdgeId>,
        actual: Vec<EdgeId>,
        div_edge_idx: Option<usize>,
        severe: bool,
        seed: u64,
        noise: &NoiseConfig,
        ctx: &mut MatchCtx<'_>,
    ) -> Case {
        let g = &self.eng.g;
        let mut r = Rng::new(seed ^ 0xB0B);
        let truth = drive(g, &actual, &DriveConfig::default(), &mut r);
        let truth_cum = cum_dist(&truth);
        let truth_t: Vec<i64> = truth.iter().map(|p| p.t_ms).collect();
        let truth_pos: Vec<LatLng> = truth.iter().map(|p| p.pos).collect();
        let plan_set: rustc_hash::FxHashSet<EdgeId> = plan.iter().copied().collect();
        let div = div_edge_idx.and_then(|k| {
            let i = truth.iter().position(|p| p.edge_idx >= k)?;
            let mut e = i;
            while e + 1 < truth.len() && !plan_set.contains(&truth[e + 1].edge) {
                e += 1;
            }
            Some(Div {
                t_ms: truth[i].t_ms,
                off_len_m: truth_cum[e] - truth_cum[i],
            })
        });
        let mut nr = Rng::new(seed ^ 0xC0FFEE);
        let tr = corrupt(&actual, truth, noise, &mut nr);
        let dec = [
            run_decisions(ctx, &tr, LAGS[0].1),
            run_decisions(ctx, &tr, LAGS[1].1),
        ];
        Case {
            kind,
            dest: g.head[*plan.last().expect("kế hoạch không rỗng") as usize],
            plan,
            div,
            severe,
            truth_t,
            truth_cum,
            truth_pos,
            dec,
        }
    }

    /// Hai ca cho mỗi seed (lệch tuyến, đi đúng tuyến) trên cùng một kế hoạch.
    fn pair_for_seed(
        &self,
        seed: u64,
        router: &mut Router,
        noise: &NoiseConfig,
        ctx: &mut MatchCtx<'_>,
    ) -> Vec<Case> {
        let g = &self.eng.g;
        let (lo, hi) = self.bounds();
        let mut rng = Rng::new(seed);
        let Some(plan) = random_trip(g, router, Mode::Car, &mut rng, lo, hi) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        let mut drng = Rng::new(seed ^ 0xDEF);
        let severe = seed % 2 == 1;
        if let Some((actual, k)) = deviate(g, router, self.inedges, &plan, severe, &mut drng) {
            out.push(self.make(
                Kind::Dev,
                plan.clone(),
                actual,
                Some(k),
                severe,
                seed,
                noise,
                ctx,
            ));
        }
        out.push(self.make(
            Kind::Straight,
            plan.clone(),
            plan,
            None,
            false,
            seed,
            noise,
            ctx,
        ));
        out
    }

    /// Ca khó: tuyến chạy dọc một đường có đường song song sát bên (6–35 m); không lệch tuyến.
    fn hard_for_seed(
        &self,
        seed: u64,
        noise: &NoiseConfig,
        ctx: &mut MatchCtx<'_>,
    ) -> Option<Case> {
        let pairs: Vec<_> = self
            .cat
            .pairs
            .iter()
            .filter(|p| p.kind == PairKind::Parallel)
            .collect();
        if pairs.is_empty() {
            return None;
        }
        let mut rng = Rng::new(seed ^ 0x4A2D);
        let p = pairs[rng.usize(pairs.len())];
        let plan = extend_route(
            &self.eng.g,
            self.inedges,
            Mode::Car,
            p.a,
            600.0,
            1_200.0,
            &mut rng,
        );
        Some(self.make(
            Kind::Hard,
            plan.clone(),
            plan,
            None,
            false,
            seed,
            noise,
            ctx,
        ))
    }
}

fn build_set(
    b: &Builder,
    seed0: u64,
    n: usize,
    n_hard: usize,
    noise: &NoiseConfig,
    mc: &MatchConfig,
) -> Vec<Case> {
    let seeds: Vec<u64> = (seed0..seed0 + n as u64).collect();
    let mut cases: Vec<Case> = seeds
        .par_iter()
        .map_init(
            || {
                (
                    Router::new(&b.eng.g),
                    MatchCtx::new(&b.eng.g, &b.eng.snap, mc.clone()),
                )
            },
            |(router, ctx), &s| b.pair_for_seed(s, router, noise, ctx),
        )
        .flatten()
        .collect();
    let hard: Vec<Case> = (seed0..seed0 + n_hard as u64)
        .into_par_iter()
        .map_init(
            || MatchCtx::new(&b.eng.g, &b.eng.snap, mc.clone()),
            |ctx, s| b.hard_for_seed(s + 7_000, noise, ctx),
        )
        .flatten()
        .collect();
    cases.extend(hard);
    cases
}

// ---------------------------------------------------------------- phát lại bộ giám sát

struct AlertRec {
    o: MatchedObs,
    last_on: Option<LatLng>,
    last_on_ms: Option<i64>,
}

struct Rep {
    alerts: Vec<AlertRec>,
    toggles: usize,
}

fn replay(g: &Graph, router: &mut Router, case: &Case, lag: usize, cfg: MonitorConfig) -> Rep {
    let mut mon = RouteMonitor::new(cfg, PlannedRoute::new(g, case.plan.clone()));
    let mut alerts = Vec::new();
    for o in &case.dec[lag] {
        if let Some(MonitorEvent::Alert { .. }) = mon.observe(o) {
            alerts.push(AlertRec {
                o: *o,
                last_on: mon.last_on_pos,
                last_on_ms: mon.last_on_ms,
            });
            // định tuyến lại phía server từ cạnh đang chạy tới đích, rồi theo dõi tuyến mới
            if let Some(e) = o.edge {
                let from = g.head[e as usize];
                let tail = if from == case.dest {
                    Some(Vec::new())
                } else {
                    router
                        .shortest_path(g, Mode::Car, from, case.dest)
                        .map(|(p, _)| p)
                };
                if let Some(path) = tail {
                    let mut edges = vec![e];
                    edges.extend(path);
                    mon.adopt(PlannedRoute::new(g, edges));
                }
            }
        }
    }
    Rep {
        alerts,
        toggles: mon.toggles(),
    }
}

#[derive(Clone, Copy)]
struct Cfg {
    name: &'static str,
    mon: MonitorConfig,
    lag: usize,
}

fn grid() -> Vec<Cfg> {
    let mut v = Vec::new();
    for lag in 0..2 {
        for &k in &[1usize, 2, 3, 5] {
            for &d in &[15.0f64, 30.0, 50.0] {
                v.push(Cfg {
                    name: "",
                    mon: MonitorConfig {
                        k_off: k,
                        off_dist_m: d,
                        ..MonitorConfig::default()
                    },
                    lag,
                });
            }
        }
    }
    for lag in 0..2 {
        v.push(Cfg {
            name: "ngây thơ",
            mon: MonitorConfig::naive(),
            lag,
        });
    }
    v
}

fn label(c: &Cfg) -> String {
    format!(
        "k_off={} d={:>2.0} m {} {}",
        if c.mon.k_off == 1 && c.mon.max_alerts == usize::MAX {
            1
        } else {
            c.mon.k_off
        },
        c.mon.off_dist_m,
        LAGS[c.lag].0,
        c.name
    )
}

#[derive(Default)]
struct Row {
    n_mat: usize,
    detected: usize,
    delay_s: Vec<f64>,
    metres: Vec<f64>,
    n_straight: usize,
    fa_straight: usize,
    toggles_straight: usize,
    n_hard: usize,
    fa_hard: usize,
    n_dev: usize,
    early_fa_dev: usize,
    alerts_dev: usize,
}

fn evaluate(g: &Graph, cases: &[Case], c: &Cfg) -> Row {
    let reps: Vec<(usize, Rep)> = cases
        .par_iter()
        .enumerate()
        .map_init(
            || Router::new(g),
            |router, (i, case)| (i, replay(g, router, case, c.lag, c.mon)),
        )
        .collect();
    let mut r = Row::default();
    for (i, rep) in reps {
        let case = &cases[i];
        match case.kind {
            Kind::Straight => {
                r.n_straight += 1;
                r.fa_straight += usize::from(!rep.alerts.is_empty());
                r.toggles_straight += rep.toggles;
            }
            Kind::Hard => {
                r.n_hard += 1;
                r.fa_hard += usize::from(!rep.alerts.is_empty());
            }
            Kind::Dev => {
                r.n_dev += 1;
                r.alerts_dev += rep.alerts.len();
                let d = case.div.as_ref().expect("ca lệch tuyến có điểm lệch");
                r.early_fa_dev += usize::from(rep.alerts.iter().any(|a| a.o.now_ms < d.t_ms));
                if case.material() {
                    r.n_mat += 1;
                    if let Some(a) = rep.alerts.iter().find(|a| a.o.now_ms >= d.t_ms) {
                        r.detected += 1;
                        r.delay_s.push((a.o.now_ms - d.t_ms) as f64 / 1000.0);
                        r.metres
                            .push((case.cum_at(a.o.now_ms) - case.cum_at(d.t_ms)).max(0.0));
                    }
                }
            }
        }
    }
    r.delay_s.sort_by(f64::total_cmp);
    r.metres.sort_by(f64::total_cmp);
    r
}

fn pct(a: usize, n: usize) -> f64 {
    if n == 0 {
        f64::NAN
    } else {
        100.0 * a as f64 / n as f64
    }
}

fn q(v: &[f64], p: f64) -> f64 {
    percentile(v, p)
}

const ROW_HEAD: &str = "  cấu hình                         phát hiện%  trễ s p50/p95   mét p50/p95/max      báo nhầm% đi đúng/song song  đổi trạng thái/chuyến đúng  báo/chuyến lệch  báo trước lệch%";

fn print_row(name: &str, r: &Row, mark: &str) {
    println!(
        "  {name:<32} {:>8.1} {:>7.1}/{:<6.1} {:>7.0}/{:>6.0}/{:<6.0} {:>8.1} /{:<8.1}      {:>10.3}              {:>10.2}        {:>8.1} {mark}",
        pct(r.detected, r.n_mat),
        q(&r.delay_s, 50.0),
        q(&r.delay_s, 95.0),
        q(&r.metres, 50.0),
        q(&r.metres, 95.0),
        r.metres.last().copied().unwrap_or(f64::NAN),
        pct(r.fa_straight, r.n_straight),
        pct(r.fa_hard, r.n_hard),
        r.toggles_straight as f64 / r.n_straight.max(1) as f64,
        r.alerts_dev as f64 / r.n_dev.max(1) as f64,
        pct(r.early_fa_dev, r.n_dev),
    );
}

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        f64::NAN
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}

// ---------------------------------------------------------------- ghép lại

/// Mốc kiểm tra sau khi báo lệch tuyến, đọc ETA bằng Engine từ vị trí **thật** (ground truth của lab).
struct Ck {
    after_s: f64,
    /// Quãng tài xế đã chạy từ lúc nhận chuyến tới mốc này (mét).
    wasted_m: f64,
    /// ETA còn lại nếu tài xế đã đi đúng tuyến (vị trí của ca đi đúng cùng kế hoạch tại cùng giờ).
    e_orig_s: f64,
    /// ETA còn lại từ vị trí thật của tài xế đã lệch.
    e_cur_s: f64,
    cur_dist_m: f64,
}

struct Event {
    severe: bool,
    cks: Vec<Ck>,
    e_alt_s: f64,
    alt_dist_m: f64,
}

#[derive(Clone, Copy)]
struct Snap {
    e_orig_s: f64,
    e_cur_s: f64,
    e_alt_s: f64,
}

fn secs(ms: u32) -> f64 {
    f64::from(ms) / 1000.0
}

fn flip_sim(pol: &RematchPolicy, ev: &Snap, sigma: f64, rng: &mut Rng) -> (u32, u32) {
    let mut st = RematchState::default();
    let mut holder_is_orig = true;
    let (mut switches, mut wrong) = (0u32, 0u32);
    let mut t = 0.0;
    while t <= 300.0 {
        let true_orig = (ev.e_cur_s - t).max(1.0);
        let true_alt = (ev.e_alt_s - t).max(1.0);
        let promised = (ev.e_orig_s - t).max(1.0);
        let (r_orig, r_alt) = (
            true_orig * (sigma * rng.gauss()).exp(),
            true_alt * (sigma * rng.gauss()).exp(),
        );
        let (cur_r, alt_r, cur_true, alt_true) = if holder_is_orig {
            (r_orig, r_alt, true_orig, true_alt)
        } else {
            (r_alt, r_orig, true_alt, true_orig)
        };
        if st.decide(pol, (t * 1000.0) as i64, promised, cur_r, alt_r) == RematchDecision::Switch {
            switches += 1;
            wrong += u32::from(alt_true > cur_true);
            holder_is_orig = !holder_is_orig;
        }
        t += 15.0;
    }
    (switches, wrong)
}

fn main() {
    let args = Args::from_env();
    let fixture = args.flag("--fixture");
    let n_tune = args.u64_or("--tune", if fixture { 30 } else { 100 }) as usize;
    let n_eval = args.u64_or("--eval", if fixture { 60 } else { 200 }) as usize;
    let n_hard = args.u64_or("--hard", if fixture { 10 } else { 40 }) as usize;
    let (g, source) = load_graph(&args);
    let (nodes, edges) = (g.node_count() as u64, g.edge_count() as u64);
    println!("== LOC-48: lệch tuyến, định tuyến lại, ghép lại ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    println!(
        "đồ thị: {}: {} nút, {} cạnh; {} seed chọn tham số {}…{}, {} seed báo cáo {}…{}",
        std::path::Path::new(&source)
            .strip_prefix(lab_root())
            .map_or(source.clone(), |p| p.display().to_string()),
        vn(nodes),
        vn(edges),
        n_tune,
        TUNE_SEED0,
        TUNE_SEED0 + n_tune as u64 - 1,
        n_eval,
        EVAL_SEED0,
        EVAL_SEED0 + n_eval as u64 - 1
    );
    println!(
        "lưu ý: trên dữ liệu mô phỏng. Vết GPS, nhiễu, đoạn lệch tuyến, xác suất tài xế rảnh và nhiễu ETA đều do lab sinh (có seed); không có GPS người thật. Không cắm vào simulator."
    );

    let root = lab_root();
    let engine = Engine::new(
        g,
        EngineConfig {
            ch_cache_dir: (!fixture).then(|| root.join("rust/target/ch-cache")),
            cache_tag: if fixture { "fixture" } else { "hcm" }.into(),
            ..EngineConfig::default()
        },
    );
    let t = Instant::now();
    let _ = engine.route(
        Profile::Car,
        engine.g.node_latlng(0),
        engine.g.node_latlng((engine.g.node_count() / 2) as u32),
        None,
    );
    println!(
        "khởi động CH ô tô (nạp từ cache hoặc dựng): {:.2} s",
        t.elapsed().as_secs_f64()
    );
    let t = Instant::now();
    let cat = Catalog::new(&engine.g, &engine.snap, Mode::Car);
    let inedges = InEdges::new(&engine.g);
    let n_par = cat
        .pairs
        .iter()
        .filter(|p| p.kind == PairKind::Parallel)
        .count();
    println!(
        "danh mục ca khó: {n_par} cặp đường song song (quét {:.1} s)",
        t.elapsed().as_secs_f64()
    );
    let b = Builder {
        eng: &engine,
        cat: &cat,
        inedges: &inedges,
        fixture,
    };
    let mc = MatchConfig::default();
    let (std_noise, rough_noise) = noises();
    println!(
        "bám đường: HMM online (MatchConfig::default() của LOC-16), lấy mẫu 1 s; nhiễu chuẩn: {}",
        std_noise.name
    );
    println!(
        "giám sát: k_on = 3, cooldown 30 s, tối đa 3 lần báo mỗi chuyến, sau mỗi lần báo server định tuyến lại và theo dõi tuyến mới (adopt)"
    );
    println!(
        "lệch tuyến «đáng kể» = đoạn lạc tuyến liên tục ≥ {MATERIAL_M:.0} m; tỉ lệ phát hiện chỉ tính trên các ca này"
    );

    let t = Instant::now();
    let tune = build_set(&b, TUNE_SEED0, n_tune, n_hard, &std_noise.cfg, &mc);
    let eval = build_set(&b, EVAL_SEED0, n_eval, n_hard, &std_noise.cfg, &mc);
    let eval_rough = build_set(&b, EVAL_SEED0, n_eval, n_hard, &rough_noise.cfg, &mc);
    println!(
        "sinh và bám đường {} + {} + {} vết: {:.1} s",
        tune.len(),
        eval.len(),
        eval_rough.len(),
        t.elapsed().as_secs_f64()
    );

    // ---- 1. tập ca
    println!("\n-- 1. tập ca --");
    for (name, set) in [("chọn tham số", &tune), ("báo cáo", &eval)] {
        let dev: Vec<&Case> = set.iter().filter(|c| c.kind == Kind::Dev).collect();
        let mut off: Vec<f64> = dev
            .iter()
            .map(|c| c.div.as_ref().map_or(0.0, |d| d.off_len_m))
            .collect();
        off.sort_by(f64::total_cmp);
        let mat = dev.iter().filter(|c| c.material()).count();
        println!(
            "  {name}: {} ca lệch tuyến ({} đáng kể), {} ca đi đúng tuyến, {} ca đi dọc đường có đường song song; đoạn lạc tuyến p50 {:.0} m, p95 {:.0} m, max {:.0} m",
            dev.len(),
            mat,
            set.iter().filter(|c| c.kind == Kind::Straight).count(),
            set.iter().filter(|c| c.kind == Kind::Hard).count(),
            q(&off, 50.0),
            q(&off, 95.0),
            off.last().copied().unwrap_or(f64::NAN)
        );
    }
    let mean_len: f64 = eval
        .iter()
        .filter(|c| c.kind == Kind::Straight)
        .map(|c| c.truth_cum.last().copied().unwrap_or(0.0))
        .sum::<f64>()
        / eval
            .iter()
            .filter(|c| c.kind == Kind::Straight)
            .count()
            .max(1) as f64;
    println!("  độ dài tuyến đón trung bình (tập báo cáo): {mean_len:.0} m");

    // ---- 2. quét tham số trên tập chọn
    let cfgs = grid();
    println!(
        "\n-- 2. quét tham số trên tập CHỌN THAM SỐ (seed {TUNE_SEED0}…{}) --",
        TUNE_SEED0 + n_tune as u64 - 1
    );
    println!(
        "quy tắc chọn đặt trước: trong các cấu hình có tỉ lệ báo nhầm ≤ 2% (cả ca đi đúng lẫn ca song song), chọn cấu hình có số mét p50 sau điểm lệch nhỏ nhất; hoà thì tỉ lệ phát hiện cao hơn"
    );
    println!("{ROW_HEAD}");
    let g = &engine.g;
    let tune_rows: Vec<Row> = cfgs.iter().map(|c| evaluate(g, &tune, c)).collect();
    for (c, r) in cfgs.iter().zip(&tune_rows) {
        print_row(&label(c), r, "");
    }
    let fa = |r: &Row| pct(r.fa_straight + r.fa_hard, r.n_straight + r.n_hard);
    let chosen_i = (0..cfgs.len())
        .filter(|&i| cfgs[i].name.is_empty() && fa(&tune_rows[i]) <= 2.0)
        .min_by(|&a, &b2| {
            q(&tune_rows[a].metres, 50.0)
                .total_cmp(&q(&tune_rows[b2].metres, 50.0))
                .then(
                    pct(tune_rows[b2].detected, tune_rows[b2].n_mat)
                        .total_cmp(&pct(tune_rows[a].detected, tune_rows[a].n_mat)),
                )
        })
        .unwrap_or_else(|| {
            (0..cfgs.len())
                .filter(|&i| cfgs[i].name.is_empty())
                .min_by(|&a, &b2| fa(&tune_rows[a]).total_cmp(&fa(&tune_rows[b2])))
                .expect("có cấu hình")
        });
    let chosen = cfgs[chosen_i];
    println!(
        "→ cấu hình được chọn trên tập chọn tham số: {}",
        label(&chosen)
    );

    // ---- 3. báo cáo trên tập báo cáo
    println!(
        "\n-- 3. TẬP BÁO CÁO (seed {EVAL_SEED0}…{}), nhiễu chuẩn: mọi cấu hình, đánh dấu cấu hình đã chọn --",
        EVAL_SEED0 + n_eval as u64 - 1
    );
    println!("{ROW_HEAD}");
    let mut eval_rows = Vec::new();
    for (i, c) in cfgs.iter().enumerate() {
        let r = evaluate(g, &eval, c);
        print_row(&label(c), &r, if i == chosen_i { "<== chọn" } else { "" });
        eval_rows.push(r);
    }
    let rc = &eval_rows[chosen_i];
    let naive_i = cfgs
        .iter()
        .position(|c| c.name == "ngây thơ" && c.lag == chosen.lag)
        .expect("có cấu hình ngây thơ");
    println!(
        "đọc: cấu hình ngây thơ báo nhầm {:.1}% ca đi đúng và {:.1}% ca song song, đổi trạng thái {:.3} lần mỗi chuyến đi đúng; cấu hình chọn {:.1}%, {:.1}%, {:.3}; đổi lại độ trễ phát hiện p50 {:.0} m so với {:.0} m",
        pct(
            eval_rows[naive_i].fa_straight,
            eval_rows[naive_i].n_straight
        ),
        pct(eval_rows[naive_i].fa_hard, eval_rows[naive_i].n_hard),
        eval_rows[naive_i].toggles_straight as f64 / eval_rows[naive_i].n_straight.max(1) as f64,
        pct(rc.fa_straight, rc.n_straight),
        pct(rc.fa_hard, rc.n_hard),
        rc.toggles_straight as f64 / rc.n_straight.max(1) as f64,
        q(&rc.metres, 50.0),
        q(&eval_rows[naive_i].metres, 50.0)
    );

    // phát hiện theo độ dài đoạn lạc tuyến
    println!(
        "\n-- 4. tỉ lệ phát hiện theo độ dài đoạn lạc tuyến (tập báo cáo): cấu hình chọn và ngây thơ --"
    );
    let buckets = [
        (0.0, 50.0),
        (50.0, 150.0),
        (150.0, 400.0),
        (400.0, f64::MAX),
    ];
    for (name, c) in [("chọn", chosen), ("ngây thơ", cfgs[naive_i])] {
        let reps: Vec<(usize, Rep)> = eval
            .par_iter()
            .enumerate()
            .map_init(
                || Router::new(g),
                |router, (i, case)| (i, replay(g, router, case, c.lag, c.mon)),
            )
            .collect();
        for &(lo, hi) in &buckets {
            let (mut n, mut hit) = (0, 0);
            for (i, rep) in &reps {
                let case = &eval[*i];
                let Some(d) = case.div.as_ref() else { continue };
                if d.off_len_m >= lo && d.off_len_m < hi {
                    n += 1;
                    hit += usize::from(rep.alerts.iter().any(|a| a.o.now_ms >= d.t_ms));
                }
            }
            println!(
                "  {name:<9} đoạn lạc {:>4.0}–{:<6} m: {hit:>3}/{n:<3} = {:>5.1}%",
                lo,
                if hi == f64::MAX {
                    "∞".to_string()
                } else {
                    format!("{hi:.0}")
                },
                pct(hit, n)
            );
        }
    }

    println!("  theo loại lệch tuyến (tập báo cáo, ca đáng kể), cấu hình chọn:");
    {
        let c = chosen;
        let reps: Vec<(usize, Rep)> = eval
            .par_iter()
            .enumerate()
            .map_init(
                || Router::new(g),
                |router, (i, case)| (i, replay(g, router, case, c.lag, c.mon)),
            )
            .collect();
        for (sev, name) in [
            (false, "nhẹ (rẽ nhầm, điều hướng lại ngay)"),
            (true, "nặng (chạy sai hướng 400–900 m)"),
        ] {
            let (mut n, mut hit) = (0, 0);
            let (mut off, mut mt) = (Vec::new(), Vec::new());
            for (i, rep) in &reps {
                let case = &eval[*i];
                let Some(d) = case.div.as_ref() else { continue };
                if case.severe == sev && case.material() {
                    n += 1;
                    off.push(d.off_len_m);
                    if let Some(a) = rep.alerts.iter().find(|a| a.o.now_ms >= d.t_ms) {
                        hit += 1;
                        mt.push((case.cum_at(a.o.now_ms) - case.cum_at(d.t_ms)).max(0.0));
                    }
                }
            }
            off.sort_by(f64::total_cmp);
            mt.sort_by(f64::total_cmp);
            println!(
                "    {name:<38} {hit:>3}/{n:<3} phát hiện; đoạn lạc p50 {:.0} m; mét sau điểm lệch p50 {:.0}, p95 {:.0}",
                q(&off, 50.0),
                q(&mt, 50.0),
                q(&mt, 95.0)
            );
        }
    }

    // ---- 5. nhiễu thô
    println!(
        "\n-- 5. độ nhạy với giả định nhiễu: tập báo cáo, nhiễu {} --",
        rough_noise.name
    );
    println!("{ROW_HEAD}");
    for (name, i) in [("cấu hình chọn", chosen_i), ("ngây thơ", naive_i)] {
        let r = evaluate(g, &eval_rough, &cfgs[i]);
        print_row(&format!("{name} ({})", label(&cfgs[i])), &r, "");
    }
    // cấu hình k_off cao nhất cùng độ trễ
    let strict_i = cfgs
        .iter()
        .position(|c| {
            c.name.is_empty() && c.lag == chosen.lag && c.mon.k_off == 5 && c.mon.off_dist_m == 50.0
        })
        .expect("lưới có k=5, d=50");
    let r = evaluate(g, &eval_rough, &cfgs[strict_i]);
    print_row(&format!("chặt nhất ({})", label(&cfgs[strict_i])), &r, "");

    // ---- 6. thông báo và chi phí định tuyến lại
    println!(
        "\n-- 6. thông báo cho hai bên và chi phí một lần định tuyến lại (cấu hình chọn, tập báo cáo, ca lệch tuyến) --"
    );
    let mut router = Router::new(g);
    let mut ms_engine = Vec::new();
    let mut ms_router = Vec::new();
    let (mut tot_alerts, mut drv_sent, mut cus_sent, mut cus_attempt) =
        (0usize, 0usize, 0usize, 0usize);
    let mut trips_with_alert = 0usize;
    let mut deltas = Vec::new();
    let mut events: Vec<Event> = Vec::new();
    let sc_path = root.join(if fixture {
        "fixtures/scenario-khu-mau.toml"
    } else {
        "fixtures/scenario-hcm-baseline.toml"
    });
    let sc = Scenario::load(&sc_path).unwrap_or_else(|e| die(&format!("{e}")));
    let world = simulator::world::generate(&engine, &sc);
    let drivers_ll: Vec<LatLng> = world.drivers.iter().map(|d| d.pos.point(g)).collect();
    let route_of = |a: LatLng, b: LatLng, ms: &mut Vec<f64>| {
        let t = Instant::now();
        let r = engine.route(Profile::Car, a, b, None);
        ms.push(t.elapsed().as_secs_f64() * 1000.0);
        r.ok()
    };
    let mut sink = Vec::new();
    for (ci, case) in eval.iter().enumerate().filter(|(_, c)| c.kind == Kind::Dev) {
        let rep = replay(g, &mut router, case, chosen.lag, chosen.mon);
        if rep.alerts.is_empty() {
            continue;
        }
        trips_with_alert += 1;
        let twin = eval
            .get(ci + 1)
            .filter(|t| t.kind == Kind::Straight && t.dest == case.dest);
        let mut nt = Notifier::new(NotifyBudget::default());
        let dest_ll = g.node_latlng(case.dest);
        for (ai, a) in rep.alerts.iter().enumerate() {
            tot_alerts += 1;
            drv_sent += usize::from(nt.driver_route_update(a.o.now_ms));
            // thời gian định tuyến lại bằng Dijkstra theo độ dài của map-match (đối chứng, không CH)
            if let Some(e) = a.o.edge {
                let t = Instant::now();
                let _ = router.shortest_path(g, Mode::Car, g.head[e as usize], case.dest);
                ms_router.push(t.elapsed().as_secs_f64() * 1000.0);
            }
            // ETA máy chủ gửi cho khách: từ điểm đã bám đường, so với ETA khi tài xế còn đi đúng tuyến (ETA từ điểm
            // trên tuyến cuối cùng, trừ thời gian đã trôi qua kể từ đó)
            let before = route_of(a.last_on.unwrap_or(a.o.pos), dest_ll, &mut ms_engine);
            let after = route_of(a.o.pos, dest_ll, &mut ms_engine);
            if let (Some(bf), Some(af)) = (before, after) {
                let elapsed = a.last_on_ms.map_or(0.0, |t| (a.o.t_ms - t) as f64 / 1000.0);
                let delta = secs(af.duration_ms) - (secs(bf.duration_ms) - elapsed).max(0.0);
                deltas.push(delta);
                cus_attempt += 1;
                cus_sent += usize::from(nt.customer_eta_update(a.o.now_ms, delta));
            }
            // sự kiện cho phần ghép lại: lần báo đầu tiên sau điểm lệch của ca đáng kể, ETA đọc từ vị trí thật tại các mốc
            let after_div = case.div.as_ref().is_some_and(|d| a.o.now_ms >= d.t_ms);
            if ai == 0
                && after_div
                && case.material()
                && let Some(tw) = twin
            {
                let last_t = case.truth_t.last().copied().unwrap_or(0);
                let mut cks = Vec::new();
                for after_s in [0.0f64, 30.0, 60.0, 90.0] {
                    let t_ms = a.o.now_ms + (after_s * 1000.0) as i64;
                    if t_ms > last_t {
                        break;
                    }
                    let cur = route_of(case.pos_at(t_ms), dest_ll, &mut sink);
                    let orig = route_of(tw.pos_at(t_ms), dest_ll, &mut sink);
                    if let (Some(c), Some(o)) = (cur, orig) {
                        cks.push(Ck {
                            after_s,
                            wasted_m: case.cum_at(t_ms),
                            e_orig_s: secs(o.duration_ms),
                            e_cur_s: secs(c.duration_ms),
                            cur_dist_m: c.distance_m,
                        });
                    }
                }
                // một tài xế khác rảnh với xác suất 67% (= 1 − 32.5% bận trung bình của baseline LOC-30)
                let mut rng = Rng::new(a.o.now_ms as u64 ^ u64::from(case.dest));
                let idle: Vec<LatLng> = drivers_ll
                    .iter()
                    .copied()
                    .filter(|_| rng.chance(0.67))
                    .collect();
                let mx = engine.matrix(Profile::Car, &idle, &[dest_ll], None);
                let best = (0..idle.len())
                    .map(|i| (mx.matrix.get(i, 0), i))
                    .filter(|&(c, _)| c != u32::MAX)
                    .min();
                if let Some((c_ms, i)) = best
                    && let Ok(alt) = engine.route(Profile::Car, idle[i], dest_ll, None)
                    && !cks.is_empty()
                {
                    events.push(Event {
                        severe: case.severe,
                        cks,
                        e_alt_s: secs(c_ms),
                        alt_dist_m: alt.distance_m,
                    });
                }
            }
        }
        let _ = nt;
    }
    ms_engine.sort_by(f64::total_cmp);
    ms_router.sort_by(f64::total_cmp);
    deltas.sort_by(f64::total_cmp);
    let n_dev_trips = eval.iter().filter(|c| c.kind == Kind::Dev).count();
    println!(
        "  {trips_with_alert}/{n_dev_trips} chuyến lệch tuyến có ít nhất một lần báo; {tot_alerts} lần báo ({:.2} mỗi chuyến có báo)",
        tot_alerts as f64 / trips_with_alert.max(1) as f64
    );
    println!(
        "  ngân sách thông báo mặc định (tài xế ≤ {} cập nhật tuyến, khách ≤ {} cập nhật ETA, cách nhau ≥ {} s, khách chỉ được báo khi ETA đổi ≥ {:.0} s):",
        NotifyBudget::default().max_driver,
        NotifyBudget::default().max_customer,
        NotifyBudget::default().min_gap_ms / 1000,
        NotifyBudget::default().customer_min_delta_s
    );
    println!(
        "    tài xế: gửi {drv_sent}/{tot_alerts} cập nhật tuyến; khách: gửi {cus_sent}/{cus_attempt} cập nhật ETA (đối chứng ngây thơ: khách nhận {tot_alerts} thông báo, mỗi lần báo một thông báo)"
    );
    println!(
        "  thay đổi ETA do lệch tuyến (ETA từ điểm lệch tới điểm đón trừ ETA nếu còn trên tuyến): p50 {:.0} s, p95 {:.0} s, max {:.0} s, tỉ lệ ≥ {:.0} s: {:.1}% (n = {})",
        q(&deltas, 50.0),
        q(&deltas, 95.0),
        deltas.last().copied().unwrap_or(f64::NAN),
        NotifyBudget::default().customer_min_delta_s,
        pct(
            deltas
                .iter()
                .filter(|&&d| d.abs() >= NotifyBudget::default().customer_min_delta_s)
                .count(),
            deltas.len()
        ),
        deltas.len()
    );
    println!(
        "  một truy vấn định tuyến lại phía server (Engine::route, CH, gồm snap và giải nén): p50 {:.2} ms, p95 {:.2} ms, max {:.2} ms (n = {}); Dijkstra theo độ dài của map-match: p50 {:.2} ms, p95 {:.2} ms (n = {}). Đo khi máy chịu tải nặng, chỉ so tương đối.",
        q(&ms_engine, 50.0),
        q(&ms_engine, 95.0),
        ms_engine.last().copied().unwrap_or(f64::NAN),
        ms_engine.len(),
        q(&ms_router, 50.0),
        q(&ms_router, 95.0),
        ms_router.len()
    );
    // chi phí giám sát
    let t = Instant::now();
    let mut pts = 0usize;
    for case in eval.iter().take(60) {
        let _ = replay(g, &mut router, case, chosen.lag, chosen.mon);
        pts += case.dec[chosen.lag].len();
    }
    println!(
        "  chi phí phát lại giám sát + định tuyến lại: {:.2} µs mỗi điểm đã bám đường ({} điểm, một luồng)",
        t.elapsed().as_secs_f64() * 1e6 / pts.max(1) as f64,
        pts
    );

    // ---- 7. ghép lại
    println!(
        "\n-- 7. ghép lại sau khi phát hiện lệch tuyến ({} sự kiện: lệch tuyến đáng kể đã được báo, ETA đọc bằng Engine từ vị trí thật của tài xế) --",
        events.len()
    );
    println!(
        "giả định: các tài xế còn lại của kịch bản {} ở vị trí khởi tạo, mỗi người rảnh với xác suất 0.67; ETA gốc = ETA từ vị trí mà cùng tài xế đó sẽ ở nếu đi đúng tuyến; ETA hiện tại = từ vị trí thật đã lệch; ứng viên = tài xế rảnh có ETA nhỏ nhất; kiểm tra lại ở mốc +0, +30, +60, +90 s sau lần báo",
        sc.name
    );
    println!(
        "  ETA hiện tại trừ ETA gốc (giây, dương = lệch tuyến làm khách chờ lâu hơn), theo mốc và loại lệch:"
    );
    for (sev, name) in [(false, "nhẹ"), (true, "nặng")] {
        for k in 0..4 {
            let mut v: Vec<f64> = events
                .iter()
                .filter(|e| e.severe == sev)
                .filter_map(|e| e.cks.get(k))
                .map(|c| c.e_cur_s - c.e_orig_s)
                .collect();
            v.sort_by(f64::total_cmp);
            println!(
                "    lệch {name:<5} mốc +{:>2} s: n = {:>3}  p50 {:>6.0}  p95 {:>6.0}  max {:>6.0}  tỉ lệ ≥ 60 s: {:>5.1}%",
                k * 30,
                v.len(),
                q(&v, 50.0),
                q(&v, 95.0),
                v.last().copied().unwrap_or(f64::NAN),
                pct(v.iter().filter(|&&x| x >= 60.0).count(), v.len())
            );
        }
    }
    let base_gap: Vec<f64> = events
        .iter()
        .filter_map(|e| e.cks.first().map(|c| c.e_orig_s - e.e_alt_s))
        .collect();
    println!(
        "  tài xế rảnh tốt nhất khác nhanh hơn chính tài xế được gán (khi chưa tính lệch tuyến) ở {:.1}% sự kiện: phần lợi này có từ lúc gán, không do lệch tuyến, và tách riêng ở cột «do lệch»",
        pct(
            base_gap.iter().filter(|&&x| x > 0.0).count(),
            base_gap.len()
        )
    );
    let pols: [(&str, RematchPolicy); 4] = [
        ("ngây thơ (đổi nếu nhanh hơn)", RematchPolicy::naive()),
        (
            "lỏng (60 s hoặc 10%)",
            RematchPolicy {
                abs_margin_s: 60.0,
                rel_margin: 0.10,
                no_rematch_within_s: 60.0,
                min_worsening_s: 30.0,
                ..RematchPolicy::guarded()
            },
        ),
        ("có biên (120 s hoặc 25%)", RematchPolicy::guarded()),
        (
            "chặt (240 s hoặc 50%)",
            RematchPolicy {
                abs_margin_s: 240.0,
                rel_margin: 0.5,
                ..RematchPolicy::guarded()
            },
        ),
    ];
    println!(
        "  {:<30} {:>8} {:>10} {:>26} {:>12} {:>15} {:>20} {:>10}",
        "chính sách (ETA đọc chính xác)",
        "đổi%",
        "sau báo s",
        "khách lợi s mean/p50/p95",
        "do lệch s",
        "km rỗng thêm",
        "tài xế mất chặng m",
        "mất ≥1km%"
    );
    for (name, p) in &pols {
        let (mut sw, mut gain, mut due, mut when) = (0usize, Vec::new(), Vec::new(), Vec::new());
        let (mut extra_km, mut lost, mut lost_big) = (0.0, Vec::new(), 0usize);
        for e in &events {
            let mut st = RematchState::default();
            for c in &e.cks {
                if st.decide(
                    p,
                    (c.after_s * 1000.0) as i64,
                    c.e_orig_s,
                    c.e_cur_s,
                    e.e_alt_s,
                ) == RematchDecision::Switch
                {
                    sw += 1;
                    gain.push(c.e_cur_s - e.e_alt_s);
                    due.push(c.e_cur_s - c.e_orig_s);
                    when.push(c.after_s);
                    extra_km += (e.alt_dist_m - c.cur_dist_m) / 1000.0;
                    lost.push(c.wasted_m);
                    lost_big += usize::from(c.wasted_m >= 1000.0);
                    break;
                }
            }
        }
        gain.sort_by(f64::total_cmp);
        println!(
            "  {name:<30} {:>7.1}% {:>10.0} {:>12.0}/{:>5.0}/{:>5.0} {:>12.0} {:>10.3} km/sk {:>20.0} {:>10.1}",
            pct(sw, events.len()),
            mean(&when),
            mean(&gain),
            q(&gain, 50.0),
            q(&gain, 95.0),
            mean(&due),
            extra_km / sw.max(1) as f64,
            mean(&lost),
            pct(lost_big, sw)
        );
    }
    println!(
        "  (km rỗng thêm = quãng tài xế mới phải chạy tới điểm đón trừ quãng tài xế đã lệch còn phải chạy, mỗi lần đổi; âm nghĩa là hệ thống chạy rỗng ít hơn khi đổi. «do lệch» = ETA hiện tại trừ ETA gốc tại lúc đổi.)"
    );

    println!(
        "\n-- 8. flip-flop khi ETA đọc có nhiễu (mô hình nhiễu tổng hợp: ETA đọc = ETA thật × exp(σ·N(0,1)), đọc mỗi 15 s trong 300 s, bắt đầu từ trạng thái mốc +60 s; cả hai tài xế cùng tiến lại điểm đón) --"
    );
    let snaps: Vec<Snap> = events
        .iter()
        .filter_map(|e| {
            e.cks.get(2).or(e.cks.last()).map(|c| Snap {
                e_orig_s: c.e_orig_s,
                e_cur_s: c.e_cur_s,
                e_alt_s: e.e_alt_s,
            })
        })
        .collect();
    println!(
        "  {:<30} {:>6} {:>14} {:>14} {:>16} {:>18}",
        "chính sách",
        "σ",
        "đổi/sự kiện",
        "đổi ≥ 2 lần%",
        "đổi tối đa",
        "đổi sang người chậm hơn thật/sự kiện"
    );
    for &sigma in &[0.15f64, 0.30] {
        for (name, p) in [&pols[0], &pols[2]] {
            let mut rng = Rng::new(0xF11F ^ (sigma * 100.0) as u64);
            let (mut sw_tot, mut wr_tot, mut ge2, mut mx) = (0u32, 0u32, 0usize, 0u32);
            for e in &snaps {
                let (s, w) = flip_sim(p, e, sigma, &mut rng);
                sw_tot += s;
                wr_tot += w;
                ge2 += usize::from(s >= 2);
                mx = mx.max(s);
            }
            let n = snaps.len().max(1) as f64;
            println!(
                "  {name:<30} {sigma:>6.2} {:>14.3} {:>13.1}% {:>16} {:>18.3}",
                f64::from(sw_tot) / n,
                pct(ge2, snaps.len()),
                mx,
                f64::from(wr_tot) / n
            );
        }
    }

    println!("\n-- 9. kết luận ngắn (từ các số ở trên) --");
    println!(
        "  Bên nào lợi: khách (ETA tới điểm đón ngắn hơn khi tài xế khác thật sự gần hơn) và hệ thống khi lệch tuyến làm ETA xấu rõ rệt. Bên nào chịu: tài xế đã lệch bị rút chuyến sau khi đã chạy một đoạn rỗng (cột «tài xế mất chặng đã chạy»), và cả hai bên bị làm phiền nếu hysteresis và ngân sách thông báo quá lỏng."
    );
    println!(
        "  Giới hạn: vết và nhiễu do lab sinh. Lệch tuyến mô phỏng là rẽ sang một cạnh khác (nhẹ: điều hướng định tuyến lại ngay; nặng: chạy tiếp sai hướng 400–900 m rồi mới định tuyến lại), không có quay đầu giữa đường, không có cấm rẽ, và không phân biệt được lệch cố ý với điều hướng chỉ khác. ETA ở phần ghép lại đọc từ vị trí thật (không qua bám đường) nên là cận trên của độ chính xác máy chủ; tài xế còn lại ở vị trí khởi tạo của simulator và rảnh ngẫu nhiên 67%; nhiễu ETA là mô hình log-chuẩn đơn giản; đây không phải kết quả của simulator nên không so được với baseline LOC-30."
    );
    println!("uptime lúc kết thúc: {}", uptime());
}
