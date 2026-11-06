//! Backend **tract** (thuần Rust, không cần thư viện ngoài). Hai kế hoạch chạy: một dòng và lô cố định.

use crate::{EtaError, Predictor};
use std::path::Path;
use tract_onnx::prelude::*;

type Plan = Arc<TypedRunnableModel>;

pub struct TractModel {
    n_features: usize,
    single: Plan,
    batch: Option<(usize, Plan)>,
}

fn be(msg: impl std::fmt::Display) -> EtaError {
    EtaError::Backend {
        backend: "tract",
        msg: msg.to_string(),
    }
}

fn plan(path: &Path, rows: usize, n: usize) -> Result<Plan, EtaError> {
    tract_onnx::onnx()
        .model_for_path(path)
        .map_err(|e| EtaError::Corrupt(format!("tract không đọc được {}: {e}", path.display())))?
        .with_input_fact(0, f32::fact([rows, n]).into())
        .map_err(be)?
        .into_optimized()
        .map_err(be)?
        .into_runnable()
        .map_err(be)
}

impl TractModel {
    /// `batch` > 1 dựng thêm kế hoạch chạy lô đúng `batch` dòng.
    pub fn load(path: &Path, n_features: usize, batch: usize) -> Result<Self, EtaError> {
        let single = plan(path, 1, n_features)?;
        let batch = (batch > 1)
            .then(|| plan(path, batch, n_features).map(|p| (batch, p)))
            .transpose()?;
        Ok(Self {
            n_features,
            single,
            batch,
        })
    }

    fn run(plan: &Plan, rows: usize, n: usize, data: &[f32]) -> Result<Vec<f32>, EtaError> {
        let input = tract_ndarray::Array2::from_shape_vec((rows, n), data.to_vec()).map_err(be)?;
        let out = plan.run(tvec!(Tensor::from(input).into())).map_err(be)?;
        let view = out[0].to_plain_array_view::<f32>().map_err(be)?;
        Ok(view.iter().copied().collect())
    }
}

impl Predictor for TractModel {
    fn backend(&self) -> &'static str {
        "tract"
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
        let v = Self::run(&self.single, 1, self.n_features, features)?;
        v.first().copied().ok_or_else(|| be("đầu ra rỗng"))
    }
    fn predict_batch(&self, rows: &[f32]) -> Result<Vec<f32>, EtaError> {
        let n = self.n_features;
        let mut out = Vec::with_capacity(rows.len() / n);
        let mut rest = rows;
        if let Some((b, plan)) = &self.batch {
            while rest.len() >= b * n {
                let (head, tail) = rest.split_at(b * n);
                out.extend(Self::run(plan, *b, n, head)?);
                rest = tail;
            }
        }
        for r in rest.chunks(n) {
            out.push(self.predict_one(r)?);
        }
        Ok(out)
    }
}
