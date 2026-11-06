//! HTTP/1.1 tối thiểu qua `TcpStream`: đủ để kiểm status, không thêm crate client.

use std::net::SocketAddr;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

async fn request(addr: SocketAddr, head: &str, body: &str) -> Result<(u16, String)> {
    let mut stream = tokio::time::timeout(Duration::from_secs(2), TcpStream::connect(addr))
        .await
        .context("connect quá thời gian")?
        .context("connect")?;
    let text = format!(
        "{head} HTTP/1.1\r\nHost: lab\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(text.as_bytes()).await?;
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await?;
    let raw = String::from_utf8_lossy(&raw).to_string();
    let status: u16 = raw
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .with_context(|| format!("không đọc được status từ {raw:?}"))?;
    let Some((_, body)) = raw.split_once("\r\n\r\n") else {
        bail!("phản hồi không có body: {raw:?}");
    };
    Ok((status, body.to_string()))
}

/// `GET path` → (status, body).
///
/// # Errors
/// Không kết nối được (ví dụ cổng đã đóng) hoặc phản hồi sai định dạng.
pub async fn http_get(addr: SocketAddr, path: &str) -> Result<(u16, String)> {
    request(addr, &format!("GET {path}"), "").await
}

/// `POST path` với body JSON → (status, body).
///
/// # Errors
/// Như [`http_get`].
pub async fn http_post(addr: SocketAddr, path: &str, body: &str) -> Result<(u16, String)> {
    request(addr, &format!("POST {path}"), body).await
}
