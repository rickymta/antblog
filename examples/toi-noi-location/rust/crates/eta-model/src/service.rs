//! `EtaService`: luôn trả ETA, có hoặc không có mô hình.

use crate::{EtaError, ModelFiles, ModelMeta, Predictor};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// Vì sao không dùng mô hình cho một yêu cầu (hoặc cho cả dịch vụ).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FallbackReason {
    /// Chưa từng nạp được mô hình (thiếu, hỏng, sai phiên bản, sai đặc trưng, backend lỗi).
    NotLoaded(String),
    /// Số đặc trưng của yêu cầu không khớp mô hình.
    BadInput,
    /// Backend báo lỗi lúc chạy.
    RuntimeError(String),
    /// Đầu ra không hữu hạn.
    NotFinite,
    /// Đầu ra hữu hạn nhưng vô lý (ETA ngoài khoảng cho phép quanh ETA của route-engine).
    Implausible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Model {
        version: String,
        backend: &'static str,
    },
    Fallback(FallbackReason),
}

#[derive(Debug, Clone, PartialEq)]
pub struct EtaEstimate {
    pub eta_s: f64,
    pub source: Source,
}

/// Khoảng cho phép của ETA do mô hình đưa ra, tính theo ETA gốc: `[lo × eta, hi × eta + cộng]`. Đặt trước, không chỉnh theo dữ liệu.
const PLAUSIBLE_LO: f64 = 0.25;
const PLAUSIBLE_HI: f64 = 4.0;
const PLAUSIBLE_ADD_S: f64 = 300.0;

pub struct EtaService {
    model: Result<(Box<dyn Predictor>, ModelMeta), String>,
    served: AtomicU64,
    fallbacks: AtomicU64,
}

impl EtaService {
    /// Dịch vụ với một bộ dự đoán đã có (dùng trong test và lab).
    pub fn with_predictor(p: Box<dyn Predictor>, meta: ModelMeta) -> Self {
        Self {
            model: Ok((p, meta)),
            served: AtomicU64::new(0),
            fallbacks: AtomicU64::new(0),
        }
    }

    /// Dịch vụ không có mô hình: mọi yêu cầu fallback. `reason` ghi vào [`FallbackReason::NotLoaded`].
    pub fn without_model(reason: impl Into<String>) -> Self {
        Self {
            model: Err(reason.into()),
            served: AtomicU64::new(0),
            fallbacks: AtomicU64::new(0),
        }
    }

    /// Nạp `<dir>/<version>.{onnx,json}` bằng backend do `make` dựng. Mọi lỗi nạp biến thành dịch vụ fallback (không panic).
    pub fn load(
        dir: &Path,
        version: &str,
        make: impl FnOnce(&Path) -> Result<Box<dyn Predictor>, EtaError>,
    ) -> Self {
        let files = ModelFiles::new(dir, version);
        let result = files.verify().and_then(|meta| {
            let p = make(&files.onnx)?;
            if p.n_features() != meta.features.len() {
                return Err(EtaError::FeatureMismatch(format!(
                    "backend {} báo {} đầu vào, siêu dữ liệu ghi {}",
                    p.backend(),
                    p.n_features(),
                    meta.features.len()
                )));
            }
            Ok((p, meta))
        });
        match result {
            Ok(m) => Self {
                model: Ok(m),
                served: AtomicU64::new(0),
                fallbacks: AtomicU64::new(0),
            },
            Err(e) => Self::without_model(e.to_string()),
        }
    }

    pub fn meta(&self) -> Option<&ModelMeta> {
        self.model.as_ref().ok().map(|(_, m)| m)
    }

    pub fn load_error(&self) -> Option<&str> {
        self.model.as_ref().err().map(String::as_str)
    }

    /// `(đã trả lời, số lần fallback)`.
    pub fn counters(&self) -> (u64, u64) {
        (
            self.served.load(Ordering::Relaxed),
            self.fallbacks.load(Ordering::Relaxed),
        )
    }

    fn fallback(&self, route_eta_s: f64, why: FallbackReason) -> EtaEstimate {
        self.fallbacks.fetch_add(1, Ordering::Relaxed);
        EtaEstimate {
            eta_s: route_eta_s,
            source: Source::Fallback(why),
        }
    }

    /// ETA cho một chặng. `features` theo đúng thứ tự [`crate::FEATURES_V1`]; phần tử đầu là `route_eta_s`.
    pub fn estimate(&self, route_eta_s: f64, features: &[f32]) -> EtaEstimate {
        self.served.fetch_add(1, Ordering::Relaxed);
        let (p, meta) = match &self.model {
            Ok(m) => m,
            Err(e) => return self.fallback(route_eta_s, FallbackReason::NotLoaded(e.clone())),
        };
        if features.len() != p.n_features() {
            return self.fallback(route_eta_s, FallbackReason::BadInput);
        }
        let resid = match p.predict_one(features) {
            Ok(r) => f64::from(r),
            Err(e) => {
                return self.fallback(route_eta_s, FallbackReason::RuntimeError(e.to_string()));
            }
        };
        if !resid.is_finite() {
            return self.fallback(route_eta_s, FallbackReason::NotFinite);
        }
        let eta = route_eta_s + resid;
        if eta < PLAUSIBLE_LO * route_eta_s || eta > PLAUSIBLE_HI * route_eta_s + PLAUSIBLE_ADD_S {
            return self.fallback(route_eta_s, FallbackReason::Implausible);
        }
        EtaEstimate {
            eta_s: eta,
            source: Source::Model {
                version: meta.model_version.clone(),
                backend: p.backend(),
            },
        }
    }
}
