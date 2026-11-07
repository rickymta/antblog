//! Tiện ích chung cho lab P02 (LOC-15 … LOC-18): nạp thế giới (đồ thị + chỉ mục snap), sinh chuyến có ground truth, đo tải máy.
//!
//! Hai bộ seed tách nhau: [`TUNE_SEED`] chọn tham số, [`EVAL_SEED`] báo cáo. Không bao giờ chỉnh tham số trên bộ báo cáo.

use crate::{Args, load_graph, percentile};
use map_match::Rng;
use map_match::route::Router;
use map_match::scenario::random_trip;
use map_match::trace::{DriveConfig, NoiseConfig, Trace, TruthPoint, corrupt, drive};
use road_graph::snap::SnapIndex;
use road_graph::{EdgeId, Graph, Mode};

/// Seed báo cáo (giống mọi lab của loạt).
pub const EVAL_SEED: u64 = 20_261_106;
/// Seed chọn tham số: tập chuyến và nhiễu khác hẳn tập báo cáo.
pub const TUNE_SEED: u64 = 1_106;

pub struct World {
    pub g: Graph,
    pub idx: SnapIndex,
    pub source: String,
}

pub fn world(args: &Args) -> World {
    let (g, source) = load_graph(args);
    // đường dẫn tương đối so với examples/toi-noi-location cho output gọn và không lộ thư mục làm việc
    let source = std::path::Path::new(&source)
        .strip_prefix(crate::lab_root())
        .map_or(source.clone(), |p| p.display().to_string());
    let idx = SnapIndex::build(&g);
    World { g, idx, source }
}

/// Kết quả lệnh `uptime` (tải máy lúc đo; máy viết dùng chung nên số thời gian nhiễu).
pub fn uptime() -> String {
    std::process::Command::new("uptime")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map_or_else(
            || "(không chạy được uptime)".into(),
            |s| s.trim().to_string(),
        )
}

pub fn header(title: &str, w: &World, extra: &str) {
    println!("== {title} ==");
    println!(
        "đồ thị: {}: {} nút, {} cạnh có hướng; {extra}",
        w.source,
        w.g.node_count(),
        w.g.edge_count()
    );
    println!("uptime lúc bắt đầu: {}", uptime());
    println!(
        "lưu ý: nhiễu GPS do chính lab sinh ra (có seed); mọi độ chính xác dưới đây là trên nhiễu tổng hợp này, không phải GPS thật.\n"
    );
}

/// Một chuyến: lộ trình và trạng thái thật theo từng lần lấy mẫu.
pub struct Trip {
    pub route: Vec<EdgeId>,
    pub truth: Vec<TruthPoint>,
}

/// `n` chuyến ngẫu nhiên theo đường ngắn nhất, xe chạy thật (tăng tốc, phanh, dừng đèn). Đồ thị HCM: dài 2–4,5 km;
/// đồ thị fixture nhỏ (dưới 5.000 nút): dài 0,4–1,5 km vì khu mẫu chỉ rộng khoảng 1,5 km.
pub fn trips(w: &World, n: usize, seed: u64, drive_cfg: &DriveConfig) -> Vec<Trip> {
    let (lo, hi) = if w.g.node_count() < 5_000 {
        (400.0, 1_500.0)
    } else {
        (2_000.0, 4_500.0)
    };
    let mut rng = Rng::new(seed);
    let mut router = Router::new(&w.g);
    let mut out = Vec::with_capacity(n);
    let mut failures = 0;
    while out.len() < n {
        let Some(route) = random_trip(&w.g, &mut router, Mode::Car, &mut rng, lo, hi) else {
            failures += 1;
            if failures > 200 {
                crate::die("không sinh được chuyến ngẫu nhiên trên đồ thị này");
            }
            continue;
        };
        let mut r = Rng::new(seed.wrapping_mul(7919).wrapping_add(out.len() as u64));
        let truth = drive(&w.g, &route, drive_cfg, &mut r);
        out.push(Trip { route, truth });
    }
    out
}

/// Làm hỏng cùng tập chuyến bằng `noise` (cùng chuyến, nhiễu khác thì so sánh công bằng).
pub fn corrupt_all(trips: &[Trip], noise: &NoiseConfig, seed: u64) -> Vec<Trace> {
    trips
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let mut rng = Rng::new(seed ^ (i as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
            corrupt(&t.route, t.truth.clone(), noise, &mut rng)
        })
        .collect()
}

pub fn rmse(sq_sum: f64, n: usize) -> f64 {
    (sq_sum / n.max(1) as f64).sqrt()
}

pub fn p95(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    percentile(&v, 95.0)
}
