//! Tiện ích dùng chung cho các lab (đường dẫn, tham số dòng lệnh, đo thời gian, số ngẫu nhiên có seed).

use road_graph::Graph;
use std::path::PathBuf;
use std::time::Instant;

/// Thư mục `examples/toi-noi-location`.
pub fn lab_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("lab root")
}
pub fn fixture_path() -> PathBuf {
    lab_root().join("fixtures/khu-mau.json")
}
pub fn pbf_path() -> PathBuf {
    lab_root().join("data/vietnam-latest.osm.pbf")
}
pub fn graph_bin_path() -> PathBuf {
    lab_root().join("data/hcm-graph.bin")
}
pub fn results_dir() -> PathBuf {
    lab_root().join("results")
}

/// Thoát với thông báo rõ ràng và mã lỗi 2 (thiếu dữ liệu hoặc hạ tầng).
pub fn die(msg: &str) -> ! {
    eprintln!("LỖI: {msg}");
    std::process::exit(2)
}

pub struct Args(Vec<String>);

impl Args {
    pub fn from_env() -> Self {
        Self(std::env::args().skip(1).collect())
    }
    pub fn flag(&self, name: &str) -> bool {
        self.0.iter().any(|a| a == name)
    }
    pub fn value(&self, name: &str) -> Option<&str> {
        self.0
            .iter()
            .position(|a| a == name)
            .and_then(|i| self.0.get(i + 1))
            .map(String::as_str)
    }
    pub fn u64_or(&self, name: &str, default: u64) -> u64 {
        self.value(name).map_or(default, |v| {
            v.parse()
                .unwrap_or_else(|_| die(&format!("{name} cần một số nguyên")))
        })
    }
}

/// Nạp đồ thị: `--fixture` dùng đồ thị tổng hợp, mặc định đọc `data/hcm-graph.bin` do lab LOC-06 biên dịch.
pub fn load_graph(args: &Args) -> (Graph, String) {
    if args.flag("--fixture") {
        let g = road_graph::fixture::load_fixture(&fixture_path())
            .unwrap_or_else(|e| die(&format!("không nạp được fixture: {e}")));
        return (g, "fixtures/khu-mau.json (đồ thị tổng hợp)".into());
    }
    let path = args
        .value("--graph")
        .map_or_else(graph_bin_path, PathBuf::from);
    if !path.exists() {
        die(&format!(
            "thiếu {}. Chạy lab LOC-06 trước (cần data/vietnam-latest.osm.pbf, xem data/fetch.sh), hoặc thêm --fixture để dùng đồ thị tổng hợp",
            path.display()
        ));
    }
    let g = road_graph::io::load(&path)
        .unwrap_or_else(|e| die(&format!("không nạp được {}: {e}", path.display())));
    (g, path.display().to_string())
}

/// Chạy `f` và trả (kết quả, giây).
pub fn timed<T>(f: impl FnOnce() -> T) -> (T, f64) {
    let t = Instant::now();
    let r = f();
    (r, t.elapsed().as_secs_f64())
}

/// Số ngẫu nhiên có seed (fastrand), thêm Gaussian Box-Muller.
pub struct Rand(pub fastrand::Rng);

impl Rand {
    pub fn new(seed: u64) -> Self {
        Self(fastrand::Rng::with_seed(seed))
    }
    pub fn f64(&mut self) -> f64 {
        self.0.f64()
    }
    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.0.f64()
    }
    pub fn gauss(&mut self) -> f64 {
        let (u1, u2) = (1.0 - self.0.f64(), self.0.f64());
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
    pub fn usize(&mut self, n: usize) -> usize {
        self.0.usize(..n)
    }
}

/// Phân vị trên mẫu đã sắp xếp (nearest-rank).
pub fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let idx = ((p / 100.0) * sorted.len() as f64).ceil() as usize;
    sorted[idx.saturating_sub(1).min(sorted.len() - 1)]
}
