//! Collector tạm của lab: một container Jaeger (OTLP vào, API truy vấn ra), cổng do Docker cấp, xoá khi xong.
//!
//! Chỉ dùng `docker` CLI và một HTTP client tối thiểu trên `tokio::net::TcpStream`,
//! nên không thêm crate nào. Chỉ đụng container có tiền tố `lab-66-collector-` do chính lab tạo.

use std::collections::BTreeMap;
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::checks::SpanView;

/// Image Jaeger v2 (all-in-one mặc định: OTLP receiver, bộ nhớ trong, API truy vấn).
pub const IMAGE: &str = "jaegertracing/jaeger:2.23.0";
const OTLP_HTTP_PORT: u16 = 4318;
const QUERY_PORT: u16 = 16686;

/// Container collector đang chạy; `Drop` xoá nó kể cả khi lab lỗi hoặc panic.
pub struct Collector {
    name: String,
    otlp_port: u16,
    query_port: u16,
}

impl Collector {
    /// Chạy container, chờ API truy vấn trả lời.
    ///
    /// # Errors
    /// Không có Docker, không kéo được image, hoặc container không sẵn sàng sau 90 giây.
    pub async fn start() -> Result<Collector> {
        let name = format!(
            "lab-66-collector-{}",
            &Uuid::new_v4().simple().to_string()[..8]
        );
        // `-p 127.0.0.1::PORT` để Docker tự chọn cổng trống trên loopback.
        docker(&[
            "run",
            "-d",
            "--name",
            &name,
            "-p",
            &format!("127.0.0.1::{OTLP_HTTP_PORT}"),
            "-p",
            &format!("127.0.0.1::{QUERY_PORT}"),
            IMAGE,
        ])
        .context("docker run (cần Docker và image đã kéo được)")?;
        // Từ đây container đã tồn tại: dựng struct ngay để Drop dọn nếu bước sau lỗi.
        let mut collector = Collector {
            name,
            otlp_port: 0,
            query_port: 0,
        };
        collector.otlp_port = mapped_port(&collector.name, OTLP_HTTP_PORT)?;
        collector.query_port = mapped_port(&collector.name, QUERY_PORT)?;
        collector.wait_ready(Duration::from_secs(90)).await?;
        Ok(collector)
    }

    /// Tên container (tiền tố `lab-66-collector-`).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// URL OTLP/HTTP cho traces, để đưa vào `Sink::Otlp`.
    pub fn traces_endpoint(&self) -> String {
        format!("http://127.0.0.1:{}/v1/traces", self.otlp_port)
    }

    // Thăm dò trạng thái (không đoán bằng sleep một lần): thử tới khi API trả 200 hoặc hết hạn.
    // Jaeger v2 bỏ `/api/services`; `/api/v3/services` là đường kiểm sống.
    async fn wait_ready(&self, limit: Duration) -> Result<()> {
        let start = Instant::now();
        loop {
            if let Ok((200, _)) = http_get(self.query_port, "/api/v3/services").await {
                return Ok(());
            }
            if start.elapsed() >= limit {
                bail!("collector {} chưa sẵn sàng sau {limit:?}", self.name);
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }

    /// Lấy trace từ API Jaeger. `None` nếu collector chưa có trace đó (HTTP 404).
    ///
    /// # Errors
    /// Không gọi được API hoặc JSON không đúng dạng.
    pub async fn fetch(&self, trace_id: &str) -> Result<Option<Vec<SpanView>>> {
        let (status, body) = http_get(self.query_port, &format!("/api/traces/{trace_id}")).await?;
        match status {
            404 => Ok(None),
            200 => Ok(Some(parse_trace(&body)?)),
            other => bail!("Jaeger trả HTTP {other}: {body}"),
        }
    }

    /// Chờ tới khi trace có ít nhất `min_spans` span (collector nhận span bất đồng bộ).
    ///
    /// # Errors
    /// Hết hạn mà chưa đủ span.
    pub async fn wait_for_trace(
        &self,
        trace_id: &str,
        min_spans: usize,
        limit: Duration,
    ) -> Result<Vec<SpanView>> {
        let start = Instant::now();
        loop {
            let seen = match self.fetch(trace_id).await? {
                Some(spans) if spans.len() >= min_spans => return Ok(spans),
                Some(spans) => spans.len(),
                None => 0,
            };
            if start.elapsed() >= limit {
                bail!("trace {trace_id}: sau {limit:?} mới thấy {seen}/{min_spans} span");
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }

    /// Xoá container. An toàn khi gọi lại.
    pub fn stop(&self) {
        let _ = Command::new("docker")
            .args(["rm", "-f", &self.name])
            .output();
    }
}

impl Drop for Collector {
    fn drop(&mut self) {
        self.stop();
    }
}

fn docker(args: &[&str]) -> Result<String> {
    let out = Command::new("docker").args(args).output()?;
    if !out.status.success() {
        bail!(
            "docker {} thất bại: {}",
            args.first().copied().unwrap_or(""),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

// `docker port <name> 4318/tcp` in mỗi dòng `127.0.0.1:49153` (có thể thêm dòng IPv6).
fn mapped_port(name: &str, container_port: u16) -> Result<u16> {
    let out = docker(&["port", name, &format!("{container_port}/tcp")])?;
    out.lines()
        .find_map(|line| line.rsplit(':').next()?.trim().parse::<u16>().ok())
        .with_context(|| format!("không đọc được cổng ánh xạ của {container_port}: {out:?}"))
}

/// GET HTTP/1.0 tối thiểu tới `127.0.0.1:port`; đọc tới EOF. HTTP/1.0 nên server không dùng chunked.
async fn http_get(port: u16, path: &str) -> Result<(u16, String)> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await?;
    stream
        .write_all(
            format!("GET {path} HTTP/1.0\r\nHost: 127.0.0.1\r\nAccept: application/json\r\n\r\n")
                .as_bytes(),
        )
        .await?;
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await?;
    let text = String::from_utf8_lossy(&raw);
    let (head, body) = text
        .split_once("\r\n\r\n")
        .context("phản hồi HTTP không có phần thân")?;
    let status = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .context("không đọc được mã trạng thái HTTP")?;
    Ok((status, body.to_owned()))
}

/// Đổi JSON của `GET /api/traces/<id>` sang danh sách `SpanView`.
fn parse_trace(body: &str) -> Result<Vec<SpanView>> {
    let json: Value = serde_json::from_str(body)?;
    let trace = json["data"]
        .get(0)
        .context("phản hồi Jaeger không có trace nào")?;
    let services: BTreeMap<&str, &str> = trace["processes"]
        .as_object()
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| Some((k.as_str(), v["serviceName"].as_str()?)))
                .collect()
        })
        .unwrap_or_default();
    let mut out = Vec::new();
    for span in trace["spans"].as_array().context("thiếu mảng spans")? {
        let text = |v: &Value| match v {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        let attrs: BTreeMap<String, String> = span["tags"]
            .as_array()
            .map(|tags| {
                tags.iter()
                    .filter_map(|t| Some((t["key"].as_str()?.to_owned(), text(&t["value"]))))
                    .collect()
            })
            .unwrap_or_default();
        let refs = span["references"].as_array().cloned().unwrap_or_default();
        let parent_id = refs
            .iter()
            .find(|r| r["refType"] == "CHILD_OF")
            .and_then(|r| r["spanID"].as_str())
            .map(str::to_owned);
        let links = refs
            .iter()
            .filter(|r| r["refType"] == "FOLLOWS_FROM")
            .filter_map(|r| r["traceID"].as_str().map(str::to_owned))
            .collect();
        let error = attrs.get("error").is_some_and(|v| v == "true")
            || attrs.get("otel.status_code").is_some_and(|v| v == "ERROR");
        out.push(SpanView {
            name: span["operationName"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            // Span INTERNAL không có tag `span.kind` trong Jaeger.
            kind: attrs
                .get("span.kind")
                .cloned()
                .unwrap_or_else(|| "internal".to_owned()),
            trace_id: span["traceID"].as_str().unwrap_or_default().to_owned(),
            span_id: span["spanID"].as_str().unwrap_or_default().to_owned(),
            parent_id,
            remote_parent: false,
            service: span["processID"]
                .as_str()
                .and_then(|p| services.get(p))
                .copied()
                .unwrap_or_default()
                .to_owned(),
            attrs,
            links,
            error,
        });
    }
    Ok(out)
}
