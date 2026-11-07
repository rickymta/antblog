//! Cấu hình đọc từ biến môi trường, không có file và không có giá trị bí mật mặc định.

use std::net::SocketAddr;

/// Cấu hình của service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Địa chỉ nghe. Trong container phải là `0.0.0.0`, nghe `127.0.0.1` thì cổng published không tới được.
    pub addr: SocketAddr,
    /// Chuỗi kết nối PostgreSQL. `None` = chế độ không DB.
    pub pg_url: Option<String>,
}

/// Địa chỉ nghe mặc định: mọi giao diện, cổng 8080.
pub const DEFAULT_ADDR: &str = "0.0.0.0:8080";

impl Config {
    /// Đọc `LEDGER_ADDR` và `LEDGER_PG_URL` qua một hàm tra cứu (để test không đụng môi trường thật).
    ///
    /// # Errors
    /// `LEDGER_ADDR` không phải địa chỉ socket hợp lệ.
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Config, String> {
        let raw = get("LEDGER_ADDR").unwrap_or_else(|| DEFAULT_ADDR.to_owned());
        let addr = raw
            .parse()
            .map_err(|e| format!("LEDGER_ADDR={raw:?} không hợp lệ: {e}"))?;
        let pg_url = get("LEDGER_PG_URL").filter(|s| !s.is_empty());
        Ok(Config { addr, pg_url })
    }

    /// Đọc từ môi trường tiến trình.
    ///
    /// # Errors
    /// Như [`Config::from_lookup`].
    pub fn from_env() -> Result<Config, String> {
        Config::from_lookup(|k| std::env::var(k).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |k| {
            pairs
                .iter()
                .find(|(key, _)| *key == k)
                .map(|(_, v)| (*v).to_owned())
        }
    }

    #[test]
    fn default_listens_on_all_interfaces() {
        let cfg = Config::from_lookup(env(&[])).unwrap();
        assert_eq!(cfg.addr.to_string(), "0.0.0.0:8080");
        assert_eq!(cfg.pg_url, None);
    }

    #[test]
    fn empty_pg_url_means_no_database() {
        let cfg = Config::from_lookup(env(&[("LEDGER_PG_URL", "")])).unwrap();
        assert_eq!(cfg.pg_url, None);
    }

    #[test]
    fn bad_addr_is_rejected_with_the_value_in_message() {
        let err = Config::from_lookup(env(&[("LEDGER_ADDR", "not-an-addr")])).unwrap_err();
        assert!(err.contains("not-an-addr"), "{err}");
    }
}
