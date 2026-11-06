//! Client HTTP/1.1 tối giản qua TCP, chỉ đủ để lab gọi endpoint của chính nó (không thêm reqwest).

use std::net::SocketAddr;

use anyhow::{Context, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Phản hồi đã tách: mã trạng thái và body UTF-8.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// Mã trạng thái HTTP.
    pub status: u16,
    /// Giá trị header `WWW-Authenticate`, nếu có.
    pub www_authenticate: Option<String>,
    /// Body.
    pub body: String,
}

/// `POST` body thô tới `addr`, kèm các header cho trước. Mỗi lời gọi một kết nối (`Connection: close`).
///
/// # Errors
/// Lỗi kết nối hoặc phản hồi sai dạng.
pub async fn post(
    addr: SocketAddr,
    path: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> Result<HttpResponse> {
    let mut head = format!(
        "POST {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n",
        body.len()
    );
    for (name, value) in headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str("\r\n");

    let mut stream = TcpStream::connect(addr)
        .await
        .context("kết nối tới server lab")?;
    stream.write_all(head.as_bytes()).await?;
    stream.write_all(body).await?;
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await?;

    let text = String::from_utf8_lossy(&raw);
    let (head, body) = text
        .split_once("\r\n\r\n")
        .context("phản hồi thiếu dòng trống")?;
    let status = head
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .context("không đọc được mã trạng thái")?;
    let www_authenticate = head.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("www-authenticate")
            .then(|| value.trim().to_string())
    });
    Ok(HttpResponse {
        status,
        www_authenticate,
        body: body.to_string(),
    })
}
