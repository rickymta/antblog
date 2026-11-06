//! Tiện ích dùng chung cho các lab P04 (điểm đón, điểm trả): nạp engine, hai dải seed tách bạch, chạy mô phỏng cặp
//! (cùng seed, chiến lược so với baseline) song song, và in bảng so sánh có khoảng tin cậy.
//!
//! Quy tắc đánh giá của phần này:
//! - **Hai dải seed không giao nhau**: seed chỉnh trọng số (`TUNING_SEEDS`) và seed đánh giá (`EVAL_SEEDS`, ≥ 20).
//!   Mọi khẳng định tối ưu được báo trên seed đánh giá, chưa từng dùng để chỉnh.
//! - Mọi so sánh là **cặp**: cùng seed, cùng kịch bản, chiến lược A so với baseline `AtPin` + `NearestStraightLine`.
//! - Báo mọi chỉ số, kể cả chỉ số xấu đi, kèm đuôi (p95, max).

use crate::{Args, die, lab_root, load_graph};
use pickup::dropoff::{BaselineLogged, DropEntry, DropLog};
use pickup::stats::{Paired, paired};
use pickup::strategy::{ChoiceLog, JConfig, JPickup};
use road_graph::Graph;
use route_engine::engine::{Engine, EngineConfig};
use simulator::metrics::TripRecord;
use simulator::{AtPin, NearestStraightLine, Outcome, RunOptions, Scenario, SimOutput, run};
use std::cell::RefCell;
use std::rc::Rc;

/// Seed dùng để chỉnh trọng số (10 seed). Không dùng để báo kết quả.
pub const TUNING_SEEDS: std::ops::RangeInclusive<u64> = 20_261_201..=20_261_210;
/// Seed đánh giá (20 seed), chưa từng dùng để chỉnh.
pub const EVAL_SEEDS: std::ops::RangeInclusive<u64> = 20_270_001..=20_270_020;

pub struct Env {
    pub engine: Engine,
    pub scenario: Scenario,
    pub graph_info: String,
    pub fixture: bool,
}

/// Nạp đồ thị, dựng engine (CH ô tô nạp từ cache nếu là đồ thị HCM) và kịch bản.
pub fn setup(args: &Args) -> Env {
    let (g, source): (Graph, String) = load_graph(args);
    let fixture = args.flag("--fixture");
    let sc_path = args.value("--scenario").map_or_else(
        || {
            lab_root().join(if fixture {
                "fixtures/scenario-khu-mau.toml"
            } else {
                "fixtures/scenario-hcm-baseline.toml"
            })
        },
        std::path::PathBuf::from,
    );
    let scenario = Scenario::load(&sc_path).unwrap_or_else(|e| die(&format!("{e}")));
    let graph_info = format!(
        "{source}: {} nút, {} cạnh",
        crate::p03::vn(g.node_count() as u64),
        crate::p03::vn(g.edge_count() as u64)
    );
    let cfg = EngineConfig {
        ch_cache_dir: (!fixture).then(|| lab_root().join("rust/target/ch-cache")),
        cache_tag: if fixture { "fixture" } else { "hcm" }.into(),
        ..EngineConfig::default()
    };
    Env {
        engine: Engine::new(g, cfg),
        scenario,
        graph_info,
        fixture,
    }
}

pub fn jobs(args: &Args) -> usize {
    args.u64_or("--jobs", 3) as usize
}

/// Dải seed theo cờ: `--seeds N` lấy N seed đầu của dải đánh giá (mặc định 20), `--tuning` dùng dải chỉnh.
pub fn seeds(args: &Args, tuning: bool) -> Vec<u64> {
    let n = args.u64_or("--seeds", if tuning { 10 } else { 20 }) as usize;
    let range = if tuning || args.flag("--tuning") {
        TUNING_SEEDS
    } else {
        EVAL_SEEDS
    };
    range.take(n).collect()
}

/// Kết quả một lần chạy mô phỏng cùng các nhật ký của chiến lược.
pub struct RunResult {
    pub out: SimOutput,
    pub choices: Vec<ChoiceLog>,
    pub drops: Vec<DropEntry>,
}

pub fn run_baseline(engine: &Engine, sc: &Scenario, log_drop: bool) -> RunResult {
    let log = Rc::new(RefCell::new(DropLog::default()));
    let out = if log_drop {
        run(
            engine,
            sc,
            Box::new(BaselineLogged {
                log: log.clone(),
                gate_radius_m: 80.0,
            }),
            Box::new(NearestStraightLine),
            RunOptions::default(),
        )
    } else {
        run(
            engine,
            sc,
            Box::new(AtPin),
            Box::new(NearestStraightLine),
            RunOptions::default(),
        )
    };
    let drops = log.borrow().entries.clone();
    RunResult {
        out,
        choices: Vec::new(),
        drops,
    }
}

pub fn run_j(engine: &Engine, sc: &Scenario, cfg: &JConfig) -> RunResult {
    let strat = JPickup::new(cfg.clone());
    let (log, dlog) = (strat.log.clone(), strat.drop_log.clone());
    let out = run(
        engine,
        sc,
        Box::new(strat),
        Box::new(NearestStraightLine),
        RunOptions::default(),
    );
    let choices = log.borrow().clone();
    let drops = dlog.borrow().entries.clone();
    RunResult {
        out,
        choices,
        drops,
    }
}

/// Chạy `f(seed)` song song trên `jobs` luồng, giữ nguyên thứ tự seed.
pub fn par_seeds<T: Send>(
    jobs: usize,
    seeds: &[u64],
    f: impl Fn(u64) -> T + Sync + Send,
) -> Vec<T> {
    use rayon::prelude::*;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs.max(1))
        .build()
        .expect("thread pool");
    pool.install(|| seeds.par_iter().map(|&s| f(s)).collect())
}

pub fn with_seed(sc: &Scenario, seed: u64) -> Scenario {
    let mut s = sc.clone();
    s.seed = seed;
    s
}

/// Các bộ giả định để thử độ nhạy (mỗi cái đổi MỘT giả định so với baseline P03). Tên cố định để các bài trích dẫn.
/// Một bộ giả định: tên, mô tả, hàm đổi kịch bản.
pub type Variant = (&'static str, &'static str, fn(&mut Scenario));

pub fn variants() -> Vec<Variant> {
    vec![
        (
            "gốc",
            "kịch bản baseline P03 (120 xe, đi bộ 1,3 m/s, huỷ chỉ theo ETA)",
            |_| {},
        ),
        (
            "đi bộ chậm",
            "tốc độ đi bộ 0,9 m/s (người lớn tuổi, xách đồ)",
            |s| s.customer.walk_speed_mps = 0.9,
        ),
        (
            "huỷ theo đi bộ",
            "mỗi 100 m đi bộ cộng 0,05 xác suất huỷ (mô hình huỷ có tính quãng đi bộ)",
            |s| s.cancel.walk_extra_per_100m = 0.05,
        ),
        (
            "ít xe",
            "60 tài xế (mật độ giảm một nửa)",
            |s| s.drivers = 60,
        ),
        (
            "nhiều xe",
            "240 tài xế (mật độ gấp đôi)",
            |s| s.drivers = 240,
        ),
    ]
}

pub fn f1(x: f64) -> String {
    if x.is_nan() {
        "n/a".into()
    } else {
        let s = format!("{x:.1}");
        s
    }
}

/// Quãng khách đi bộ + quãng tài xế chạy không của mỗi chuyến hoàn thành (mét): «tổng quãng của hai người».
pub fn combined_mean(out: &SimOutput) -> f64 {
    let v: Vec<f64> = out
        .trips
        .iter()
        .filter(|t| t.outcome == Outcome::Completed)
        .map(|t| t.walk_m + t.deadhead_m)
        .collect();
    crate::p03::mean(&v)
}

fn fare_mean(out: &SimOutput) -> f64 {
    let v: Vec<f64> = out
        .trips
        .iter()
        .filter(|t| t.outcome == Outcome::Completed)
        .map(|t| pickup::fare::fare_vnd(t.trip_m.round() as u64) as f64)
        .collect();
    crate::p03::mean(&v)
}

/// Một chỉ số theo seed: tên, hàm lấy giá trị, ghi chú «thấp hơn là tốt» để tô ai được lợi.
type Getter = Box<dyn Fn(&SimOutput) -> f64>;

pub fn metric_defs() -> Vec<(&'static str, Getter)> {
    fn g(f: impl Fn(&SimOutput) -> f64 + 'static) -> Getter {
        Box::new(f)
    }
    vec![
        ("walk_m trung bình", g(|o| o.summary.walk_m.mean)),
        ("walk_m p50", g(|o| o.summary.walk_m.p50)),
        ("walk_m p95", g(|o| o.summary.walk_m.p95)),
        ("walk_m max", g(|o| o.summary.walk_m.max)),
        ("deadhead_m trung bình", g(|o| o.summary.deadhead_m.mean)),
        ("deadhead_m p50", g(|o| o.summary.deadhead_m.p50)),
        ("deadhead_m p95", g(|o| o.summary.deadhead_m.p95)),
        ("deadhead_m max", g(|o| o.summary.deadhead_m.max)),
        (
            "pickup_eta_s trung bình",
            g(|o| o.summary.pickup_eta_s.mean),
        ),
        ("pickup_eta_s p50", g(|o| o.summary.pickup_eta_s.p50)),
        ("pickup_eta_s p95", g(|o| o.summary.pickup_eta_s.p95)),
        ("meet_gap_s trung bình", g(|o| o.summary.meet_gap_s.mean)),
        ("meet_gap_s p95", g(|o| o.summary.meet_gap_s.p95)),
        ("trip_m trung bình", g(|o| o.summary.trip_m.mean)),
        ("u_turns mỗi chuyến", g(|o| o.summary.u_turns_per_trip)),
        ("cancel_rate", g(|o| o.summary.cancel_rate)),
        ("chuyến hoàn thành", g(|o| o.summary.completed as f64)),
        ("walk_m + deadhead_m mỗi chuyến", g(combined_mean)),
        ("cước trung bình (đồng)", g(fare_mean)),
    ]
}

/// In bảng so sánh cặp mọi chỉ số; trả các `Paired` theo thứ tự `metric_defs`.
pub fn print_paired(
    title: &str,
    base: &[RunResult],
    new: &[RunResult],
) -> Vec<(&'static str, Paired)> {
    println!("{title}");
    println!(
        "  n = {} seed ghép cặp (cùng seed, chiến lược trừ baseline). Khoảng tin cậy 95%: t và bootstrap (4000 lần). «↓/↑» = số seed thấp hơn/cao hơn baseline.",
        base.len()
    );
    println!(
        "  {:<32} {:>10} {:>10} {:>10} {:>21} {:>21} {:>7}  {:>17}",
        "chỉ số",
        "baseline",
        "chiến lược",
        "chênh TB",
        "KTC 95% (t)",
        "KTC 95% (bootstrap)",
        "↓/↑",
        "trung vị seed b→n"
    );
    let mut res = Vec::new();
    for (name, get) in metric_defs() {
        let b: Vec<f64> = base.iter().map(|r| get(&r.out)).collect();
        let n: Vec<f64> = new.iter().map(|r| get(&r.out)).collect();
        let p = paired(&b, &n);
        let d = if name == "cancel_rate" { 4 } else { 1 };
        let fmt = |x: f64| format!("{x:.d$}");
        println!(
            "  {:<32} {:>10} {:>10} {:>10} {:>21} {:>21} {:>7}  {:>17}{}",
            name,
            fmt(p.mean_base),
            fmt(p.mean_new),
            format!("{:+.d$}", p.mean_diff),
            format!("[{:+.d$}, {:+.d$}]", p.t_lo, p.t_hi),
            format!("[{:+.d$}, {:+.d$}]", p.boot_lo, p.boot_hi),
            format!("{}/{}", p.lower, p.higher),
            format!("{}→{}", fmt(p.median_base), fmt(p.median_new)),
            if p.significant() { "  *" } else { "" }
        );
        res.push((name, p));
    }
    println!("  (* = khoảng tin cậy t không chứa 0)");
    res
}

/// Cặp (baseline, mới) cho các khách hoàn thành ở cả hai lần chạy cùng seed.
pub fn customer_pairs<'a>(
    base: &'a SimOutput,
    new: &'a SimOutput,
) -> Vec<(&'a TripRecord, &'a TripRecord)> {
    let done = |o: &'a SimOutput| -> std::collections::BTreeMap<u32, &'a TripRecord> {
        o.trips
            .iter()
            .filter(|t| t.outcome == Outcome::Completed)
            .map(|t| (t.customer, t))
            .collect()
    };
    let (b, n) = (done(base), done(new));
    b.iter()
        .filter_map(|(c, tb)| n.get(c).map(|tn| (*tb, *tn)))
        .collect()
}

/// Ai được lợi, ai chịu thiệt: phân bố chênh lệch `walk_m` của từng khách (mới − baseline) gộp mọi seed.
pub fn print_who_pays(base: &[RunResult], new: &[RunResult]) {
    let mut dw = Vec::new();
    let mut tail_dw = Vec::new();
    let mut head_dw = Vec::new();
    let (mut worse50, mut worse100, mut better50) = (0usize, 0usize, 0usize);
    for (b, n) in base.iter().zip(new) {
        for (tb, tn) in customer_pairs(&b.out, &n.out) {
            let d = tn.walk_m - tb.walk_m;
            dw.push(d);
            if tb.walk_m > 300.0 {
                tail_dw.push(d);
            } else {
                head_dw.push(d);
            }
            worse50 += usize::from(d > 50.0);
            worse100 += usize::from(d > 100.0);
            better50 += usize::from(d < -50.0);
        }
    }
    let pct = |k: usize, n: usize| 100.0 * k as f64 / n.max(1) as f64;
    let mut s = dw.clone();
    s.sort_by(f64::total_cmp);
    let q = |p: f64| simulator::metrics::percentile(&s, p);
    println!(
        "  theo từng khách (cùng seed, cùng khách, hoàn thành ở cả hai; {} cặp): chênh walk_m mới − baseline: p5 {:+.1}  p50 {:+.1}  p95 {:+.1}  min {:+.1}  max {:+.1} m",
        dw.len(),
        q(5.0),
        q(50.0),
        q(95.0),
        s.first().copied().unwrap_or(f64::NAN),
        s.last().copied().unwrap_or(f64::NAN)
    );
    println!(
        "  đi bộ nhiều hơn > 50 m: {:.1}% khách; > 100 m: {:.1}%; ít hơn > 50 m: {:.1}%",
        pct(worse50, dw.len()),
        pct(worse100, dw.len()),
        pct(better50, dw.len())
    );
    println!(
        "  phân tầng theo baseline: khách có walk_m baseline > 300 m (đuôi): {} cặp, chênh TB {:+.1} m; còn lại: {} cặp, chênh TB {:+.1} m",
        tail_dw.len(),
        crate::p03::mean(&tail_dw),
        head_dw.len(),
        crate::p03::mean(&head_dw)
    );
}

/// Hệ số `uptime` cho dòng cấu hình.
pub fn config_line(env: &Env, seeds: &[u64], extra: &str) -> String {
    format!(
        "cấu hình: kịch bản {} (seed gốc {}), đồ thị {}, seed {}…{} ({} seed), {extra}",
        env.scenario.name,
        env.scenario.seed,
        env.graph_info,
        seeds.first().copied().unwrap_or(0),
        seeds.last().copied().unwrap_or(0),
        seeds.len()
    )
}

/// Baseline `AtPin` có ghi lại điểm đón của từng khách: «lịch sử đón» mà LOC-39 học từ đó (đón ở đâu thì ghi ở đó).
pub struct LoggedStops {
    pub stops: Rc<RefCell<Vec<(u32, geo_core::LatLng, geo_core::LatLng)>>>,
}

impl simulator::PickupStrategy for LoggedStops {
    fn name(&self) -> &str {
        "at-pin"
    }
    fn choose(
        &mut self,
        ctx: &simulator::strategy::Ctx,
        c: &simulator::strategy::CustomerView,
    ) -> Option<simulator::strategy::PickupChoice> {
        let r = AtPin.choose(ctx, c);
        if let Some(ch) = &r {
            self.stops.borrow_mut().push((c.id, ch.stop_point, c.pin));
        }
        r
    }
}

/// Chạy baseline và trả về (kết quả, điểm đón của từng khách: mã, điểm dừng, ghim).
pub fn run_history(
    engine: &Engine,
    sc: &Scenario,
) -> (SimOutput, Vec<(u32, geo_core::LatLng, geo_core::LatLng)>) {
    let stops = Rc::new(RefCell::new(Vec::new()));
    let out = run(
        engine,
        sc,
        Box::new(LoggedStops {
            stops: stops.clone(),
        }),
        Box::new(NearestStraightLine),
        RunOptions::default(),
    );
    let v = stops.borrow().clone();
    (out, v)
}

/// Đọc bảng điểm đón học được (CSV có cột `lat`, `lng`).
pub fn read_hubs(path: &std::path::Path) -> Vec<geo_core::LatLng> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| die(&format!("không đọc được {}: {e}", path.display())));
    let mut lines = text.lines();
    let head: Vec<&str> = lines.next().unwrap_or("").split(',').collect();
    let (ci, cj) = (
        head.iter().position(|h| *h == "lat").unwrap_or(1),
        head.iter().position(|h| *h == "lng").unwrap_or(2),
    );
    lines
        .filter_map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            geo_core::LatLng::try_new(f.get(ci)?.parse().ok()?, f.get(cj)?.parse().ok()?).ok()
        })
        .collect()
}
