//! JSON Schema của JSON camelCase (`schemas/ledger_entry_posted.v1.schema.json`).
use anyhow::{Result, anyhow};
use serde_json::Value;

/// Nội dung schema, nhúng lúc biên dịch.
pub const SCHEMA_V1: &str = include_str!("../schemas/ledger_entry_posted.v1.schema.json");

/// Kiểm một JSON với schema v1; `Err` kèm danh sách vi phạm (vị trí: lý do).
pub fn validate_v1(instance: &Value) -> Result<(), Vec<String>> {
    let schema: Value = serde_json::from_str(SCHEMA_V1).expect("schema là JSON hợp lệ");
    let validator = jsonschema::validator_for(&schema).expect("schema hợp lệ");
    let errors: Vec<String> = validator
        .iter_errors(instance)
        .map(|e| format!("{}: {e}", e.instance_path()))
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Như `validate_v1` nhưng cho dữ liệu dạng bytes.
pub fn validate_v1_bytes(bytes: &[u8]) -> Result<Result<(), Vec<String>>> {
    let value: Value =
        serde_json::from_slice(bytes).map_err(|e| anyhow!("không phải JSON: {e}"))?;
    Ok(validate_v1(&value))
}
