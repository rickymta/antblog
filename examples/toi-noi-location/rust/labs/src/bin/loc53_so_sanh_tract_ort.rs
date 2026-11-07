//! LOC-53: chạy mô hình ETA (ONNX, xuất từ Python bằng `onnxmltools`) bằng `tract` và `ort` trong Rust. So số học với LightGBM và
//! onnxruntime Python, đo độ trễ một dòng và theo lô, p99 khi nhiều luồng, và bộ nhớ (RSS).
//!
//! Chạy từ gốc repo (cần `loc53_onnx_export.py` chạy trước; ort cần ORT_DYLIB_PATH trỏ tới libonnxruntime):
//!   ORT_DYLIB_PATH=<đường dẫn libonnxruntime.dylib> cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml \
//!     -p labs --bin loc53_so_sanh_tract_ort -- --backend ort --version eta-resid-v1
//! Tham số: --backend tract|ort  --version tên (eta-resid-v1 | eta-resid-v1-cls)  --calls N (20000)  --model-dir DIR
//!
//! Đo khi máy đang chịu tải nặng: chỉ so sánh tương đối trong cùng một lần chạy, không kết luận về thời gian tuyệt đối.

use eta_model::{FEATURES_V1, ModelFiles, Predictor};
use labs::p03::uptime;
use labs::{Args, die, lab_root, percentile};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Instant;

fn rss_mb() -> f64 {
    let pid = std::process::id().to_string();
    Command::new("ps")
        .args(["-o", "rss=", "-p", &pid])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<f64>().ok())
        .map_or(f64::NAN, |kib| kib / 1024.0)
}

fn read_matrix(p: &Path) -> (Vec<f32>, usize) {
    let text = std::fs::read_to_string(p).unwrap_or_else(|e| die(&format!("{}: {e}", p.display())));
    let mut flat = Vec::new();
    let mut n = 0;
    for l in text.lines() {
        flat.extend(l.split(',').map(|x| x.parse::<f32>().unwrap_or(f32::NAN)));
        n += 1;
    }
    (flat, n)
}

fn read_col(p: &Path) -> Vec<f64> {
    std::fs::read_to_string(p)
        .unwrap_or_else(|e| die(&format!("{}: {e}", p.display())))
        .lines()
        .map(|l| l.trim().parse::<f64>().unwrap_or(f64::NAN))
        .collect()
}

fn summarize(label: &str, mut us: Vec<f64>) {
    us.sort_by(f64::total_cmp);
    let mean = us.iter().sum::<f64>() / us.len() as f64;
    println!(
        "  {label:<44} n {:>7}  trung bình {:>8.1} µs  p50 {:>8.1}  p95 {:>8.1}  p99 {:>8.1}  p99,9 {:>9.1}  max {:>9.1}",
        us.len(),
        mean,
        percentile(&us, 50.0),
        percentile(&us, 95.0),
        percentile(&us, 99.0),
        percentile(&us, 99.9),
        us.last().copied().unwrap_or(f64::NAN)
    );
}

fn make(
    backend: &str,
    path: &Path,
    batch: usize,
) -> Result<Arc<dyn Predictor>, eta_model::EtaError> {
    let n = FEATURES_V1.len();
    match backend {
        "tract" => Ok(Arc::new(eta_model::backend_tract::TractModel::load(
            path, n, batch,
        )?)),
        "ort" => Ok(Arc::new(eta_model::backend_ort::OrtModel::load(
            path, n, 1,
        )?)),
        x => die(&format!("--backend không hợp lệ: {x}")),
    }
}

fn main() {
    let args = Args::from_env();
    let backend = args
        .value("--backend")
        .unwrap_or_else(|| die("cần --backend tract|ort"))
        .to_string();
    let version = args
        .value("--version")
        .unwrap_or("eta-resid-v1")
        .to_string();
    let dir: PathBuf = args.value("--model-dir").map_or_else(
        || lab_root().join("python/p06/artifacts/model"),
        PathBuf::from,
    );
    let calls = args.u64_or("--calls", 20_000) as usize;
    println!("== LOC-53: {backend} chạy mô hình ONNX {version} (trên dữ liệu mô phỏng) ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    let n = FEATURES_V1.len();
    let rss0 = rss_mb();

    // ---- 1. nạp và kiểm phiên bản --------------------------------------------------------------------------------------
    let files = ModelFiles::new(&dir, &version);
    let meta = files
        .verify()
        .unwrap_or_else(|e| die(&format!("kiểm tra phiên bản: {e}")));
    let size = std::fs::metadata(&files.onnx).map_or(0, |m| m.len());
    println!(
        "\n-- 1. nạp: {} ({:.1} KiB), phiên bản {}, đầu ra {}, {} đặc trưng, SHA-256 {}… (đã khớp) --",
        files
            .onnx
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?"),
        size as f64 / 1024.0,
        meta.model_version,
        meta.output,
        meta.features.len(),
        &meta.sha256[..16]
    );
    println!("RSS trước khi nạp: {rss0:.1} MB");
    let t = Instant::now();
    let model = match make(&backend, &files.onnx, 64) {
        Ok(m) => m,
        Err(e) => {
            println!("KHÔNG NẠP ĐƯỢC bằng {backend}: {e}");
            println!(
                "(đây là kết quả đo, không phải lỗi của lab: xem mục «tract và TreeEnsembleRegressor» trong results/p06.md)"
            );
            println!("uptime lúc kết thúc: {}", uptime());
            return;
        }
    };
    println!(
        "nạp xong sau {:.1} ms; RSS sau khi nạp: {:.1} MB",
        t.elapsed().as_secs_f64() * 1e3,
        rss_mb()
    );

    // ---- 2. so số học -----------------------------------------------------------------------------------------------------
    let (rows, nrows) = read_matrix(&dir.join("eta_test_inputs.csv"));
    let lgbm = read_col(&dir.join("eta_test_pred_lgbm.csv"));
    let py_name = if version.ends_with("-cls") {
        "eta_test_pred_ort_py_cls.csv"
    } else {
        "eta_test_pred_ort_py.csv"
    };
    let pyort = read_col(&dir.join(py_name));
    assert_eq!(nrows, lgbm.len());
    let nan_rows: Vec<bool> = rows
        .chunks(n)
        .map(|r| r.iter().any(|x| x.is_nan()))
        .collect();
    println!(
        "\n-- 2. so số học trên {nrows} dòng (mẫu có seed từ tập kiểm tra; {} dòng có NaN) --",
        nan_rows.iter().filter(|&&b| b).count()
    );
    let single: Vec<f64> = rows
        .chunks(n)
        .map(|r| f64::from(model.predict_one(r).expect("chạy")))
        .collect();
    let batch: Vec<f64> = model
        .predict_batch(&rows)
        .expect("chạy lô")
        .into_iter()
        .map(f64::from)
        .collect();
    let cmp = |name: &str, a: &[f64], b: &[f64]| {
        let d: Vec<f64> = a.iter().zip(b).map(|(x, y)| (x - y).abs()).collect();
        let mut s = d.clone();
        s.sort_by(f64::total_cmp);
        let dn = d
            .iter()
            .zip(&nan_rows)
            .filter(|&(_, &m)| m)
            .map(|(v, _)| *v)
            .fold(0.0, f64::max);
        println!(
            "  {name:<42} max |Δ| {:.3e}  trung bình {:.3e}  p99 {:.3e}  max trên dòng NaN {:.3e}  số dòng |Δ|>1e-3: {}  >1e-2: {}  >1: {}",
            s.last().copied().unwrap_or(0.0),
            d.iter().sum::<f64>() / d.len() as f64,
            percentile(&s, 99.0),
            dn,
            d.iter().filter(|&&v| v > 1e-3).count(),
            d.iter().filter(|&&v| v > 1e-2).count(),
            d.iter().filter(|&&v| v > 1.0).count()
        );
    };
    cmp(
        &format!("{backend} (một dòng) so với LightGBM"),
        &single,
        &lgbm,
    );
    cmp(
        &format!("{backend} (một dòng) so với onnxruntime Python"),
        &single,
        &pyort,
    );
    cmp(
        &format!("{backend} (lô 64) so với {backend} (một dòng)"),
        &batch,
        &single,
    );

    // ---- 3. độ trễ một dòng ------------------------------------------------------------------------------------------------
    println!(
        "\n-- 3. độ trễ ({calls} lần gọi, luồng đơn, dòng lấy vòng tròn từ tập mẫu; đã chạy nóng 500 lần) --"
    );
    for i in 0..500 {
        let _ = model.predict_one(&rows[(i % nrows) * n..(i % nrows + 1) * n]);
    }
    let mut us = Vec::with_capacity(calls);
    for i in 0..calls {
        let r = &rows[(i % nrows) * n..(i % nrows + 1) * n];
        let t = Instant::now();
        let _ = std::hint::black_box(model.predict_one(r));
        us.push(t.elapsed().as_secs_f64() * 1e6);
    }
    summarize(&format!("{backend}: một dòng"), us);
    let mut us = Vec::new();
    for i in 0..(calls / 64).max(50) {
        let start = (i * 64) % (nrows - 64);
        let r = &rows[start * n..(start + 64) * n];
        let t = Instant::now();
        let _ = std::hint::black_box(model.predict_batch(r));
        us.push(t.elapsed().as_secs_f64() * 1e6);
    }
    let per_row: Vec<f64> = us.iter().map(|v| v / 64.0).collect();
    summarize(&format!("{backend}: lô 64 dòng (cả lô)"), us);
    summarize(&format!("{backend}: lô 64 dòng (chia mỗi dòng)"), per_row);

    // ---- 4. nhiều luồng --------------------------------------------------------------------------------------------------------
    println!(
        "\n-- 4. p99 khi nhiều luồng cùng gọi (mỗi luồng 5.000 lần; máy dùng chung và đang tải nặng) --"
    );
    let per_thread = 5_000usize;
    for threads in [1usize, 4, 8] {
        for shared in [true, false] {
            if !shared && threads == 1 {
                continue;
            }
            let models: Vec<Arc<dyn Predictor>> = (0..threads)
                .map(|_| {
                    if shared {
                        Arc::clone(&model)
                    } else {
                        make(&backend, &files.onnx, 1).expect("nạp lại")
                    }
                })
                .collect();
            let all: Vec<Vec<f64>> = std::thread::scope(|s| {
                let hs: Vec<_> = models
                    .iter()
                    .enumerate()
                    .map(|(ti, m)| {
                        let rows = &rows;
                        s.spawn(move || {
                            let mut us = Vec::with_capacity(per_thread);
                            for i in 0..per_thread {
                                let k = (i * 7 + ti * 131) % nrows;
                                let t = Instant::now();
                                let _ =
                                    std::hint::black_box(m.predict_one(&rows[k * n..(k + 1) * n]));
                                us.push(t.elapsed().as_secs_f64() * 1e6);
                            }
                            us
                        })
                    })
                    .collect();
                hs.into_iter().map(|h| h.join().expect("luồng")).collect()
            });
            let flat: Vec<f64> = all.into_iter().flatten().collect();
            summarize(
                &format!(
                    "{backend}: {threads} luồng, {}",
                    if shared {
                        "dùng chung một mô hình"
                    } else {
                        "mỗi luồng một mô hình"
                    }
                ),
                flat,
            );
        }
    }

    // ---- 5. bộ nhớ --------------------------------------------------------------------------------------------------------------------
    println!(
        "\n-- 5. bộ nhớ (RSS của cả tiến trình, MB; lab chạy riêng cho từng backend để khỏi cộng dồn) --"
    );
    println!(
        "  trước khi nạp {rss0:.1}; ở cuối lab (sau mọi lần gọi và mọi bản sao mô hình) {:.1}",
        rss_mb()
    );
    println!("uptime lúc kết thúc: {}", uptime());
}
