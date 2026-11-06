//! Tiện ích dùng chung cho các lab P05 (ghép chuyến): nạp engine, kịch bản và biến thể, in bảng so sánh có cặp.

use crate::{Args, die, lab_root, load_graph};
use geo_core::LatLng;
use matcher::eval::{Paired, RunMetrics, ZONE_NAMES, column, paired};
use road_graph::Graph;
use route_engine::engine::{Engine, EngineConfig};
use route_engine::position::Position;
use route_engine::profile::Profile;
use simulator::Scenario;

/// Nạp đồ thị, dựng engine (CH ô tô nạp từ cache ở `rust/target/ch-cache` nếu là đồ thị HCM) và kịch bản.
/// Trả (engine, mô tả đồ thị, kịch bản, cờ fixture).
pub fn setup(args: &Args) -> (Engine, String, Scenario, bool) {
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
    let sc = Scenario::load(&sc_path).unwrap_or_else(|e| die(&format!("{e}")));
    let info = format!(
        "{source}: {} nút, {} cạnh",
        crate::p03::vn(g.node_count() as u64),
        crate::p03::vn(g.edge_count() as u64)
    );
    let cfg = EngineConfig {
        ch_cache_dir: (!fixture).then(|| lab_root().join("rust/target/ch-cache")),
        cache_tag: if fixture { "fixture" } else { "hcm" }.into(),
        ..EngineConfig::default()
    };
    (Engine::new(g, cfg), info, sc, fixture)
}

/// Số luồng chạy song song các seed (mặc định 3; máy dùng chung nên đừng đặt cao).
pub fn jobs(args: &Args) -> usize {
    args.u64_or("--jobs", 3) as usize
}

/// Số seed đánh giá (mặc định 20; tối thiểu 20 khi báo cáo cho bài).
pub fn n_seeds(args: &Args) -> usize {
    args.u64_or("--seeds", 20) as usize
}

pub type Tweak = fn(&mut Scenario);

/// Các bộ giả định để thử độ nhạy: mật độ tài xế, mức cầu, mô hình huỷ. Tên cố định để các bài trích dẫn.
pub fn variants() -> Vec<(&'static str, &'static str, Tweak)> {
    vec![
        ("thường", "120 xe, 3,5 chuyến/phút (baseline P03)", |_| {}),
        (
            "cao điểm",
            "120 xe, 7,0 chuyến/phút (cầu ×2)",
            |s| {
                s.demand.requests_per_min *= 2.0;
            },
        ),
        ("thưa xe", "60 xe, 3,5 chuyến/phút", |s| s.drivers = 60),
        ("dày xe", "240 xe, 3,5 chuyến/phút", |s| s.drivers = 240),
        (
            "huỷ theo chờ",
            "120 xe, 3,5/phút, huỷ tính theo tổng thời gian chờ từ lúc đặt (cùng đường cong 0,5 từ 180 s đến 900 s)",
            |s| s.cancel.basis = simulator::scenario::CancelBasis::TotalWait,
        ),
        (
            "huỷ nặng",
            "120 xe, 3,5/phút, huỷ: tối đa 0,8 từ ETA 120 s (nhạy ETA hơn baseline)",
            |s| {
                s.cancel.max_prob = 0.8;
                s.cancel.eta_free_s = 120.0;
            },
        ),
    ]
}

/// Chọn bộ giả định theo `--variant tên1,tên2` (tên như ở [`variants`]); không có cờ thì dùng `default`.
pub fn select_variants(args: &Args, default: &[&str]) -> Vec<(&'static str, &'static str, Tweak)> {
    let wanted: Vec<String> = args.value("--variant").map_or_else(
        || default.iter().map(|s| (*s).to_string()).collect(),
        |v| v.split(',').map(|s| s.trim().to_string()).collect(),
    );
    variants()
        .into_iter()
        .filter(|(n, _, _)| wanted.iter().any(|w| w == n))
        .collect()
}

pub fn f1(x: f64) -> String {
    if x.is_nan() {
        "n/a".into()
    } else {
        format!("{x:.1}")
    }
}
pub fn f3(x: f64) -> String {
    if x.is_nan() {
        "n/a".into()
    } else {
        format!("{x:.3}")
    }
}

/// Điểm đón từ một ghim: (điểm dừng, các vị trí có hướng vào được), giống `AtPin`.
pub fn pickup_of(engine: &Engine, pin: LatLng) -> Option<(LatLng, Vec<Position>)> {
    let s = engine.snap_point(Profile::Car, pin, "ghim").ok()?;
    let ps = s.positions(engine.view(Profile::Car));
    (!ps.is_empty()).then_some((s.point, ps))
}

type Getter = fn(&RunMetrics) -> f64;

/// (tên, thấp hơn là tốt hơn, cách lấy, số chữ số thập phân)
pub const METRICS: &[(&str, bool, Getter, usize)] = &[
    ("deadhead_m trung bình", true, |r| r.deadhead_mean, 1),
    ("deadhead_m p50", true, |r| r.deadhead_p50, 1),
    ("deadhead_m p95", true, |r| r.deadhead_p95, 1),
    ("deadhead_m max", true, |r| r.deadhead_max, 1),
    ("pickup_eta_s trung bình", true, |r| r.eta_mean, 1),
    ("pickup_eta_s p95", true, |r| r.eta_p95, 1),
    ("wait_s trung bình", true, |r| r.wait_mean, 1),
    ("wait_s p50", true, |r| r.wait_p50, 1),
    ("wait_s p95", true, |r| r.wait_p95, 1),
    ("wait_s max", true, |r| r.wait_max, 1),
    ("walk_m trung bình", true, |r| r.walk_mean, 2),
    ("trip_m trung bình", false, |r| r.trip_mean, 1),
    ("u_turns mỗi chuyến", true, |r| r.u_turns_per_trip, 3),
    ("cancel_rate", true, |r| r.cancel_rate, 4),
    (
        "unserved_rate (hết kiên nhẫn)",
        true,
        |r| r.unserved_rate,
        4,
    ),
    ("chuyến hoàn thành", false, |r| r.completed as f64, 1),
    (
        "km rỗng tổng / khách đặt (m)",
        true,
        |r| r.empty_m_per_request,
        1,
    ),
];

fn verdict(p: &Paired, lower_better: bool) -> &'static str {
    if !p.significant() {
        "không phân biệt được"
    } else if (p.mean_diff < 0.0) == lower_better {
        "TỐT HƠN"
    } else {
        "XẤU HƠN"
    }
}

fn fmt_p(v: f64, d: usize) -> String {
    if v.is_nan() {
        "n/a".into()
    } else {
        format!("{v:.d$}")
    }
}

/// In bảng so sánh có cặp: mỗi dòng một chỉ số, hiệu = chiến lược − baseline, KTC 95% (t Student trên hiệu từng seed).
pub fn print_paired(title: &str, a_name: &str, b_name: &str, a: &[RunMetrics], b: &[RunMetrics]) {
    println!("\n{title}");
    println!(
        "  A = {a_name}; B = {b_name}; hiệu = B − A; n = {} seed ghép cặp; KTC 95% theo t Student trên hiệu từng seed",
        a.len()
    );
    println!(
        "  {:<32} {:>10} {:>10} {:>10}  {:<24} {:>10} {:>10} {:>7}  nhận xét",
        "chỉ số", "A (TB)", "B (TB)", "hiệu TB", "KTC 95%", "trung vị A", "trung vị B", "B<A/B>A"
    );
    for &(name, lower, get, d) in METRICS {
        let p = paired(&column(a, get), &column(b, get));
        println!(
            "  {:<32} {:>10} {:>10} {:>10}  {:<24} {:>10} {:>10} {:>3}/{:<3}  {}",
            name,
            fmt_p(p.mean_a, d),
            fmt_p(p.mean_b, d),
            fmt_p(p.mean_diff, d),
            format!("[{}, {}]", fmt_p(p.ci_lo, d), fmt_p(p.ci_hi, d)),
            fmt_p(p.med_a, d),
            fmt_p(p.med_b, d),
            p.b_lower,
            p.b_higher,
            verdict(&p, lower)
        );
    }
}

/// Bảng theo vùng (khách phân vùng theo điểm ghim): số khách, tỉ lệ hoàn thành, ETA, thời gian chờ, km rỗng; hiệu có cặp.
pub fn print_zones(a_name: &str, b_name: &str, a: &[RunMetrics], b: &[RunMetrics]) {
    println!(
        "\n  Theo vùng (vùng của khách theo khoảng cách ghim tới Chợ Bến Thành); hiệu = B − A, KTC 95% có cặp:"
    );
    println!(
        "  {:<22} {:>10} {:>24} {:>28} {:>28} {:>28}",
        "vùng",
        "khách/seed",
        "tỉ lệ hoàn thành A → B",
        "wait_s TB hiệu [KTC]",
        "pickup_eta_s TB hiệu [KTC]",
        "deadhead_m TB hiệu [KTC]"
    );
    for (z, zn) in ZONE_NAMES.iter().enumerate() {
        let req = column(a, |r| r.zone[z].requests as f64);
        let comp = |rs: &[RunMetrics]| {
            column(rs, |r| {
                r.zone[z].completed as f64 / (r.zone[z].requests.max(1)) as f64
            })
        };
        let (ca, cb) = (comp(a), comp(b));
        let pc = paired(&ca, &cb);
        let w = paired(
            &column(a, |r| r.zone[z].wait_mean),
            &column(b, |r| r.zone[z].wait_mean),
        );
        let e = paired(
            &column(a, |r| r.zone[z].eta_mean),
            &column(b, |r| r.zone[z].eta_mean),
        );
        let d = paired(
            &column(a, |r| r.zone[z].deadhead_mean),
            &column(b, |r| r.zone[z].deadhead_mean),
        );
        let cell = |p: &Paired| {
            format!(
                "{:+.1} [{:.1}, {:.1}]{}",
                p.mean_diff,
                p.ci_lo,
                p.ci_hi,
                if p.significant() { " *" } else { "" }
            )
        };
        println!(
            "  {:<22} {:>10.1} {:>24} {:>28} {:>28} {:>28}",
            zn,
            req.iter().sum::<f64>() / req.len().max(1) as f64,
            format!(
                "{:.3} → {:.3}{}",
                pc.mean_a,
                pc.mean_b,
                if pc.significant() { " *" } else { "" }
            ),
            cell(&w),
            cell(&e),
            cell(&d)
        );
    }
    println!("  (* = KTC 95% không chứa 0)  A = {a_name}; B = {b_name}");
}
