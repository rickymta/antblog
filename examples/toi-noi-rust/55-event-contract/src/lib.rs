//! Lab 55: hợp đồng sự kiện `LedgerEntryPosted` xuyên ngôn ngữ.
//!
//! - `typed`: kiểu prost sinh từ proto v1 và bút toán mẫu.
//! - `pool`: descriptor set của mọi schema, đọc/ghi động bằng prost-reflect.
//! - `semantics`: bảng «trường nào ở đâu» của từng schema và giá trị đúng.
//! - `matrix`: ca đo tương thích tiến/lùi, phân loại OK/LOSS/ERR, rollout.
//! - `compat`: kiểm tương thích bằng so descriptor (thay cho schema registry).
//! - `json_contract`: JSON Schema của JSON camelCase.
//! - `fixtures`: sinh và kiểm tệp fixture commit sẵn.
//! - `consumers`: chạy consumer C#, Go, Python thật và gộp kết quả.

pub mod compat;
pub mod consumers;
pub mod fixtures;
pub mod json_contract;
pub mod matrix;
pub mod pool;
pub mod semantics;
pub mod typed;

use std::path::PathBuf;

/// Thư mục gốc của lab (nơi có `schemas/`, `fixtures/`, `consumers/`).
pub fn lab_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
