//! Hợp đồng đặc trưng và siêu dữ liệu phiên bản của file mô hình.

use crate::EtaError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Phiên bản lược đồ của file siêu dữ liệu (không phải phiên bản mô hình).
pub const SCHEMA_VERSION: u32 = 1;

/// Danh sách và thứ tự đặc trưng của mô hình v1. **Phải trùng** `p06lib.common.FEATURES` bên Python; test đọc lại danh sách này
/// từ file siêu dữ liệu của mô hình thử nghiệm để bắt lệch.
pub const FEATURES_V1: [&str; 19] = [
    "route_eta_s",
    "route_dist_m",
    "n_edges",
    "leg_kind",
    "tod",
    "dow",
    "is_weekend",
    "o_lat",
    "o_lng",
    "d_lat",
    "d_lng",
    "crow_m",
    "detour",
    "o_dist_centre_m",
    "d_dist_centre_m",
    "req_to_depart_s",
    "hist_speed_o",
    "hist_speed_d",
    "hist_resid_o",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelMeta {
    pub schema_version: u32,
    /// Ví dụ `eta-resid-v1`: tên cố định trong tên file.
    pub model_version: String,
    /// `residual_s`: đầu ra là giây cộng vào ETA của route-engine.
    pub output: String,
    pub features: Vec<String>,
    /// SHA-256 (hex) của file `.onnx`.
    pub sha256: String,
    #[serde(default)]
    pub objective: String,
    #[serde(default)]
    pub trained_on: String,
    #[serde(default)]
    pub notes: String,
}

/// Đường dẫn của một phiên bản mô hình: `<dir>/<version>.onnx` và `<dir>/<version>.json`.
#[derive(Debug, Clone)]
pub struct ModelFiles {
    pub onnx: PathBuf,
    pub json: PathBuf,
}

impl ModelFiles {
    pub fn new(dir: &Path, version: &str) -> Self {
        Self {
            onnx: dir.join(format!("{version}.onnx")),
            json: dir.join(format!("{version}.json")),
        }
    }

    /// Đọc siêu dữ liệu và kiểm: lược đồ, danh sách đặc trưng đúng thứ tự, SHA-256 khớp file. Không nạp mô hình.
    pub fn verify(&self) -> Result<ModelMeta, EtaError> {
        let json = std::fs::read_to_string(&self.json)
            .map_err(|e| EtaError::Missing(format!("{}: {e}", self.json.display())))?;
        let meta: ModelMeta = serde_json::from_str(&json)
            .map_err(|e| EtaError::Corrupt(format!("{}: {e}", self.json.display())))?;
        if meta.schema_version != SCHEMA_VERSION {
            return Err(EtaError::VersionMismatch(format!(
                "lược đồ {} (crate hiểu {SCHEMA_VERSION})",
                meta.schema_version
            )));
        }
        if meta.output != "residual_s" {
            return Err(EtaError::VersionMismatch(format!(
                "đầu ra «{}», crate chỉ hiểu «residual_s»",
                meta.output
            )));
        }
        let expected: Vec<&str> = FEATURES_V1.to_vec();
        let got: Vec<&str> = meta.features.iter().map(String::as_str).collect();
        if got != expected {
            return Err(EtaError::FeatureMismatch(format!(
                "mô hình cần {} đặc trưng {:?}, crate cung cấp {} đặc trưng {:?}",
                got.len(),
                got,
                expected.len(),
                expected
            )));
        }
        let bytes = std::fs::read(&self.onnx)
            .map_err(|e| EtaError::Missing(format!("{}: {e}", self.onnx.display())))?;
        let sha: String = Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        if sha != meta.sha256 {
            return Err(EtaError::Corrupt(format!(
                "SHA-256 của file .onnx là {sha}, siêu dữ liệu ghi {}",
                meta.sha256
            )));
        }
        Ok(meta)
    }
}
