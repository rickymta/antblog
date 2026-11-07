//! LOC-53: khi mô hình lỗi thì sao. `EtaService` luôn trả ETA: mô hình thiếu, hỏng, sai phiên bản, sai đặc trưng, backend không nạp được,
//! trả NaN hay số vô lý ⇒ fallback về ETA gốc của route-engine, kèm lý do và bộ đếm.
//!
//! Chạy từ gốc repo (cần `loc53_onnx_export.py` chạy trước):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc53_fallback
//! Nếu đặt ORT_DYLIB_PATH, lab thêm một dòng thử backend ort; không đặt thì dòng đó cho thấy ort không nạp được và dịch vụ fallback.

use eta_model::{EtaError, EtaService, FEATURES_V1, FallbackReason, ModelFiles, Predictor, Source};
use labs::p03::uptime;
use labs::{Args, die, lab_root};
use std::path::{Path, PathBuf};

fn tract_make(p: &Path) -> Result<Box<dyn Predictor>, EtaError> {
    Ok(Box::new(eta_model::backend_tract::TractModel::load(
        p,
        FEATURES_V1.len(),
        1,
    )?))
}
fn ort_make(p: &Path) -> Result<Box<dyn Predictor>, EtaError> {
    Ok(Box::new(eta_model::backend_ort::OrtModel::load(
        p,
        FEATURES_V1.len(),
        1,
    )?))
}

/// Bộ dự đoán bọc mô hình thật và cứ `every` lần thì hỏng (lỗi, NaN, hoặc số vô lý) để đo hành vi fallback.
struct Flaky {
    inner: Box<dyn Predictor>,
    every: u64,
    calls: std::sync::atomic::AtomicU64,
}
impl Predictor for Flaky {
    fn backend(&self) -> &'static str {
        self.inner.backend()
    }
    fn n_features(&self) -> usize {
        self.inner.n_features()
    }
    fn predict_one(&self, f: &[f32]) -> Result<f32, EtaError> {
        let k = self
            .calls
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if k % self.every == self.every - 1 {
            return match (k / self.every) % 3 {
                0 => Err(EtaError::Backend {
                    backend: "flaky",
                    msg: "lỗi giả lập".into(),
                }),
                1 => Ok(f32::NAN),
                _ => Ok(1.0e6),
            };
        }
        self.inner.predict_one(f)
    }
}

fn show(name: &str, svc: &EtaService, route_eta: f64, f: &[f32]) {
    let e = svc.estimate(route_eta, f);
    let src = match &e.source {
        Source::Model { version, backend } => format!("MÔ HÌNH {version} qua {backend}"),
        Source::Fallback(FallbackReason::NotLoaded(why)) => {
            format!("FALLBACK (không nạp được): {why}")
        }
        Source::Fallback(r) => format!("FALLBACK ({r:?})"),
    };
    println!(
        "  {name:<46} ETA gốc {route_eta:>7.1} s → trả {:>7.1} s  [{src}]",
        e.eta_s
    );
}

fn main() {
    let args = Args::from_env();
    let dir: PathBuf = args.value("--model-dir").map_or_else(
        || lab_root().join("python/p06/artifacts/model"),
        PathBuf::from,
    );
    println!("== LOC-53: mô hình lỗi thì fallback (trên dữ liệu mô phỏng) ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    let version = "eta-resid-v1-cls";
    let text = std::fs::read_to_string(dir.join("eta_test_inputs.csv"))
        .unwrap_or_else(|e| die(&format!("cần loc53_onnx_export.py chạy trước: {e}")));
    let rows: Vec<Vec<f32>> = text
        .lines()
        .take(200)
        .map(|l| {
            l.split(',')
                .map(|x| x.parse().unwrap_or(f32::NAN))
                .collect()
        })
        .collect();
    let row = &rows[0];
    let eta = f64::from(row[0]);

    println!(
        "\n-- 1. các cách nạp hỏng (mỗi dòng: một lần gọi `estimate`, ETA gốc của route-engine = {eta:.1} s) --"
    );
    let ok = EtaService::load(&dir, version, tract_make);
    show("bình thường (tract, bản -cls)", &ok, eta, row);
    show(
        "không có thư mục mô hình",
        &EtaService::load(Path::new("/không/có"), version, tract_make),
        eta,
        row,
    );
    show(
        "tên phiên bản không tồn tại",
        &EtaService::load(&dir, "eta-resid-v999", tract_make),
        eta,
        row,
    );

    let tmp = std::env::temp_dir().join(format!("loc53-fallback-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).expect("tmp");
    let files = ModelFiles::new(&dir, version);
    let copy = |name: &str| {
        let d = tmp.join(name);
        std::fs::create_dir_all(&d).expect("tmp");
        std::fs::copy(&files.onnx, d.join(format!("{version}.onnx"))).expect("copy");
        std::fs::copy(&files.json, d.join(format!("{version}.json"))).expect("copy");
        d
    };
    let d = copy("corrupt");
    let mut b = std::fs::read(d.join(format!("{version}.onnx"))).expect("đọc");
    let m = b.len() / 2;
    b[m] ^= 0xFF;
    std::fs::write(d.join(format!("{version}.onnx")), b).expect("ghi");
    show(
        "file .onnx bị lật một byte (SHA-256 lệch)",
        &EtaService::load(&d, version, tract_make),
        eta,
        row,
    );
    let d = copy("order");
    let mut j: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(d.join(format!("{version}.json"))).expect("đọc"),
    )
    .expect("json");
    j["features"].as_array_mut().expect("mảng").swap(0, 1);
    std::fs::write(d.join(format!("{version}.json")), j.to_string()).expect("ghi");
    show(
        "siêu dữ liệu đổi thứ tự đặc trưng",
        &EtaService::load(&d, version, tract_make),
        eta,
        row,
    );
    let d = copy("schema");
    let mut j: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(d.join(format!("{version}.json"))).expect("đọc"),
    )
    .expect("json");
    j["schema_version"] = 2.into();
    std::fs::write(d.join(format!("{version}.json")), j.to_string()).expect("ghi");
    show(
        "siêu dữ liệu lược đồ phiên bản 2",
        &EtaService::load(&d, version, tract_make),
        eta,
        row,
    );
    show(
        "bản hồi quy gốc (-v1) nạp bằng tract",
        &EtaService::load(&dir, "eta-resid-v1", tract_make),
        eta,
        row,
    );
    show(
        "bản gốc (-v1) nạp bằng ort",
        &EtaService::load(&dir, "eta-resid-v1", ort_make),
        eta,
        row,
    );
    show(
        "bản -cls nạp bằng ort",
        &EtaService::load(&dir, version, ort_make),
        eta,
        row,
    );
    show(
        "yêu cầu thiếu đặc trưng (5 thay vì 19)",
        &ok,
        eta,
        &row[..5],
    );

    println!(
        "\n-- 2. mô hình thỉnh thoảng hỏng lúc chạy: cứ 20 lần gọi có 1 lần lỗi / NaN / số vô lý (lần lượt) --"
    );
    let flaky = EtaService::with_predictor(
        Box::new(Flaky {
            inner: tract_make(&files.onnx).expect("nạp"),
            every: 20,
            calls: std::sync::atomic::AtomicU64::new(0),
        }),
        files.verify().expect("hợp lệ"),
    );
    let (mut non_finite, mut by_reason) =
        (0usize, std::collections::BTreeMap::<String, usize>::new());
    let n = 10_000usize;
    let mut sum_abs_dev = 0.0;
    for i in 0..n {
        let r = &rows[i % rows.len()];
        let e = flaky.estimate(f64::from(r[0]), r);
        if !e.eta_s.is_finite() {
            non_finite += 1;
        }
        sum_abs_dev += (e.eta_s - f64::from(r[0])).abs();
        if let Source::Fallback(why) = &e.source {
            let k = match why {
                FallbackReason::RuntimeError(_) => "lỗi lúc chạy",
                FallbackReason::NotFinite => "đầu ra không hữu hạn",
                FallbackReason::Implausible => "đầu ra vô lý",
                FallbackReason::BadInput => "sai số đặc trưng",
                FallbackReason::NotLoaded(_) => "chưa nạp",
            };
            *by_reason.entry(k.into()).or_default() += 1;
        }
    }
    let (served, fallbacks) = flaky.counters();
    println!(
        "  {n} yêu cầu: trả lời {served}, fallback {fallbacks} ({:.2}%); số ETA không hữu hạn trả ra: {non_finite}",
        100.0 * fallbacks as f64 / served as f64
    );
    println!(
        "  theo lý do: {by_reason:?}; |ETA trả − ETA gốc| trung bình {:.2} s (các yêu cầu fallback đúng bằng 0)",
        sum_abs_dev / n as f64
    );
    let _ = std::fs::remove_dir_all(&tmp);
    println!("uptime lúc kết thúc: {}", uptime());
}
