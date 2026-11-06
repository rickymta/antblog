//! `eta-model` (LOC-53): đưa mô hình ETA từ Python vào Rust.
//!
//! Mô hình là LightGBM dự đoán **phần dư** `thời gian thật − ETA của route-engine`, đã xuất sang ONNX (`onnxmltools`). Crate này:
//! - nạp file `.onnx` bằng **tract** (thuần Rust) hoặc **ort** (ONNX Runtime, nạp `libonnxruntime` lúc chạy), cùng một trait [`Predictor`];
//! - kiểm tra **phiên bản** (`<tên>.json` đi kèm: lược đồ, danh sách và thứ tự đặc trưng, SHA-256 của file `.onnx`);
//! - [`EtaService`] luôn trả lời được: mô hình thiếu, hỏng, sai đặc trưng, lỗi chạy hay trả số vô lý ⇒ **fallback** về ETA gốc của
//!   route-engine (kèm lý do), không bao giờ làm hỏng đường nóng.
//!
//! Mô hình học trên dữ liệu mô phỏng nên chỉ chứng minh phương pháp, không chứng minh độ chính xác ngoài đời.

#[cfg(feature = "ort")]
pub mod backend_ort;
#[cfg(feature = "tract")]
pub mod backend_tract;
pub mod meta;
pub mod service;

pub use meta::{FEATURES_V1, ModelFiles, ModelMeta, SCHEMA_VERSION};
pub use service::{EtaEstimate, EtaService, FallbackReason, Source};

/// Lỗi của lớp mô hình. Mỗi biến thể ứng với một lý do fallback khác nhau.
#[derive(Debug, thiserror::Error)]
pub enum EtaError {
    #[error("thiếu file mô hình: {0}")]
    Missing(String),
    #[error("file mô hình hỏng hoặc không đọc được: {0}")]
    Corrupt(String),
    #[error("phiên bản mô hình không khớp: {0}")]
    VersionMismatch(String),
    #[error("đặc trưng không khớp hợp đồng: {0}")]
    FeatureMismatch(String),
    #[error("lỗi backend {backend}: {msg}")]
    Backend { backend: &'static str, msg: String },
}

/// Bộ dự đoán: nhận các dòng đặc trưng (row-major, `n_features` số `f32` mỗi dòng, `NaN` = thiếu) và trả phần dư (giây) cho từng dòng.
pub trait Predictor: Send + Sync {
    fn backend(&self) -> &'static str;
    fn n_features(&self) -> usize;
    /// Một dòng.
    fn predict_one(&self, features: &[f32]) -> Result<f32, EtaError>;
    /// Nhiều dòng một lần (`rows.len()` chia hết cho `n_features`).
    fn predict_batch(&self, rows: &[f32]) -> Result<Vec<f32>, EtaError> {
        let n = self.n_features();
        rows.chunks(n).map(|r| self.predict_one(r)).collect()
    }
}
