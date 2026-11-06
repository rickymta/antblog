//! Cấu hình đọc từ biến môi trường, kiểm một lần lúc khởi động (rút gọn từ bài 29).

use std::fmt;
use std::net::SocketAddr;
use std::time::Duration;

use thiserror::Error;

/// Lỗi cấu hình: gom mọi vấn đề rồi báo một lần.
#[derive(Debug, Error)]
#[error("cấu hình không hợp lệ:\n{}", .0.iter().map(|p| format!("  - {p}")).collect::<Vec<_>>().join("\n"))]
pub struct ConfigError(pub Vec<String>);

/// Khóa JWT: không bao giờ vào log. `Debug` che nội dung.
#[derive(Clone)]
pub struct Secret(pub Vec<u8>);

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<đã che>)")
    }
}

/// Mọi thứ service cần biết lúc chạy.
#[derive(Debug, Clone)]
pub struct Config {
    /// URL PostgreSQL (`TOI_NOI_PG_URL`).
    pub pg_url: String,
    /// Schema dùng làm `search_path`; `None` là mặc định của role.
    pub pg_schema: Option<String>,
    /// Danh sách broker Kafka (`TOI_NOI_KAFKA`).
    pub kafka: String,
    /// Topic vào: `<prefix>.trip.completed`.
    pub topic_in: String,
    /// Topic ra: `<prefix>.ledger.entry-posted`.
    pub topic_out: String,
    /// Consumer group của service.
    pub group: String,
    /// Địa chỉ HTTP; cổng 0 là để hệ điều hành chọn.
    pub http_addr: SocketAddr,
    /// Địa chỉ gRPC; cổng 0 là để hệ điều hành chọn.
    pub grpc_addr: SocketAddr,
    /// Chu kỳ quét outbox khi không có ai đánh thức relay.
    pub relay_poll: Duration,
    /// Tổng thời gian tối đa cho cả quy trình tắt.
    pub shutdown_timeout: Duration,
    /// Khóa HS256 để kiểm JWT (`LEDGER_JWT_SECRET`, tối thiểu 32 byte).
    pub jwt_secret: Secret,
}

impl Config {
    /// Đọc từ biến môi trường thật của tiến trình.
    ///
    /// # Errors
    /// Thiếu biến bắt buộc hoặc giá trị sai dạng.
    pub fn from_env() -> Result<Config, ConfigError> {
        Config::from_lookup(|name| std::env::var(name).ok())
    }

    /// Như [`Config::from_env`] nhưng nguồn là một hàm tra cứu (test không đụng env toàn cục).
    ///
    /// # Errors
    /// Như [`Config::from_env`].
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Config, ConfigError> {
        let mut problems = Vec::new();
        let mut required = |name: &str| {
            get(name)
                .filter(|v| !v.trim().is_empty())
                .unwrap_or_else(|| {
                    problems.push(format!("thiếu biến {name}"));
                    String::new()
                })
        };
        let pg_url = required("TOI_NOI_PG_URL");
        let kafka = required("TOI_NOI_KAFKA");
        let secret = required("LEDGER_JWT_SECRET");
        if !secret.is_empty() && secret.len() < ledger_api::auth::MIN_SECRET_BYTES {
            problems.push(format!(
                "LEDGER_JWT_SECRET cần ít nhất {} byte",
                ledger_api::auth::MIN_SECRET_BYTES
            ));
        }

        let prefix = get("LEDGER_TOPIC_PREFIX").unwrap_or_else(|| "toi-noi".to_string());
        let group = get("LEDGER_CONSUMER_GROUP").unwrap_or_else(|| "ledger-service".to_string());
        let mut parse = |name: &str, default: &str| -> SocketAddr {
            let raw = get(name).unwrap_or_else(|| default.to_string());
            raw.parse().unwrap_or_else(|_| {
                problems.push(format!("{name} không phải host:port: {raw:?}"));
                SocketAddr::from(([127, 0, 0, 1], 0))
            })
        };
        let http_addr = parse("LEDGER_HTTP_ADDR", "127.0.0.1:0");
        let grpc_addr = parse("LEDGER_GRPC_ADDR", "127.0.0.1:0");
        let shutdown_ms = get("LEDGER_SHUTDOWN_TIMEOUT_MS")
            .map_or(Ok(10_000), |raw| raw.parse::<u64>())
            .unwrap_or_else(|_| {
                problems.push("LEDGER_SHUTDOWN_TIMEOUT_MS không phải số nguyên".to_string());
                10_000
            });

        if !problems.is_empty() {
            return Err(ConfigError(problems));
        }
        Ok(Config {
            pg_url,
            pg_schema: get("LEDGER_PG_SCHEMA").filter(|s| !s.is_empty()),
            kafka,
            topic_in: format!("{prefix}.trip.completed"),
            topic_out: format!("{prefix}.ledger.entry-posted"),
            group,
            http_addr,
            grpc_addr,
            relay_poll: Duration::from_millis(200),
            shutdown_timeout: Duration::from_millis(shutdown_ms),
            jwt_secret: Secret(secret.into_bytes()),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn lookup(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        move |name| map.get(name).cloned()
    }

    #[test]
    fn missing_variables_are_reported_together() {
        let err = Config::from_lookup(lookup(&[])).unwrap_err();
        assert_eq!(err.0.len(), 3, "{err}");
    }

    #[test]
    fn short_secret_is_refused_and_never_printed() {
        let err = Config::from_lookup(lookup(&[
            ("TOI_NOI_PG_URL", "postgres://x"),
            ("TOI_NOI_KAFKA", "127.0.0.1:9092"),
            ("LEDGER_JWT_SECRET", "ngan"),
        ]))
        .unwrap_err();
        assert!(err.to_string().contains("ít nhất 32 byte"));
        assert!(!err.to_string().contains("ngan"));
    }

    #[test]
    fn prefix_decides_topic_names_and_secret_is_masked() {
        let cfg = Config::from_lookup(lookup(&[
            ("TOI_NOI_PG_URL", "postgres://x"),
            ("TOI_NOI_KAFKA", "127.0.0.1:9092"),
            ("LEDGER_TOPIC_PREFIX", "lab.58.abc"),
            ("LEDGER_JWT_SECRET", "0123456789abcdef0123456789abcdef"),
        ]))
        .unwrap();
        assert_eq!(cfg.topic_in, "lab.58.abc.trip.completed");
        assert_eq!(cfg.topic_out, "lab.58.abc.ledger.entry-posted");
        assert!(!format!("{cfg:?}").contains("0123456789abcdef"));
    }
}
