//! Cấu hình của `ledger-service` đọc từ biến môi trường, kiểu bài 29 (rút gọn từ `Config` của bài 58).
//! Dùng để chứng minh file triển khai cấp đủ và đúng mọi biến service cần.

use std::net::SocketAddr;
use std::time::Duration;

use secrecy::{ExposeSecret, SecretString};
use thiserror::Error;

/// Khóa JWT tối thiểu, theo bài 58.
pub const MIN_JWT_BYTES: usize = 32;

/// Gom mọi vấn đề rồi báo một lần, như bài 29.
#[derive(Debug, Error)]
#[error("cấu hình không hợp lệ, {} vấn đề: {}", .0.len(), .0.join("; "))]
pub struct SettingsError(pub Vec<String>);

/// Giá trị đã kiểm. Hai trường bí mật mang kiểu `SecretString` nên `Debug` không in ra.
#[derive(Debug)]
pub struct Settings {
    pub pg_url: SecretString,
    pub jwt_secret: SecretString,
    pub kafka: String,
    pub topic_in: String,
    pub group: String,
    pub http_addr: SocketAddr,
    pub grpc_addr: SocketAddr,
    pub shutdown_timeout: Duration,
}

impl Settings {
    /// Nguồn là một hàm tra cứu, nên test không đụng biến môi trường toàn cục.
    ///
    /// # Errors
    /// Thiếu biến bắt buộc hoặc giá trị sai dạng; mọi lỗi được gom vào một [`SettingsError`].
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Settings, SettingsError> {
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
        let jwt = required("LEDGER_JWT_SECRET");
        if !jwt.is_empty() && jwt.len() < MIN_JWT_BYTES {
            problems.push(format!(
                "LEDGER_JWT_SECRET cần ít nhất {MIN_JWT_BYTES} byte"
            ));
        }
        let prefix = get("LEDGER_TOPIC_PREFIX").unwrap_or_else(|| "toi-noi".to_string());
        let group = get("LEDGER_CONSUMER_GROUP").unwrap_or_else(|| "ledger-service".to_string());
        let mut addr = |name: &str| -> SocketAddr {
            // Mặc định của bài 58 là loopback, cổng 0: trong container phải đặt rõ.
            let raw = get(name).unwrap_or_else(|| "127.0.0.1:0".to_string());
            raw.parse().unwrap_or_else(|_| {
                problems.push(format!("{name} không phải host:port: {raw:?}"));
                SocketAddr::from(([127, 0, 0, 1], 0))
            })
        };
        let http_addr = addr("LEDGER_HTTP_ADDR");
        let grpc_addr = addr("LEDGER_GRPC_ADDR");
        let shutdown_ms = get("LEDGER_SHUTDOWN_TIMEOUT_MS")
            .map_or(Ok(10_000), |raw| raw.parse::<u64>())
            .unwrap_or_else(|_| {
                problems.push("LEDGER_SHUTDOWN_TIMEOUT_MS không phải số nguyên".to_string());
                10_000
            });
        if !problems.is_empty() {
            return Err(SettingsError(problems));
        }
        Ok(Settings {
            pg_url: SecretString::from(pg_url),
            jwt_secret: SecretString::from(jwt),
            kafka,
            topic_in: format!("{prefix}.trip.completed"),
            group,
            http_addr,
            grpc_addr,
            shutdown_timeout: Duration::from_millis(shutdown_ms),
        })
    }

    /// Độ dài chuỗi kết nối, để chứng minh đọc được mà không in nội dung.
    pub fn pg_url_len(&self) -> usize {
        self.pg_url.expose_secret().len()
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

    const JWT: &str = "giá-trị-thử-đủ-dài-cho-khóa-jwt-0123456789";
    const PG: &str = "postgres://thu:mat-khau-gia@db.invalid:5432/ledger";

    #[test]
    fn reads_secrets_from_env_and_hides_them_in_debug() {
        let s = Settings::from_lookup(lookup(&[
            ("TOI_NOI_PG_URL", PG),
            ("TOI_NOI_KAFKA", "kafka.invalid:9092"),
            ("LEDGER_JWT_SECRET", JWT),
        ]))
        .unwrap();
        assert_eq!(s.pg_url_len(), PG.len());
        let shown = format!("{s:?}");
        assert!(!shown.contains("mat-khau-gia"), "{shown}");
        assert!(!shown.contains(JWT), "{shown}");
        assert!(shown.contains("REDACTED"));
    }

    #[test]
    fn missing_secrets_are_reported_together() {
        let err =
            Settings::from_lookup(lookup(&[("TOI_NOI_KAFKA", "kafka.invalid:9092")])).unwrap_err();
        assert_eq!(err.0.len(), 2, "{err}");
        assert!(err.to_string().contains("TOI_NOI_PG_URL"));
        assert!(err.to_string().contains("LEDGER_JWT_SECRET"));
    }

    #[test]
    fn short_jwt_secret_is_rejected() {
        let err = Settings::from_lookup(lookup(&[
            ("TOI_NOI_PG_URL", PG),
            ("TOI_NOI_KAFKA", "kafka.invalid:9092"),
            ("LEDGER_JWT_SECRET", "ngan"),
        ]))
        .unwrap_err();
        assert!(err.to_string().contains("ít nhất 32 byte"));
    }

    #[test]
    fn defaults_bind_to_loopback_port_zero() {
        let s = Settings::from_lookup(lookup(&[
            ("TOI_NOI_PG_URL", PG),
            ("TOI_NOI_KAFKA", "kafka.invalid:9092"),
            ("LEDGER_JWT_SECRET", JWT),
        ]))
        .unwrap();
        assert!(s.http_addr.ip().is_loopback());
        assert_eq!(s.http_addr.port(), 0);
        assert_eq!(s.topic_in, "toi-noi.trip.completed");
        assert_eq!(s.shutdown_timeout, Duration::from_millis(10_000));
    }
}
