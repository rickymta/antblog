//! Client HTTP/1.1 tối thiểu qua TCP thật (chỉ cho lab, để không thêm crate client).

use std::net::SocketAddr;

use axum::Router;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

/// Response đã tách.
#[derive(Debug)]
pub struct Reply {
    /// Mã trạng thái.
    pub status: u16,
    /// Header, tên viết thường.
    pub headers: Vec<(String, String)>,
    /// Body dạng chữ.
    pub body: String,
}

impl Reply {
    /// Giá trị header (tên viết thường).
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

/// Chạy app trên `127.0.0.1:0` và trả về cổng thật.
pub async fn spawn(app: Router) -> (SocketAddr, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });
    (addr, handle)
}

/// Gửi một request, đọc tới hết (`Connection: close`).
pub async fn send(
    addr: SocketAddr,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &str,
) -> Reply {
    let mut raw = format!("{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n");
    for (k, v) in headers {
        raw.push_str(&format!("{k}: {v}\r\n"));
    }
    raw.push_str(&format!("Content-Length: {}\r\n\r\n{body}", body.len()));
    let mut stream = TcpStream::connect(addr).await.expect("connect");
    stream.write_all(raw.as_bytes()).await.expect("write");
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await.expect("read");
    let text = String::from_utf8_lossy(&buf).into_owned();
    let (head, body) = text.split_once("\r\n\r\n").expect("head/body");
    let mut lines = head.lines();
    let status = lines
        .next()
        .and_then(|l| l.split(' ').nth(1))
        .and_then(|s| s.parse().ok())
        .expect("status");
    let headers = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string()))
        .collect();
    Reply {
        status,
        headers,
        body: body.to_string(),
    }
}
