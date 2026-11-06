//! Client HTTP/1.1 tối thiểu qua `TcpStream`, đủ để gọi server thật trên `127.0.0.1:0`.
//! Cố ý không dùng reqwest: lab giữ ít dependency, và gõ tay byte HTTP cho thấy rõ
//! phần nào của giao thức mà `oneshot` bỏ qua. Chỉ dùng trong lab và test.

use std::net::SocketAddr;

use anyhow::{Context, Result, bail};
use axum::Router;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

/// Phản hồi đã tách status, header, thân.
#[derive(Debug)]
pub struct RawResponse {
    /// Mã HTTP.
    pub status: u16,
    /// Header, tên viết thường, theo thứ tự nhận.
    pub headers: Vec<(String, String)>,
    /// Thân dạng chuỗi.
    pub body: String,
}

impl RawResponse {
    /// Giá trị header (không phân biệt hoa thường).
    pub fn header(&self, name: &str) -> Option<&str> {
        let name = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.as_str())
    }

    /// Thân đọc như JSON.
    pub fn json(&self) -> Result<serde_json::Value> {
        serde_json::from_str(&self.body).context("thân không phải JSON")
    }
}

/// Gửi một request, đọc tới khi server đóng kết nối (`Connection: close`).
pub async fn request(
    addr: SocketAddr,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<&str>,
) -> Result<RawResponse> {
    let mut stream = TcpStream::connect(addr).await?;
    let mut head = format!("{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n");
    for (k, v) in headers {
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    if let Some(b) = body {
        head.push_str(&format!("Content-Length: {}\r\n", b.len()));
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes()).await?;
    if let Some(b) = body {
        stream.write_all(b.as_bytes()).await?;
    }
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await?;
    let text = String::from_utf8(raw).context("phản hồi không phải UTF-8")?;
    let (head, body) = text
        .split_once("\r\n\r\n")
        .context("phản hồi thiếu dòng trống ngăn header và thân")?;
    let mut lines = head.split("\r\n");
    let status_line = lines.next().context("thiếu status line")?;
    let status: u16 = status_line
        .split(' ')
        .nth(1)
        .context("status line sai dạng")?
        .parse()?;
    let headers: Vec<(String, String)> = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string()))
        .collect();
    if headers
        .iter()
        .any(|(k, v)| k == "transfer-encoding" && v.contains("chunked"))
    {
        bail!("client tối thiểu không đọc chunked");
    }
    Ok(RawResponse {
        status,
        headers,
        body: body.to_string(),
    })
}

/// Bind `127.0.0.1:0`, đọc cổng thật rồi phục vụ `app` (có `ConnectInfo<SocketAddr>`).
/// Trả địa chỉ và task server; gọi `abort()` để dừng.
pub async fn serve(app: Router) -> Result<(SocketAddr, JoinHandle<()>)> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let task = tokio::spawn(async move {
        let service = app.into_make_service_with_connect_info::<SocketAddr>();
        if let Err(e) = axum::serve(listener, service).await {
            eprintln!("server dừng với lỗi: {e}");
        }
    });
    Ok((addr, task))
}
