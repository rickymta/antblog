//! Cấu hình: file mặc định + biến môi trường, kiểm tra một lần lúc khởi động.

use std::collections::HashMap;
use std::path::Path;

use config::{Config, Environment, File};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use thiserror::Error;
use tracing_subscriber::EnvFilter;

/// Đường dẫn file mặc định, tính từ thư mục của crate nên chạy từ đâu cũng tìm thấy.
pub const DEFAULT_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/config/default.toml");

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub log: LogSettings,
}

#[derive(Debug, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    // Không có trong file mặc định; thiếu thì rỗng và `validate` báo lỗi.
    #[serde(default = "no_password")]
    pub password: SecretString,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize)]
pub struct LogSettings {
    /// Cú pháp của `EnvFilter`, ví dụ `info,ledger_29_config_tracing=debug`.
    pub filter: String,
}

fn no_password() -> SecretString {
    SecretString::from(String::new())
}

/// Lỗi khởi động: hoặc không đọc/parse được, hoặc đọc được nhưng giá trị sai.
#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("không đọc được cấu hình: {0}")]
    Load(#[from] config::ConfigError),
    #[error("cấu hình không hợp lệ, {} vấn đề:\n{}", .0.len(), bullets(.0))]
    Invalid(Vec<String>),
}

fn bullets(problems: &[String]) -> String {
    problems
        .iter()
        .map(|p| format!("  - {p}"))
        .collect::<Vec<_>>()
        .join("\n")
}

impl Settings {
    /// Đọc từ file mặc định và biến môi trường thật của tiến trình.
    pub fn load() -> Result<Settings, SettingsError> {
        Settings::load_from(Path::new(DEFAULT_FILE), None)
    }

    /// `env = Some(map)` thay biến môi trường thật bằng một map (test không đụng env toàn cục).
    pub fn load_from(
        file: &Path,
        env: Option<HashMap<String, String>>,
    ) -> Result<Settings, SettingsError> {
        // `LEDGER__DATABASE__MAX_CONNECTIONS` -> khoá `database.max_connections`.
        // Không bật `try_parsing`: nó đổi mật khẩu `00012345678` thành số 12345678.
        let environment = Environment::with_prefix("LEDGER")
            .separator("__")
            .source(env);
        let settings: Settings = Config::builder()
            .add_source(File::from(file))
            .add_source(environment) // nguồn thêm sau thắng nguồn thêm trước
            .build()?
            .try_deserialize()?;
        settings.validate()?;
        Ok(settings)
    }

    /// Gom MỌI vấn đề rồi trả một lần, thay vì dừng ở lỗi đầu tiên.
    fn validate(&self) -> Result<(), SettingsError> {
        let mut problems = Vec::new();
        if self.server.port == 0 {
            problems.push("server.port phải khác 0".to_string());
        }
        if self.database.host.trim().is_empty() {
            problems.push("database.host rỗng".to_string());
        }
        if !(1..=100).contains(&self.database.max_connections) {
            problems.push(format!(
                "database.max_connections phải trong 1..=100, nhận {}",
                self.database.max_connections
            ));
        }
        if self.database.password.expose_secret().len() < 8 {
            problems.push(
                "database.password thiếu hoặc ngắn hơn 8 ký tự (đặt LEDGER__DATABASE__PASSWORD)"
                    .to_string(),
            );
        }
        if EnvFilter::try_new(&self.log.filter).is_err() {
            problems.push(format!(
                "log.filter không parse được: {:?}",
                self.log.filter
            ));
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(SettingsError::Invalid(problems))
        }
    }
}

impl DatabaseSettings {
    /// Chuỗi kết nối chứa mật khẩu, nên trả `SecretString`: nơi gọi phải `expose_secret()` tường minh.
    pub fn connection_url(&self) -> SecretString {
        SecretString::from(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.name
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load(pairs: &[(&str, &str)]) -> Result<Settings, SettingsError> {
        let env = pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        Settings::load_from(Path::new(DEFAULT_FILE), Some(env))
    }

    const PASSWORD: (&str, &str) = ("LEDGER__DATABASE__PASSWORD", "lab-only-secret");

    #[test]
    fn env_overrides_default_file() {
        let from_file = load(&[PASSWORD]).unwrap();
        assert_eq!(from_file.database.max_connections, 10);
        let overridden = load(&[PASSWORD, ("LEDGER__DATABASE__MAX_CONNECTIONS", "20")]).unwrap();
        assert_eq!(overridden.database.max_connections, 20);
        assert_eq!(overridden.database.name, "toi_noi_rust_lab"); // khoá khác giữ nguyên từ file
    }

    #[test]
    fn missing_password_is_a_startup_error() {
        let Err(SettingsError::Invalid(problems)) = load(&[]) else {
            panic!("thiếu mật khẩu phải là lỗi");
        };
        assert_eq!(problems.len(), 1);
        assert!(problems[0].contains("database.password"));
    }

    #[test]
    fn all_problems_are_reported_together() {
        let Err(SettingsError::Invalid(problems)) = load(&[
            ("LEDGER__DATABASE__MAX_CONNECTIONS", "500"),
            ("LEDGER__SERVER__PORT", "0"),
            ("LEDGER__LOG__FILTER", "info,[[hỏng"),
        ]) else {
            panic!("phải là lỗi validate");
        };
        assert_eq!(problems.len(), 4, "{problems:?}");
    }

    #[test]
    fn unparsable_number_is_a_load_error() {
        let result = load(&[PASSWORD, ("LEDGER__DATABASE__MAX_CONNECTIONS", "nhieu")]);
        assert!(matches!(result, Err(SettingsError::Load(_))));
    }

    #[test]
    fn numeric_looking_password_is_kept_verbatim() {
        let settings = load(&[("LEDGER__DATABASE__PASSWORD", "00012345678")]).unwrap();
        assert_eq!(settings.database.password.expose_secret(), "00012345678");
    }

    #[test]
    fn debug_hides_password_but_connection_url_needs_expose() {
        let settings = load(&[PASSWORD]).unwrap();
        let debug = format!("{settings:?}");
        assert!(!debug.contains("lab-only-secret"));
        assert!(debug.contains("REDACTED"));
        let url = settings.database.connection_url();
        assert_eq!(
            url.expose_secret(),
            "postgres://toi_noi:lab-only-secret@127.0.0.1:5433/toi_noi_rust_lab"
        );
    }
}
