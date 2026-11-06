//! Nạp các file trong `deploy/`: JSON của ECS, YAML của Container Apps và file giả định.

use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;

/// Thư mục `deploy/` của lab, tính từ thư mục crate nên chạy từ đâu cũng thấy.
pub const DEFAULT_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/deploy");

/// Sáu file cấu hình, đều đã parse về `serde_json::Value` để duyệt chung một kiểu.
#[derive(Debug, Clone)]
pub struct Manifests {
    pub ecs_task: Value,
    pub ecs_service: Value,
    pub ecs_target_group: Value,
    pub ecs_scalable_target: Value,
    pub ecs_scaling_policy: Value,
    pub container_app: Value,
}

impl Manifests {
    /// Đọc và parse đủ sáu file.
    ///
    /// # Errors
    /// Thiếu file hoặc JSON/YAML sai cú pháp; lỗi nói rõ tên file.
    pub fn load(dir: &Path) -> Result<Manifests> {
        Ok(Manifests {
            ecs_task: read_json(&dir.join("ecs-task-definition.json"))?,
            ecs_service: read_json(&dir.join("ecs-service.json"))?,
            ecs_target_group: read_json(&dir.join("ecs-target-group.json"))?,
            ecs_scalable_target: read_json(&dir.join("ecs-scalable-target.json"))?,
            ecs_scaling_policy: read_json(&dir.join("ecs-scaling-policy.json"))?,
            container_app: read_yaml(&dir.join("containerapp.yaml"))?,
        })
    }

    /// Tên và nội dung từng file, theo thứ tự cố định (dùng để quét chung).
    pub fn all(&self) -> [(&'static str, &Value); 6] {
        [
            ("ecs-task-definition.json", &self.ecs_task),
            ("ecs-service.json", &self.ecs_service),
            ("ecs-target-group.json", &self.ecs_target_group),
            ("ecs-scalable-target.json", &self.ecs_scalable_target),
            ("ecs-scaling-policy.json", &self.ecs_scaling_policy),
            ("containerapp.yaml", &self.container_app),
        ]
    }
}

fn read_json(path: &Path) -> Result<Value> {
    let text = std::fs::read_to_string(path).with_context(|| format!("đọc {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parse JSON {}", path.display()))
}

fn read_yaml(path: &Path) -> Result<Value> {
    let text = std::fs::read_to_string(path).with_context(|| format!("đọc {}", path.display()))?;
    serde_yaml_ng::from_str(&text).with_context(|| format!("parse YAML {}", path.display()))
}

/// Giả định của lab (không đo từ môi trường thật), gom một chỗ để kiểm chéo.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assumptions {
    /// Chỉ để người đọc file thấy cảnh báo.
    #[allow(dead_code)]
    pub note: String,
    pub kafka: KafkaAssumptions,
    pub postgres: PostgresAssumptions,
    pub shutdown: ShutdownAssumptions,
    pub startup: StartupAssumptions,
    pub autoscale: AutoscaleAssumptions,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KafkaAssumptions {
    pub topic: String,
    pub consumer_group: String,
    pub partitions: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresAssumptions {
    /// `max_connections` của instance managed.
    pub max_connections: u32,
    /// Phần dành cho superuser, migration, công cụ quản trị, giám sát.
    pub reserved_connections: u32,
    /// Trần pool của một replica (`PgPoolOptions::max_connections` trong bài 58).
    pub pool_max_per_replica: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShutdownAssumptions {
    /// Giá trị `LEDGER_SHUTDOWN_TIMEOUT_MS`: hạn chung của các bước tắt.
    pub shutdown_timeout_ms: u64,
    /// Phần việc nằm ngoài hạn chung (đóng pool, flush), ước lượng.
    pub after_deadline_allowance_ms: u64,
    /// Dư địa tối thiểu giữa lúc tắt xong và SIGKILL.
    pub margin_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartupAssumptions {
    /// Từ lúc process chạy tới lúc consumer có partition đầu tiên, tính bằng giây.
    pub ready_allowance_s: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AutoscaleAssumptions {
    /// Lag mục tiêu mỗi replica.
    pub lag_threshold: u64,
    /// Số replica tối đa có thể chạy cùng lúc khi đổi revision, tính phần trăm của `maxReplicas`.
    pub container_apps_surge_percent: u64,
}

impl Assumptions {
    /// Đọc `assumptions.json`.
    ///
    /// # Errors
    /// Thiếu file, sai cú pháp hoặc có khóa lạ.
    pub fn load(dir: &Path) -> Result<Assumptions> {
        let path = dir.join("assumptions.json");
        let text =
            std::fs::read_to_string(&path).with_context(|| format!("đọc {}", path.display()))?;
        serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))
    }
}

/// Chuỗi tại một JSON pointer (`/a/0/b`).
pub fn str_at<'a>(v: &'a Value, pointer: &str) -> Option<&'a str> {
    v.pointer(pointer)?.as_str()
}

/// Số nguyên không dấu tại một JSON pointer.
pub fn u64_at(v: &Value, pointer: &str) -> Option<u64> {
    v.pointer(pointer)?.as_u64()
}

/// Nguồn của một biến môi trường trong container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvSource {
    /// Giá trị ghi thẳng trong file.
    Literal(String),
    /// Tham chiếu tới kho bí mật (ARN của Secrets Manager hoặc tên secret của Container Apps).
    Secret(String),
}

/// Biến môi trường của container ECS: `environment` là literal, `secrets` là tham chiếu.
pub fn ecs_env(task: &Value) -> Vec<(String, EnvSource)> {
    let mut out = Vec::new();
    let base = "/containerDefinitions/0";
    for (key, field, literal) in [
        ("environment", "value", true),
        ("secrets", "valueFrom", false),
    ] {
        let items = task
            .pointer(&format!("{base}/{key}"))
            .and_then(Value::as_array);
        for item in items.into_iter().flatten() {
            if let (Some(name), Some(value)) = (
                item.get("name").and_then(Value::as_str),
                item.get(field).and_then(Value::as_str),
            ) {
                let source = if literal {
                    EnvSource::Literal(value.to_string())
                } else {
                    EnvSource::Secret(value.to_string())
                };
                out.push((name.to_string(), source));
            }
        }
    }
    out
}

/// Biến môi trường của container Container Apps: `value` là literal, `secretRef` là tham chiếu.
pub fn aca_env(app: &Value) -> Vec<(String, EnvSource)> {
    let items = app
        .pointer("/properties/template/containers/0/env")
        .and_then(Value::as_array);
    let mut out = Vec::new();
    for item in items.into_iter().flatten() {
        let Some(name) = item.get("name").and_then(Value::as_str) else {
            continue;
        };
        if let Some(value) = item.get("value").and_then(Value::as_str) {
            out.push((name.to_string(), EnvSource::Literal(value.to_string())));
        } else if let Some(secret) = item.get("secretRef").and_then(Value::as_str) {
            out.push((name.to_string(), EnvSource::Secret(secret.to_string())));
        }
    }
    out
}

/// Giá trị literal của một biến, nếu có.
pub fn literal<'a>(env: &'a [(String, EnvSource)], name: &str) -> Option<&'a str> {
    env.iter().find_map(|(n, source)| match source {
        EnvSource::Literal(v) if n == name => Some(v.as_str()),
        _ => None,
    })
}
