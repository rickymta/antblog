//! Kiểm thử `eta-model` trên mô hình thử nghiệm nhỏ (`tests/data/eta-resid-vtest.*`, sinh bằng
//! `loc53_onnx_export.py --tiny`). Test ort chỉ chạy khi có `ORT_DYLIB_PATH`; không có thì in «BỎ QUA» (không tính là đạt ort).

use eta_model::{
    EtaError, EtaService, FEATURES_V1, FallbackReason, ModelFiles, ModelMeta, Predictor, Source,
};
use std::path::{Path, PathBuf};

const VERSION: &str = "eta-resid-vtest";
/// Cùng cây nhưng viết lại thành TreeEnsembleClassifier: tract không cài TreeEnsembleRegressor.
const VERSION_CLS: &str = "eta-resid-vtest-cls";

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data")
}

fn read_csv(name: &str) -> Vec<Vec<f32>> {
    std::fs::read_to_string(data_dir().join(name))
        .expect("đọc csv")
        .lines()
        .map(|l| {
            l.split(',')
                .map(|x| x.parse::<f32>().expect("số"))
                .collect()
        })
        .collect()
}

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("eta-model-test-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    for f in [format!("{VERSION}.onnx"), format!("{VERSION}.json")] {
        std::fs::copy(data_dir().join(&f), d.join(&f)).unwrap();
    }
    d
}

#[cfg(feature = "tract")]
fn tract_make(p: &Path) -> Result<Box<dyn Predictor>, EtaError> {
    Ok(Box::new(eta_model::backend_tract::TractModel::load(
        p,
        FEATURES_V1.len(),
        8,
    )?))
}
#[cfg(not(feature = "tract"))]
fn tract_make(_: &Path) -> Result<Box<dyn Predictor>, EtaError> {
    Err(EtaError::Missing("feature tract tắt".into()))
}

/// Bộ dự đoán giả để thử đường fallback.
struct Fake(Result<f32, ()>);
impl Predictor for Fake {
    fn backend(&self) -> &'static str {
        "fake"
    }
    fn n_features(&self) -> usize {
        FEATURES_V1.len()
    }
    fn predict_one(&self, _: &[f32]) -> Result<f32, EtaError> {
        self.0.map_err(|()| EtaError::Backend {
            backend: "fake",
            msg: "hỏng".into(),
        })
    }
}

fn meta() -> ModelMeta {
    ModelFiles::new(&data_dir(), VERSION)
        .verify()
        .expect("mô hình thử nghiệm hợp lệ")
}

#[test]
fn feature_contract_matches_the_exported_metadata() {
    let m = meta();
    let got: Vec<&str> = m.features.iter().map(String::as_str).collect();
    assert_eq!(
        got,
        FEATURES_V1.to_vec(),
        "thứ tự đặc trưng Rust phải trùng Python"
    );
    assert_eq!(m.output, "residual_s");
}

#[cfg(feature = "tract")]
#[test]
fn tract_matches_lightgbm_including_rows_with_nan() {
    let svc = EtaService::load(&data_dir(), VERSION_CLS, tract_make);
    assert!(svc.load_error().is_none(), "{:?}", svc.load_error());
    let inputs = read_csv("vtest_inputs.csv");
    let expected = read_csv("vtest_expected.csv");
    let model = eta_model::backend_tract::TractModel::load(
        &data_dir().join(format!("{VERSION_CLS}.onnx")),
        FEATURES_V1.len(),
        8,
    )
    .unwrap();
    let flat: Vec<f32> = inputs.iter().flatten().copied().collect();
    let batch = model.predict_batch(&flat).unwrap();
    let mut with_nan = 0;
    for (i, (row, want)) in inputs.iter().zip(&expected).enumerate() {
        let one = model.predict_one(row).unwrap();
        assert!(
            (one - want[0]).abs() < 1e-3,
            "dòng {i}: tract {one} ≠ LightGBM {}",
            want[0]
        );
        assert!(
            (batch[i] - one).abs() < 1e-4,
            "dòng {i}: lô {} ≠ một dòng {one}",
            batch[i]
        );
        with_nan += usize::from(row.iter().any(|x| x.is_nan()));
    }
    assert!(with_nan > 10, "phải có dòng NaN để thử xử lý giá trị thiếu");
}

/// Ghi lại giới hạn đã đo: tract 0.23.10 không cài `TreeEnsembleRegressor`. Nếu bản tract sau này cài được, test này sẽ gãy
/// và là dấu hiệu để bỏ bản viết lại «-cls».
#[cfg(feature = "tract")]
#[test]
fn tract_cannot_load_the_tree_ensemble_regressor_file() {
    let r = eta_model::backend_tract::TractModel::load(
        &data_dir().join(format!("{VERSION}.onnx")),
        FEATURES_V1.len(),
        1,
    );
    let msg = match r {
        Err(e) => e.to_string(),
        Ok(_) => panic!("tract đã chạy được TreeEnsembleRegressor: có thể bỏ bản viết lại -cls"),
    };
    assert!(msg.contains("TreeEnsembleRegressor"), "{msg}");
}

#[cfg(feature = "ort")]
#[test]
fn ort_matches_lightgbm_when_the_runtime_library_is_available() {
    if std::env::var("ORT_DYLIB_PATH").is_err() {
        eprintln!(
            "BỎ QUA: chưa đặt ORT_DYLIB_PATH, test ort không chạy (ort chưa được kiểm trong lần chạy này)"
        );
        return;
    }
    let (inputs, expected) = (read_csv("vtest_inputs.csv"), read_csv("vtest_expected.csv"));
    for v in [VERSION, VERSION_CLS] {
        let model = eta_model::backend_ort::OrtModel::load(
            &data_dir().join(format!("{v}.onnx")),
            FEATURES_V1.len(),
            1,
        )
        .expect("nạp ort");
        for (i, (row, want)) in inputs.iter().zip(&expected).enumerate() {
            let got = model.predict_one(row).unwrap();
            assert!(
                (got - want[0]).abs() < 1e-3,
                "{v} dòng {i}: ort {got} ≠ LightGBM {}",
                want[0]
            );
        }
    }
}

#[cfg(feature = "ort")]
#[test]
fn ort_without_library_is_an_error_not_a_panic() {
    if std::env::var("ORT_DYLIB_PATH").is_ok() {
        return; // đã có thư viện: không thử được đường lỗi này
    }
    let r = eta_model::backend_ort::OrtModel::load(
        &data_dir().join(format!("{VERSION}.onnx")),
        FEATURES_V1.len(),
        1,
    );
    assert!(matches!(r, Err(EtaError::Backend { backend: "ort", .. })));
}

#[test]
fn missing_model_falls_back_to_the_route_engine_eta() {
    let svc = EtaService::load(Path::new("/không/có/thư/mục"), VERSION, tract_make);
    assert!(svc.load_error().unwrap().contains("thiếu"));
    let e = svc.estimate(321.0, &[0.0; 19]);
    assert_eq!(e.eta_s, 321.0);
    assert!(matches!(
        e.source,
        Source::Fallback(FallbackReason::NotLoaded(_))
    ));
    assert_eq!(svc.counters(), (1, 1));
}

#[test]
fn corrupted_onnx_is_caught_by_the_sha256_check() {
    let d = scratch("sha");
    let mut bytes = std::fs::read(d.join(format!("{VERSION}.onnx"))).unwrap();
    let n = bytes.len();
    bytes[n / 2] ^= 0xFF;
    std::fs::write(d.join(format!("{VERSION}.onnx")), bytes).unwrap();
    let svc = EtaService::load(&d, VERSION, tract_make);
    assert!(
        svc.load_error().unwrap().contains("SHA-256"),
        "{:?}",
        svc.load_error()
    );
    assert_eq!(svc.estimate(100.0, &[0.0; 19]).eta_s, 100.0);
}

#[test]
fn garbage_model_with_a_valid_hash_is_rejected_by_the_backend() {
    let d = scratch("garbage");
    let garbage = b"day khong phai file onnx".to_vec();
    std::fs::write(d.join(format!("{VERSION}.onnx")), &garbage).unwrap();
    let mut m: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(d.join(format!("{VERSION}.json"))).unwrap())
            .unwrap();
    use sha2::{Digest, Sha256};
    m["sha256"] = Sha256::digest(&garbage)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()
        .into();
    std::fs::write(d.join(format!("{VERSION}.json")), m.to_string()).unwrap();
    let svc = EtaService::load(&d, VERSION, tract_make);
    assert!(svc.load_error().is_some());
    assert!(matches!(
        svc.estimate(50.0, &[0.0; 19]).source,
        Source::Fallback(FallbackReason::NotLoaded(_))
    ));
}

#[test]
fn wrong_feature_list_or_schema_version_is_refused() {
    for (tag, edit) in [
        (
            "feat",
            Box::new(|m: &mut serde_json::Value| {
                m["features"].as_array_mut().unwrap().pop();
            }) as Box<dyn Fn(&mut serde_json::Value)>,
        ),
        (
            "order",
            Box::new(|m| {
                m["features"].as_array_mut().unwrap().swap(0, 1);
            }),
        ),
        (
            "schema",
            Box::new(|m| {
                m["schema_version"] = 2.into();
            }),
        ),
        (
            "output",
            Box::new(|m| {
                m["output"] = "total_s".into();
            }),
        ),
    ] {
        let d = scratch(tag);
        let mut m: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(d.join(format!("{VERSION}.json"))).unwrap(),
        )
        .unwrap();
        edit(&mut m);
        std::fs::write(d.join(format!("{VERSION}.json")), m.to_string()).unwrap();
        let err = ModelFiles::new(&d, VERSION).verify().unwrap_err();
        match tag {
            "feat" | "order" => {
                assert!(matches!(err, EtaError::FeatureMismatch(_)), "{tag}: {err}")
            }
            _ => assert!(matches!(err, EtaError::VersionMismatch(_)), "{tag}: {err}"),
        }
        let svc = EtaService::load(&d, VERSION, tract_make);
        assert_eq!(svc.estimate(200.0, &[0.0; 19]).eta_s, 200.0, "{tag}");
    }
}

fn svc_with(p: Fake) -> EtaService {
    EtaService::with_predictor(Box::new(p), meta())
}

#[test]
fn runtime_failures_and_nonsense_outputs_fall_back() {
    let f = [0.0f32; 19];
    assert!(matches!(
        svc_with(Fake(Err(()))).estimate(100.0, &f).source,
        Source::Fallback(FallbackReason::RuntimeError(_))
    ));
    assert!(matches!(
        svc_with(Fake(Ok(f32::NAN))).estimate(100.0, &f).source,
        Source::Fallback(FallbackReason::NotFinite)
    ));
    assert!(matches!(
        svc_with(Fake(Ok(f32::INFINITY))).estimate(100.0, &f).source,
        Source::Fallback(FallbackReason::NotFinite)
    ));
    // ETA 100 s, phần dư −90 s ⇒ 10 s < 25% ETA gốc: vô lý
    assert!(matches!(
        svc_with(Fake(Ok(-90.0))).estimate(100.0, &f).source,
        Source::Fallback(FallbackReason::Implausible)
    ));
    // phần dư +5.000 s: vô lý
    assert!(matches!(
        svc_with(Fake(Ok(5000.0))).estimate(100.0, &f).source,
        Source::Fallback(FallbackReason::Implausible)
    ));
    // sai số đặc trưng
    assert!(matches!(
        svc_with(Fake(Ok(1.0))).estimate(100.0, &f[..5]).source,
        Source::Fallback(FallbackReason::BadInput)
    ));
    let ok = svc_with(Fake(Ok(12.5))).estimate(100.0, &f);
    assert_eq!(ok.eta_s, 112.5);
    assert_eq!(
        ok.source,
        Source::Model {
            version: VERSION.into(),
            backend: "fake"
        }
    );
}

#[cfg(feature = "tract")]
#[test]
fn service_with_the_real_test_model_returns_model_estimates() {
    let svc = EtaService::load(&data_dir(), VERSION_CLS, tract_make);
    let inputs = read_csv("vtest_inputs.csv");
    let expected = read_csv("vtest_expected.csv");
    let mut model_hits = 0;
    for (row, want) in inputs.iter().zip(&expected) {
        let e = svc.estimate(f64::from(row[0]), row);
        if let Source::Model { version, backend } = &e.source {
            assert_eq!((version.as_str(), *backend), (VERSION_CLS, "tract"));
            assert!((e.eta_s - (f64::from(row[0]) + f64::from(want[0]))).abs() < 1e-2);
            model_hits += 1;
        }
    }
    assert!(
        model_hits > 30,
        "hầu hết dòng phải đi qua mô hình, còn {model_hits}"
    );
}
