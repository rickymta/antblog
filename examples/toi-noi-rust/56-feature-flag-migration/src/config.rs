//! Cấu hình kiểu an toàn: enum thay vì chuỗi, trường lạ bị từ chối, kiểm khoảng giá trị lúc khởi động.

use serde::Deserialize;
use thiserror::Error;

use crate::policy::PolicyVersion;

/// Cấu hình mẫu của lab (JSON, camelCase như các sự kiện của loạt).
pub const DEFAULT_JSON: &str = r#"{
  "defaultFeeRule": "v1",
  "migration": { "batchSize": 100, "lockTimeoutMs": 200, "maxAttempts": 5 }
}"#;

/// Hình dạng trên dây, chưa kiểm khoảng giá trị. `deny_unknown_fields` bắt lỗi gõ sai tên trường.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawConfig {
    default_fee_rule: PolicyVersion,
    migration: RawMigration,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawMigration {
    batch_size: i64,
    lock_timeout_ms: u32,
    max_attempts: u32,
}

/// Tham số chạy migration, đã kiểm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MigrationSettings {
    /// Số bút toán (hoặc sự kiện) mỗi lô backfill.
    pub batch_size: i64,
    /// Chờ khóa tối đa bấy nhiêu mili giây cho mỗi lần thử DDL.
    pub lock_timeout_ms: u32,
    /// Số lần thử DDL trước khi bỏ cuộc.
    pub max_attempts: u32,
}

/// Cấu hình đã kiểm. Có được giá trị này nghĩa là mọi trường hợp lệ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// Quy tắc phí khi cờ tắt.
    pub default_fee_rule: PolicyVersion,
    /// Tham số migration.
    pub migration: MigrationSettings,
}

/// Lỗi cấu hình. Service thoát ngay với lỗi này thay vì chạy với giá trị đoán.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    /// JSON sai cú pháp, thiếu trường, trường lạ hoặc giá trị enum lạ.
    #[error("không đọc được cấu hình: {0}")]
    Parse(String),
    /// Giá trị ngoài khoảng cho phép.
    #[error("`{field}` = {value}, cần {expected}")]
    OutOfRange {
        /// Tên trường.
        field: &'static str,
        /// Giá trị đã nhận.
        value: i64,
        /// Khoảng hợp lệ.
        expected: &'static str,
    },
}

fn in_range(
    field: &'static str,
    value: i64,
    lo: i64,
    hi: i64,
    expected: &'static str,
) -> Result<(), ConfigError> {
    if (lo..=hi).contains(&value) {
        Ok(())
    } else {
        Err(ConfigError::OutOfRange {
            field,
            value,
            expected,
        })
    }
}

impl Config {
    /// Đọc và kiểm cấu hình.
    ///
    /// # Errors
    /// [`ConfigError`] ở lỗi đầu tiên.
    pub fn from_json(json: &str) -> Result<Config, ConfigError> {
        let raw: RawConfig =
            serde_json::from_str(json).map_err(|e| ConfigError::Parse(e.to_string()))?;
        let m = &raw.migration;
        in_range(
            "migration.batchSize",
            m.batch_size,
            1,
            10_000,
            "1 đến 10000",
        )?;
        in_range(
            "migration.lockTimeoutMs",
            i64::from(m.lock_timeout_ms),
            10,
            5_000,
            "10 đến 5000",
        )?;
        in_range(
            "migration.maxAttempts",
            i64::from(m.max_attempts),
            1,
            20,
            "1 đến 20",
        )?;
        Ok(Config {
            default_fee_rule: raw.default_fee_rule,
            migration: MigrationSettings {
                batch_size: m.batch_size,
                lock_timeout_ms: m.lock_timeout_ms,
                max_attempts: m.max_attempts,
            },
        })
    }
}

/// Cách SAI để đối chứng: cấu hình là chuỗi, gì không nhận ra thì rơi về mặc định.
/// `"V2"`, `"v 2"`, `"v3"` đều thành `V1` mà không báo gì.
pub fn legacy_fee_rule(raw: &str) -> PolicyVersion {
    if raw == "v2" {
        PolicyVersion::V2
    } else {
        PolicyVersion::V1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_json_is_valid() {
        let cfg = Config::from_json(DEFAULT_JSON).unwrap();
        assert_eq!(cfg.default_fee_rule, PolicyVersion::V1);
        assert_eq!(cfg.migration.batch_size, 100);
        assert_eq!(cfg.migration.lock_timeout_ms, 200);
        assert_eq!(cfg.migration.max_attempts, 5);
    }

    #[test]
    fn unknown_enum_value_is_rejected_with_the_accepted_list() {
        let json = DEFAULT_JSON.replace("\"v1\"", "\"v3\"");
        let err = Config::from_json(&json).unwrap_err().to_string();
        assert!(err.contains("unknown variant `v3`"), "{err}");
        assert!(err.contains("`v1`") && err.contains("`v2`"), "{err}");
    }

    #[test]
    fn misspelled_field_is_rejected() {
        let json = DEFAULT_JSON.replace("batchSize", "batchSise");
        let err = Config::from_json(&json).unwrap_err().to_string();
        assert!(err.contains("batchSise"), "{err}");
    }

    #[test]
    fn out_of_range_values_are_rejected() {
        let zero = DEFAULT_JSON.replace("100", "0");
        assert!(matches!(
            Config::from_json(&zero),
            Err(ConfigError::OutOfRange {
                field: "migration.batchSize",
                ..
            })
        ));
        let big = DEFAULT_JSON.replace("\"lockTimeoutMs\": 200", "\"lockTimeoutMs\": 60000");
        assert!(matches!(
            Config::from_json(&big),
            Err(ConfigError::OutOfRange {
                field: "migration.lockTimeoutMs",
                ..
            })
        ));
    }

    #[test]
    fn legacy_string_config_silently_falls_back() {
        assert_eq!(legacy_fee_rule("v2"), PolicyVersion::V2);
        assert_eq!(legacy_fee_rule("V2"), PolicyVersion::V1);
        assert_eq!(legacy_fee_rule("v3"), PolicyVersion::V1);
    }
}
