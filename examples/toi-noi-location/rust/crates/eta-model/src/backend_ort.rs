//! Backend **ort** (ONNX Runtime). Dùng feature `load-dynamic`: build không tải nhị phân; lúc chạy cần `ORT_DYLIB_PATH` trỏ tới
//! `libonnxruntime` (ví dụ file nằm trong gói Python `onnxruntime`). Không có thì [`OrtModel::load`] trả lỗi, không panic.

use crate::{EtaError, Predictor};
use ort::environment::Environment;
use ort::session::Session;
use ort::value::Tensor;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

pub struct OrtModel {
    n_features: usize,
    input: String,
    session: Mutex<Session>,
}

fn be(msg: impl std::fmt::Display) -> EtaError {
    EtaError::Backend {
        backend: "ort",
        msg: msg.to_string(),
    }
}

/// Môi trường ONNX Runtime dùng chung cả tiến trình (nạp thư viện động đúng một lần).
fn environment() -> Result<&'static Environment, EtaError> {
    static ENV: OnceLock<Result<Environment, String>> = OnceLock::new();
    let r = ENV.get_or_init(|| {
        let dylib = match std::env::var("ORT_DYLIB_PATH") {
            Ok(p) if Path::new(&p).exists() => p,
            Ok(p) => return Err(format!("ORT_DYLIB_PATH={p} không tồn tại")),
            Err(_) => return Err("chưa đặt ORT_DYLIB_PATH (đường dẫn tới libonnxruntime)".into()),
        };
        // `ort` có thể panic nếu thư viện động không dùng được; chặn lại để thành lỗi bình thường
        std::panic::catch_unwind(|| {
            ort::init_from(&dylib)
                .map_err(|e| format!("không nạp được {dylib}: {e}"))?
                .with_name("eta-model")
                .build()
                .map_err(|e| format!("không tạo được môi trường ort: {e}"))
        })
        .unwrap_or_else(|_| Err("ort panic khi khởi tạo (kiểm tra libonnxruntime)".into()))
    });
    r.as_ref().map_err(be)
}

impl OrtModel {
    pub fn load(path: &Path, n_features: usize, intra_threads: usize) -> Result<Self, EtaError> {
        let env = environment()?;
        let mut builder = Session::builder(env)
            .map_err(be)?
            .with_intra_threads(intra_threads)
            .map_err(be)?;
        let session = builder.commit_from_file(path).map_err(|e| {
            EtaError::Corrupt(format!("ort không nạp được {}: {e}", path.display()))
        })?;
        let input = session
            .inputs()
            .first()
            .map(|i| i.name().to_string())
            .ok_or_else(|| be("mô hình không có đầu vào"))?;
        Ok(Self {
            n_features,
            input,
            session: Mutex::new(session),
        })
    }

    fn run(&self, rows: usize, data: &[f32]) -> Result<Vec<f32>, EtaError> {
        let tensor = Tensor::from_array(([rows, self.n_features], data.to_vec())).map_err(be)?;
        let mut s = self.session.lock().map_err(|_| be("khoá bị nhiễm độc"))?;
        let outputs = s
            .run(ort::inputs![self.input.as_str() => tensor])
            .map_err(be)?;
        let (_, v) = outputs[0].try_extract_tensor::<f32>().map_err(be)?;
        Ok(v.to_vec())
    }
}

impl Predictor for OrtModel {
    fn backend(&self) -> &'static str {
        "ort"
    }
    fn n_features(&self) -> usize {
        self.n_features
    }
    fn predict_one(&self, features: &[f32]) -> Result<f32, EtaError> {
        if features.len() != self.n_features {
            return Err(EtaError::FeatureMismatch(format!(
                "cần {} đặc trưng, nhận {}",
                self.n_features,
                features.len()
            )));
        }
        self.run(1, features)?
            .first()
            .copied()
            .ok_or_else(|| be("đầu ra rỗng"))
    }
    fn predict_batch(&self, rows: &[f32]) -> Result<Vec<f32>, EtaError> {
        if !rows.len().is_multiple_of(self.n_features) {
            return Err(EtaError::FeatureMismatch(
                "độ dài lô không chia hết cho số đặc trưng".into(),
            ));
        }
        self.run(rows.len() / self.n_features, rows)
    }
}
