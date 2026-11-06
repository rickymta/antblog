//! Client HTTP/1.1 tối thiểu qua TCP thật, chỉ để `--verify` gọi server của chính lab.
//! Dùng `Connection: close` nên đọc tới EOF là hết response; không phải client dùng cho sản phẩm.

use std::error::Error;
use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Phản hồi đã tách: mã, `Content-Type`, body.
#[derive(Debug, Clone)]
pub struct Reply {
    /// Mã HTTP.
    pub status: u16,
    /// Giá trị header `Content-Type` nếu có.
    pub content_type: Option<String>,
    /// Body dạng chuỗi UTF-8.
    pub body: String,
}

impl Reply {
    /// Body như JSON.
    ///
    /// # Errors
    /// Body không phải JSON hợp lệ.
    pub fn json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.body)
    }
}

/// Gửi một request và đọc hết response.
///
/// # Errors
/// Lỗi kết nối hoặc response sai dạng.
pub async fn send(
    addr: SocketAddr,
    method: &str,
    path: &str,
    content_type: Option<&str>,
    body: &str,
) -> Result<Reply, Box<dyn Error>> {
    let mut stream = TcpStream::connect(addr).await?;
    let mut head = format!("{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n");
    if let Some(ct) = content_type {
        head.push_str(&format!("Content-Type: {ct}\r\n"));
    }
    head.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
    stream.write_all(head.as_bytes()).await?;
    stream.write_all(body.as_bytes()).await?;
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await?;
    parse(&raw)
}

fn parse(raw: &[u8]) -> Result<Reply, Box<dyn Error>> {
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or("response không có dòng trống kết thúc header")?;
    let head = std::str::from_utf8(&raw[..split])?;
    let mut lines = head.split("\r\n");
    let status_line = lines.next().ok_or("thiếu status line")?;
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .ok_or("status line sai dạng")?
        .parse()?;
    let mut content_type = None;
    let mut chunked = false;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        match name.trim().to_ascii_lowercase().as_str() {
            "content-type" => content_type = Some(value.trim().to_string()),
            "transfer-encoding" => chunked = value.trim().eq_ignore_ascii_case("chunked"),
            _ => {}
        }
    }
    let body_bytes = &raw[split + 4..];
    let body = if chunked {
        String::from_utf8(dechunk(body_bytes)?)?
    } else {
        String::from_utf8_lossy(body_bytes).into_owned()
    };
    Ok(Reply {
        status,
        content_type,
        body,
    })
}

// Giải mã `Transfer-Encoding: chunked`: mỗi khối là `<hex>\r\n<dữ liệu>\r\n`, kết thúc bằng khối 0.
fn dechunk(mut data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut out = Vec::new();
    loop {
        let eol = data
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or("khối chunked thiếu CRLF")?;
        let size = usize::from_str_radix(std::str::from_utf8(&data[..eol])?.trim(), 16)?;
        data = &data[eol + 2..];
        if size == 0 {
            return Ok(out);
        }
        out.extend_from_slice(data.get(..size).ok_or("khối chunked bị cụt")?);
        data = data.get(size + 2..).ok_or("khối chunked thiếu CRLF cuối")?;
    }
}
