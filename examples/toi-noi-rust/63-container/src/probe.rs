//! Client HTTP tối thiểu bằng `std`, dùng cho subcommand `healthcheck` và cho `--verify`.
//!
//! Image distroless không có shell, `curl` hay `wget`. Nếu muốn `HEALTHCHECK` thì chính binary phải tự
//! hỏi mình. Không kéo thêm crate HTTP client vào image chỉ để làm việc này.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

/// Hạn cho cả kết nối lẫn mỗi lần đọc/ghi; lớn hơn hạn kiểm DB (2 giây) của `/readyz`.
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// Phản hồi đã tách: mã trạng thái và thân.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply {
    /// Mã HTTP.
    pub status: u16,
    /// Thân phản hồi.
    pub body: String,
}

/// `GET path` tới `addr`, dùng HTTP/1.0 + `Connection: close` để đọc tới EOF là xong.
///
/// # Errors
/// Không kết nối được, quá hạn hoặc phản hồi không phải HTTP.
pub fn http_get(addr: SocketAddr, path: &str) -> Result<Reply, String> {
    let mut stream = TcpStream::connect_timeout(&addr, PROBE_TIMEOUT)
        .map_err(|e| format!("kết nối {addr}: {e}"))?;
    stream.set_read_timeout(Some(PROBE_TIMEOUT)).ok();
    stream.set_write_timeout(Some(PROBE_TIMEOUT)).ok();
    let request = format!("GET {path} HTTP/1.0\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("gửi request: {e}"))?;
    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .map_err(|e| format!("đọc phản hồi: {e}"))?;
    parse_reply(&String::from_utf8_lossy(&raw))
}

/// Tách dòng trạng thái và thân.
fn parse_reply(raw: &str) -> Result<Reply, String> {
    let (head, body) = raw
        .split_once("\r\n\r\n")
        .ok_or_else(|| format!("phản hồi không có phần thân: {raw:?}"))?;
    let status_line = head.lines().next().unwrap_or_default();
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("dòng trạng thái lạ: {status_line:?}"))?;
    Ok(Reply {
        status,
        body: body.to_owned(),
    })
}

/// Kiểm `/healthz` của chính service: `Ok` khi trả 200.
///
/// # Errors
/// Không kết nối được hoặc mã khác 200.
pub fn healthcheck(addr: SocketAddr) -> Result<(), String> {
    let reply = http_get(addr, "/healthz")?;
    if reply.status == 200 {
        Ok(())
    } else {
        Err(format!("/healthz trả {}", reply.status))
    }
}

/// Địa chỉ để tự gọi: cùng cổng với địa chỉ nghe, nhưng luôn là loopback.
/// `0.0.0.0:8080` là địa chỉ để nghe, không phải địa chỉ để kết nối.
#[must_use]
pub fn loopback_of(listen: SocketAddr) -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], listen.port()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_status_and_body() {
        let reply =
            parse_reply("HTTP/1.1 503 Service Unavailable\r\nA: b\r\n\r\n{\"x\":1}").unwrap();
        assert_eq!(reply.status, 503);
        assert_eq!(reply.body, "{\"x\":1}");
    }

    #[test]
    fn rejects_non_http() {
        assert!(parse_reply("garbage").is_err());
        assert!(parse_reply("NOPE\r\n\r\n").is_err());
    }

    #[test]
    fn unspecified_address_becomes_loopback() {
        let listen: SocketAddr = "0.0.0.0:8080".parse().unwrap();
        assert_eq!(loopback_of(listen).to_string(), "127.0.0.1:8080");
    }

    #[test]
    fn closed_port_is_an_error() {
        // Bind cổng 0 để lấy một cổng chắc chắn đang trống, thả listener rồi probe vào đó.
        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        assert!(healthcheck(addr).is_err());
    }
}
