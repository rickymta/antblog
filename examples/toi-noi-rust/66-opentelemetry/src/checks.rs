//! Kịch bản kiểm chứng của lab. Mỗi hàm in điều nó thấy và trả `Err` nếu điều kiện của bài không đúng.

use std::collections::BTreeMap;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context as _, Result, bail, ensure};
use opentelemetry::Context;
use opentelemetry::KeyValue;
use opentelemetry::global;
use opentelemetry::metrics::MeterProvider as _;
use opentelemetry::propagation::Extractor as _;
use opentelemetry::trace::{
    Span as _, SpanContext, SpanId, SpanKind, Status as SpanStatus, TraceContextExt, TraceFlags,
    TraceId, TraceState, Tracer as _, TracerProvider as _,
};
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::metrics::{InMemoryMetricExporter, PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{
    BatchConfigBuilder, BatchSpanProcessor, InMemorySpanExporter, Sampler, SdkTracerProvider,
    SpanData, SpanExporter,
};
use rdkafka::message::Headers;
use tonic::Code;
use tracing::Instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use uuid::Uuid;

use crate::events::{LedgerEntryPosted, PostingDto, SCHEMA_VERSION};
use crate::metrics::{self, Series};
use crate::pb::{PostEntryRequest, Posting};
use crate::propagation::{MetadataExtractor, current_kafka_headers, extract_from_metadata};
use crate::stack::Stack;
use crate::telemetry::Telemetry;
use crate::upstream::{call_with_raw_traceparent, capture_request, capture_trip};

/// Hạn chờ cho việc «chắc chắn sẽ xảy ra» (không phải để đoán thời gian).
pub const WAIT: Duration = Duration::from_secs(30);

/// Một span đã chuẩn hoá, từ exporter in-memory hoặc từ API Jaeger, để dùng chung một bộ kiểm.
#[derive(Debug, Clone)]
pub struct SpanView {
    pub name: String,
    /// `client`, `server`, `producer`, `consumer` hoặc `internal`.
    pub kind: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_id: Option<String>,
    /// Cha ở tiến trình khác (chỉ biết với dữ liệu in-memory).
    pub remote_parent: bool,
    pub service: String,
    pub attrs: BTreeMap<String, String>,
    /// `trace_id` của các span được link tới.
    pub links: Vec<String>,
    pub error: bool,
}

impl SpanView {
    fn from_data(d: &SpanData, service: &str) -> SpanView {
        let kind = match d.span_kind {
            SpanKind::Client => "client",
            SpanKind::Server => "server",
            SpanKind::Producer => "producer",
            SpanKind::Consumer => "consumer",
            SpanKind::Internal => "internal",
        };
        SpanView {
            name: d.name.to_string(),
            kind: kind.to_owned(),
            trace_id: d.span_context.trace_id().to_string(),
            span_id: d.span_context.span_id().to_string(),
            parent_id: (d.parent_span_id != SpanId::INVALID).then(|| d.parent_span_id.to_string()),
            remote_parent: d.parent_span_is_remote,
            service: service.to_owned(),
            attrs: d
                .attributes
                .iter()
                .map(|kv| (kv.key.to_string(), kv.value.to_string()))
                .collect(),
            links: d
                .links
                .iter()
                .map(|l| l.span_context.trace_id().to_string())
                .collect(),
            error: matches!(d.status, SpanStatus::Error { .. }),
        }
    }
}

/// Mọi span của một trace đang có trong exporter in-memory.
pub fn spans_of(t: &Telemetry, trace_id: &str) -> Vec<SpanView> {
    let Some(exporter) = &t.spans else {
        return Vec::new();
    };
    exporter
        .get_finished_spans()
        .unwrap_or_default()
        .iter()
        .map(|d| SpanView::from_data(d, crate::telemetry::SERVICE_NAME))
        .filter(|s| s.trace_id == trace_id)
        .collect()
}

/// Chờ tới khi trace có ít nhất `min` span (span đóng bất đồng bộ sau khi RPC trả về).
pub async fn wait_spans(
    t: &Telemetry,
    trace_id: &str,
    min: usize,
    limit: Duration,
) -> Result<Vec<SpanView>> {
    let start = Instant::now();
    loop {
        let spans = spans_of(t, trace_id);
        if spans.len() >= min {
            return Ok(spans);
        }
        if start.elapsed() >= limit {
            bail!(
                "trace {trace_id}: sau {limit:?} mới thấy {}/{min} span",
                spans.len()
            );
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

/// Cây cha-con kỳ vọng của một yêu cầu thành công: (con, cha), mỗi nút là (tên, kind).
/// Một nút của cây: (tên span, kind).
type Node = (&'static str, &'static str);

const EDGES: [(Node, Node); 6] = [
    (
        ("ledger.v1.Ledger/PostEntry", "client"),
        ("payment.capture-trip", "internal"),
    ),
    (
        ("ledger.v1.Ledger/PostEntry", "server"),
        ("ledger.v1.Ledger/PostEntry", "client"),
    ),
    (
        ("ledger.entry.post", "internal"),
        ("ledger.v1.Ledger/PostEntry", "server"),
    ),
    (
        ("send ledger.entry-posted", "producer"),
        ("ledger.v1.Ledger/PostEntry", "server"),
    ),
    (
        ("process ledger.entry-posted", "consumer"),
        ("send ledger.entry-posted", "producer"),
    ),
    (
        ("projection.apply", "internal"),
        ("process ledger.entry-posted", "consumer"),
    ),
];

fn find<'a>(spans: &'a [SpanView], node: (&str, &str)) -> Result<&'a SpanView> {
    let found: Vec<_> = spans
        .iter()
        .filter(|s| s.name == node.0 && s.kind == node.1)
        .collect();
    ensure!(
        found.len() == 1,
        "cần đúng 1 span {:?} [{}], thấy {}",
        node.0,
        node.1,
        found.len()
    );
    Ok(found[0])
}

/// Kiểm cây: cùng một `trace_id`, đủ bảy span, đúng quan hệ cha-con, gốc không có cha.
///
/// # Errors
/// Thiếu span, sai cha, hoặc khác `trace_id`.
pub fn assert_tree(spans: &[SpanView]) -> Result<()> {
    ensure!(spans.len() == 7, "cần 7 span, thấy {}", spans.len());
    let trace = &spans[0].trace_id;
    ensure!(
        spans.iter().all(|s| &s.trace_id == trace),
        "span khác trace_id"
    );
    for (child, parent) in EDGES {
        let (c, p) = (find(spans, child)?, find(spans, parent)?);
        ensure!(
            c.parent_id.as_deref() == Some(p.span_id.as_str()),
            "{} [{}] phải là con của {} [{}], cha thật là {:?}",
            c.name,
            c.kind,
            p.name,
            p.kind,
            c.parent_id
        );
    }
    let root = find(spans, ("payment.capture-trip", "internal"))?;
    ensure!(root.parent_id.is_none(), "span gốc không được có cha");
    Ok(())
}

/// Vẽ cây span dạng chữ để in ra.
pub fn render_tree(spans: &[SpanView]) -> String {
    fn walk(spans: &[SpanView], parent: Option<&str>, depth: usize, out: &mut String) {
        let mut children: Vec<_> = spans
            .iter()
            .filter(|s| s.parent_id.as_deref() == parent)
            .collect();
        children.sort_by(|a, b| a.name.cmp(&b.name));
        for s in children {
            let svc = if s.service.is_empty() {
                String::new()
            } else {
                format!("  ({})", s.service)
            };
            out.push_str(&format!(
                "{}{} [{}]{svc}\n",
                "  ".repeat(depth),
                s.name,
                s.kind
            ));
            walk(spans, Some(&s.span_id), depth + 1, out);
        }
    }
    let ids: Vec<&str> = spans.iter().map(|s| s.span_id.as_str()).collect();
    let mut out = String::new();
    // Gốc: không có cha, hoặc cha không nằm trong tập (ví dụ cha là span của upstream chưa thu được).
    let mut roots: Vec<_> = spans
        .iter()
        .filter(|s| s.parent_id.as_deref().is_none_or(|p| !ids.contains(&p)))
        .collect();
    roots.sort_by(|a, b| a.name.cmp(&b.name));
    for r in roots {
        let svc = if r.service.is_empty() {
            String::new()
        } else {
            format!("  ({})", r.service)
        };
        out.push_str(&format!("{} [{}]{svc}\n", r.name, r.kind));
        walk(spans, Some(&r.span_id), 1, &mut out);
    }
    out
}

/// Không span nào chứa mã tài khoản, số tiền hay số định danh người dùng (như bài 50).
///
/// # Errors
/// Một attribute hoặc tên span chứa dữ liệu nhạy cảm.
pub fn assert_no_sensitive(spans: &[SpanView]) -> Result<()> {
    const FORBIDDEN: [&str; 9] = [
        "CU-001",
        "DR-001",
        "customer:",
        "driver:",
        "psp:",
        "25500",
        "25.500",
        "20400",
        "5100",
    ];
    // Giá trị kỹ thuật do lab sinh (uuid của topic và group, cổng ngẫu nhiên): số hex hoặc số cổng có thể
    // tình cờ chứa `5100`, không phải dữ liệu nghiệp vụ, nên không quét để test không lỗi ngẫu nhiên.
    const TECHNICAL: [&str; 3] = [
        "messaging.destination.name",
        "messaging.consumer.group.name",
        "server.port",
    ];
    for s in spans {
        let attrs = s
            .attrs
            .iter()
            .filter(|(key, _)| !TECHNICAL.contains(&key.as_str()))
            .map(|(_, value)| value);
        for text in std::iter::once(&s.name).chain(attrs) {
            for bad in FORBIDDEN {
                ensure!(
                    !text.contains(bad),
                    "span {:?} chứa {bad:?}: {text:?}",
                    s.name
                );
            }
        }
    }
    Ok(())
}

/// Kết quả một yêu cầu capture thành công đi hết đường.
pub struct Captured {
    pub trace_id: String,
    pub entry_id: String,
    pub spans: Vec<SpanView>,
}

/// Client (giả lập C#) gọi `PostEntry`, consumer đọc thông điệp, rồi chờ đủ 7 span.
///
/// # Errors
/// RPC lỗi, consumer hết giờ, hoặc span chưa đủ.
pub async fn run_capture(t: &Telemetry, stack: &Stack) -> Result<Captured> {
    let call = capture_trip(stack.addr, capture_request()).await?;
    let reply = call
        .result
        .map_err(|s| anyhow::anyhow!("PostEntry lỗi: {s}"))?;
    ensure!(!reply.replayed, "lần đầu không được là replay");
    let consumed = stack.consumer.process_next(WAIT).await?;
    ensure!(
        consumed == reply.entry_id,
        "consumer áp dụng {consumed}, server ghi {}",
        reply.entry_id
    );
    let spans = wait_spans(t, &call.trace_id, 7, WAIT).await?;
    Ok(Captured {
        trace_id: call.trace_id,
        entry_id: reply.entry_id,
        spans,
    })
}

fn split_request() -> PostEntryRequest {
    let posting = |account: &str, amount_dong: i64| Posting {
        account: account.to_owned(),
        amount_dong,
    };
    PostEntryRequest {
        idempotency_key: "trip:TN-0001:split".to_owned(),
        postings: vec![
            posting("psp:vixanh:clearing", -25_500),
            posting("platform:revenue", 5_100),
            posting("driver:DR-001", 20_400),
        ],
    }
}

fn refund_request() -> PostEntryRequest {
    let posting = |account: &str, amount_dong: i64| Posting {
        account: account.to_owned(),
        amount_dong,
    };
    PostEntryRequest {
        idempotency_key: "trip:TN-0001:refund:1".to_owned(),
        postings: vec![
            posting("psp:vixanh:clearing", -5_000),
            posting("customer:CU-001", 5_000),
        ],
    }
}

fn unbalanced_request() -> PostEntryRequest {
    let mut req = capture_request();
    req.idempotency_key = "trip:TN-0003:capture".to_owned();
    req.postings[1].amount_dong = 25_000;
    req
}

/// Các dòng log JSON có `trace_id` đã cho.
pub fn logs_of(t: &Telemetry, trace_id: &str) -> Vec<serde_json::Value> {
    t.logs
        .lines()
        .iter()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .filter(|v| v["trace_id"] == trace_id)
        .collect()
}

/// Kịch bản 1: capture đi hết đường, kiểm cây, cha ở xa, log tương quan và dữ liệu nhạy cảm.
///
/// # Errors
/// Một điều kiện không đúng.
pub fn check_capture(t: &Telemetry, c: &Captured) -> Result<()> {
    assert_tree(&c.spans)?;
    assert_no_sensitive(&c.spans)?;
    let server = find(&c.spans, ("ledger.v1.Ledger/PostEntry", "server"))?;
    let process = find(&c.spans, ("process ledger.entry-posted", "consumer"))?;
    ensure!(
        server.remote_parent,
        "span server phải có cha ở xa (từ metadata)"
    );
    ensure!(
        process.remote_parent,
        "span consumer phải có cha ở xa (từ header Kafka)"
    );
    ensure!(
        server.attrs.get("rpc.status_code").map(String::as_str) == Some("OK"),
        "rpc.status_code của server"
    );
    // Log: cùng một trace_id ở server (trước Kafka) và ở consumer (sau Kafka).
    let logs = logs_of(t, &c.trace_id);
    let messages: Vec<&str> = logs.iter().filter_map(|l| l["message"].as_str()).collect();
    ensure!(
        messages.contains(&"bút toán đã ghi") && messages.contains(&"đã áp dụng vào read model"),
        "log phải có cả phía server lẫn consumer với cùng trace_id, thấy {messages:?}"
    );
    println!("trace_id        : {}", c.trace_id);
    println!("cây span ({} span, một trace):", c.spans.len());
    for line in render_tree(&c.spans).lines() {
        println!("  {line}");
    }
    println!("log cùng trace_id: {messages:?}");
    Ok(())
}

/// Kịch bản 2: header `traceparent` viết tay (ví dụ trong đặc tả W3C) được server nhận làm cha.
///
/// # Errors
/// Server không nhận đúng cha hoặc consumer không nối vào producer.
pub async fn check_raw_traceparent(t: &Telemetry, stack: &Stack) -> Result<()> {
    const TRACE: &str = "4bf92f3577b34da6a3ce929d0e0e4736";
    const PARENT: &str = "00f067aa0ba902b7";
    let header = format!("00-{TRACE}-{PARENT}-01");
    let reply = call_with_raw_traceparent(stack.addr, Some(&header), split_request())
        .await?
        .map_err(|s| anyhow::anyhow!("PostEntry lỗi: {s}"))?;
    stack.consumer.process_next(WAIT).await?;
    let spans = wait_spans(t, TRACE, 5, WAIT).await?;
    let server = find(&spans, ("ledger.v1.Ledger/PostEntry", "server"))?;
    ensure!(
        server.parent_id.as_deref() == Some(PARENT),
        "cha của span server là {:?}",
        server.parent_id
    );
    ensure!(server.remote_parent, "cha phải được đánh dấu ở xa");
    let send = find(&spans, ("send ledger.entry-posted", "producer"))?;
    let process = find(&spans, ("process ledger.entry-posted", "consumer"))?;
    ensure!(
        process.parent_id.as_deref() == Some(send.span_id.as_str()),
        "consumer phải là con của producer"
    );
    println!(
        "header viết tay {header} -> server là con của {PARENT}, bút toán {}",
        reply.entry_id
    );
    Ok(())
}

/// Kịch bản 2b: client KHÔNG gửi `traceparent`. Server mở trace mới; từ đó Kafka vẫn nối tiếp,
/// nhưng trace bị đứt làm hai: phía upstream và phía sổ cái không còn chung `trace_id`.
///
/// # Errors
/// Span server có cha, hoặc nghiệp vụ không chạy.
pub async fn check_missing_header(t: &Telemetry, stack: &Stack) -> Result<()> {
    let mut req = capture_request();
    req.idempotency_key = "trip:TN-0002:capture".to_owned();
    let reply = call_with_raw_traceparent(stack.addr, None, req)
        .await?
        .map_err(|s| anyhow::anyhow!("PostEntry lỗi: {s}"))?;
    stack.consumer.process_next(WAIT).await?;
    let exporter = t.spans.as_ref().context("cần exporter in-memory")?;
    let roots: Vec<SpanView> = exporter
        .get_finished_spans()
        .unwrap_or_default()
        .iter()
        .map(|d| SpanView::from_data(d, crate::telemetry::SERVICE_NAME))
        .filter(|s| {
            s.name == "ledger.v1.Ledger/PostEntry" && s.kind == "server" && s.parent_id.is_none()
        })
        .collect();
    ensure!(
        roots.len() == 1,
        "cần đúng 1 span server không cha, thấy {}",
        roots.len()
    );
    let spans = wait_spans(t, &roots[0].trace_id, 5, WAIT).await?;
    ensure!(
        spans.iter().all(|s| s.kind != "client"),
        "không có span client nào trong trace mới"
    );
    println!(
        "không có traceparent: span server là gốc của trace mới {} ({} span, bút toán {}), Kafka vẫn nối tiếp",
        roots[0].trace_id,
        spans.len(),
        reply.entry_id
    );
    Ok(())
}

/// Kịch bản 3: cờ `sampled=00` của upstream được tôn trọng suốt đường (server, ledger, producer, consumer).
///
/// # Errors
/// Có span nào được xuất cho trace bị upstream loại, hoặc nghiệp vụ không chạy.
pub async fn check_unsampled(t: &Telemetry, stack: &Stack) -> Result<()> {
    let trace = Uuid::new_v4().simple().to_string();
    let header = format!("00-{trace}-1122334455667788-00");
    let reply = call_with_raw_traceparent(stack.addr, Some(&header), refund_request())
        .await?
        .map_err(|s| anyhow::anyhow!("PostEntry lỗi: {s}"))?;
    let applied = stack.consumer.process_next(WAIT).await?;
    // Nghiệp vụ vẫn chạy đủ: bút toán ghi và consumer áp dụng. Chỉ telemetry bị bỏ.
    ensure!(
        applied == reply.entry_id,
        "nghiệp vụ phải chạy dù trace không được lấy mẫu"
    );
    ensure!(
        spans_of(t, &trace).is_empty(),
        "trace sampled=00 không được xuất span nào"
    );
    println!("sampled=00: bút toán {applied} đã ghi và áp dụng, 0 span được xuất");
    Ok(())
}

/// Kịch bản 4: lỗi nghiệp vụ. Span ghi lỗi bằng mã cố định, không có số tiền; không có thông điệp Kafka.
///
/// # Errors
/// Mã gRPC, trạng thái span hoặc metric lỗi không đúng.
pub async fn check_error_path(t: &Telemetry, stack: &Stack) -> Result<()> {
    let call = capture_trip(stack.addr, unbalanced_request()).await?;
    let status = call.result.err().context("bút toán lệch phải bị từ chối")?;
    ensure!(
        status.code() == Code::InvalidArgument,
        "mã gRPC {:?}",
        status.code()
    );
    let spans = wait_spans(t, &call.trace_id, 4, WAIT).await?;
    let post = find(&spans, ("ledger.entry.post", "internal"))?;
    ensure!(post.error, "span ledger.entry.post phải ở trạng thái lỗi");
    ensure!(
        post.attrs.get("error.type").map(String::as_str) == Some("unbalanced"),
        "error.type"
    );
    ensure!(
        spans.iter().all(|s| s.kind != "producer"),
        "bút toán bị từ chối thì không phát sự kiện"
    );
    assert_no_sensitive(&spans)?;
    println!(
        "bút toán lệch: {:?}, span {} error.type=unbalanced, không có span producer",
        status.code(),
        post.name
    );
    Ok(())
}

fn synthetic_event(entry_id: &str) -> LedgerEntryPosted {
    LedgerEntryPosted {
        schema_version: SCHEMA_VERSION,
        entry_id: entry_id.to_owned(),
        idempotency_key: format!("synthetic:{entry_id}"),
        postings: vec![PostingDto {
            account: "psp:vixanh:clearing".to_owned(),
            amount_dong: 0,
        }],
        posted_at: jiff::Timestamp::now(),
    }
}

/// Kịch bản 5: lô. Bốn thông điệp từ bốn trace khác nhau; một tiêu thụ riêng (lag 3), ba tiêu thụ theo lô (link).
///
/// # Errors
/// Lag sai, span lô có cha, hoặc link không khớp producer.
pub async fn check_batch_and_lag(t: &Telemetry, stack: &Stack) -> Result<()> {
    let publisher = stack.new_publisher()?;
    let mut producers = Vec::new(); // (trace_id, span_id của span producer)
    for n in 1..=4 {
        let root = tracing::info_span!(parent: None, "upstream.job");
        let event = synthetic_event(&format!("JE-90000{n}"));
        publisher.publish(&event).instrument(root.clone()).await?;
        let trace = root.context().span().span_context().trace_id().to_string();
        producers.push(trace);
    }
    // Một thông điệp xử lý riêng: lag = 4 đã gửi - 1 đã đọc = 3.
    stack.consumer.process_next(WAIT).await?;
    t.flush()?;
    let lag = metrics::find(&metrics::latest(&t.metrics), "ledger.consumer.lag")
        .and_then(|s| s.points.first().map(|p| p.value))
        .context("thiếu metric ledger.consumer.lag")?;
    ensure!(
        (lag - 3.0).abs() < f64::EPSILON,
        "lag phải là 3, thấy {lag}"
    );

    // Ba thông điệp còn lại: một span gốc, ba link.
    let batch_ids = stack.consumer.process_batch(3, WAIT).await?;
    ensure!(
        batch_ids == ["JE-900002", "JE-900003", "JE-900004"],
        "lô: {batch_ids:?}"
    );
    t.flush()?;
    let lag_after = metrics::find(&metrics::latest(&t.metrics), "ledger.consumer.lag")
        .and_then(|s| s.points.first().map(|p| p.value))
        .context("thiếu metric ledger.consumer.lag")?;
    ensure!(lag_after == 0.0, "lag sau lô phải là 0, thấy {lag_after}");

    let batch = batch_span(t, &producers[1..])?;
    ensure!(batch.parent_id.is_none(), "span lô phải là gốc");
    ensure!(
        batch.name == "process ledger.entry-posted" && batch.kind == "consumer",
        "tên span lô theo `{{operation}} {{destination}}`, thấy {:?}",
        batch.name
    );
    ensure!(
        batch.attrs.contains_key("messaging.consumer.group.name"),
        "span lô phải có group"
    );
    ensure!(
        !producers.contains(&batch.trace_id),
        "span lô nằm trong trace riêng, không thuộc trace nào của producer"
    );
    let mut linked = batch.links.clone();
    linked.sort();
    let mut expected: Vec<String> = producers[1..].to_vec();
    expected.sort();
    ensure!(
        linked == expected,
        "link {linked:?} khác producer {expected:?}"
    );
    // Mỗi trace của producer dừng ở span producer: không có span consumer là con của nó.
    for trace in &producers[1..] {
        ensure!(
            spans_of(t, trace).iter().all(|s| s.kind != "consumer"),
            "chế độ link không tạo span con trong trace của producer"
        );
    }
    println!("lag sau 1/4 thông điệp: {lag}; sau cả lô: {lag_after}");
    println!(
        "span lô: gốc của trace riêng, {} link tới {} trace producer",
        batch.links.len(),
        expected.len()
    );
    Ok(())
}

fn batch_span(t: &Telemetry, _traces: &[String]) -> Result<SpanView> {
    let exporter = t.spans.as_ref().context("cần exporter in-memory")?;
    let all: Vec<SpanView> = exporter
        .get_finished_spans()
        .unwrap_or_default()
        .iter()
        .map(|d| SpanView::from_data(d, crate::telemetry::SERVICE_NAME))
        .filter(|s| s.attrs.contains_key("messaging.batch.message_count"))
        .collect();
    ensure!(all.len() == 1, "cần đúng 1 span lô, thấy {}", all.len());
    Ok(all[0].clone())
}

/// Metric cuối ca: số mẫu, attribute có giới hạn, không có account hay trip.
///
/// # Errors
/// Số mẫu hoặc tập attribute không đúng.
pub fn check_metrics(t: &Telemetry) -> Result<()> {
    t.flush()?;
    let series = metrics::latest(&t.metrics);
    let duration = metrics::find(&series, "ledger.entry.post.duration")
        .context("thiếu histogram ghi bút toán")?;
    ensure!(duration.unit == "s", "đơn vị phải là giây");
    let count = |outcome: &str| {
        duration
            .points
            .iter()
            .find(|p| p.attributes.get("ledger.outcome").map(String::as_str) == Some(outcome))
            .map_or(0.0, |p| p.value)
    };
    ensure!(
        count("posted") == 4.0 && count("rejected") == 1.0,
        "số mẫu posted/rejected"
    );
    let errors = metrics::find(&series, "ledger.entry.post.errors").context("thiếu counter lỗi")?;
    ensure!(
        errors.points.len() == 1 && errors.points[0].value == 1.0,
        "counter lỗi"
    );
    ensure!(
        errors.points[0]
            .attributes
            .get("error.type")
            .map(String::as_str)
            == Some("unbalanced"),
        "error.type của counter"
    );
    let process = metrics::find(&series, "messaging.process.duration")
        .context("thiếu messaging.process.duration")?;
    let processed: f64 = process.points.iter().map(|p| p.value).sum();
    // 5 thông điệp xử lý riêng và 1 lô.
    ensure!(
        processed == 6.0,
        "số thao tác process phải là 6, thấy {processed}"
    );
    for s in &series {
        for p in &s.points {
            for key in p.attributes.keys() {
                ensure!(
                    !key.contains("account") && !key.contains("trip") && !key.contains("entry_id"),
                    "metric {} có attribute không giới hạn: {key}",
                    s.name
                );
            }
        }
    }
    println!(
        "metric: {} (s) posted=4 rejected=1; lỗi unbalanced=1; thao tác process={processed} (5 riêng, 1 lô); mọi attribute có giới hạn",
        duration.name
    );
    Ok(())
}

/// Chạy trọn bộ kịch bản cần Kafka trên một `Stack`. Dùng cho `--verify` và `cargo test`.
///
/// # Errors
/// Kịch bản nào lỗi cũng dừng ngay và trả lỗi đó (stack vẫn được dọn bởi người gọi).
pub async fn run_stack_suite(t: &Telemetry, stack: &Stack) -> Result<()> {
    println!("== 1. Một trace_id từ client qua gRPC và Kafka tới consumer ==");
    let captured = run_capture(t, stack).await?;
    check_capture(t, &captured)?;
    println!("\n== 2. traceparent viết tay theo ví dụ W3C ==");
    check_raw_traceparent(t, stack).await?;
    println!("\n== 2b. Không có traceparent: trace bị đứt ==");
    check_missing_header(t, stack).await?;
    println!("\n== 3. Tôn trọng cờ sampled=00 của upstream ==");
    check_unsampled(t, stack).await?;
    println!("\n== 4. Lỗi nghiệp vụ trong span và metric ==");
    check_error_path(t, stack).await?;
    println!("\n== 5. Lô: span gốc có link, và consumer lag ==");
    check_batch_and_lag(t, stack).await?;
    println!("\n== 6. Metric ==");
    check_metrics(t)?;
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Các kịch bản không cần hạ tầng.
// ---------------------------------------------------------------------------------------------

/// Định dạng W3C: `traceparent` đúng dạng, `tracestate` đi nguyên vẹn, header hỏng thì bỏ.
///
/// # Errors
/// Một điều kiện không đúng.
pub fn check_w3c_format() -> Result<()> {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let mut md = tonic::metadata::MetadataMap::new();
    md.insert(
        "traceparent",
        "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01".parse()?,
    );
    md.insert("tracestate", "vendor=abc".parse()?);
    let cx = extract_from_metadata(&md);
    let sc = cx.span().span_context().clone();
    ensure!(sc.is_valid() && sc.is_remote(), "context hợp lệ và ở xa");
    ensure!(
        sc.trace_id().to_string() == "4bf92f3577b34da6a3ce929d0e0e4736",
        "trace_id"
    );
    ensure!(sc.span_id().to_string() == "00f067aa0ba902b7", "span_id");
    ensure!(sc.is_sampled(), "cờ sampled=01");
    ensure!(
        sc.trace_state().get("vendor") == Some("abc"),
        "tracestate phải còn nguyên"
    );
    // Ghi lại: cùng trace_id, tracestate giữ nguyên.
    let mut out = tonic::metadata::MetadataMap::new();
    global::get_text_map_propagator(|p| {
        p.inject_context(&cx, &mut crate::propagation::MetadataInjector(&mut out));
    });
    let written = out
        .get("traceparent")
        .and_then(|v| v.to_str().ok())
        .context("thiếu traceparent")?;
    let parts: Vec<&str> = written.split('-').collect();
    ensure!(
        parts.len() == 4
            && parts[0] == "00"
            && parts[1].len() == 32
            && parts[2].len() == 16
            && parts[3].len() == 2,
        "dạng {written}"
    );
    ensure!(
        out.get("tracestate").and_then(|v| v.to_str().ok()) == Some("vendor=abc"),
        "tracestate ghi lại"
    );
    // Hỏng: không panic, ra context rỗng.
    for bad in [
        "garbage",
        "00-00000000000000000000000000000000-00f067aa0ba902b7-01",
        "ff-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
    ] {
        let mut md = tonic::metadata::MetadataMap::new();
        md.insert("traceparent", bad.parse()?);
        let cx = extract_from_metadata(&md);
        ensure!(
            !cx.span().span_context().is_valid(),
            "header hỏng {bad:?} phải bị bỏ"
        );
    }
    // Extractor đọc khoá không phân biệt hoa thường theo cách của HTTP/2: khoá luôn chữ thường.
    ensure!(
        MetadataExtractor(&md).keys().contains(&"traceparent"),
        "keys"
    );
    println!("W3C: traceparent {written}; tracestate giữ nguyên; header hỏng bị bỏ");
    Ok(())
}

/// Cờ `sampled=00` đi ra header Kafka (consumer sau đó cũng không lấy mẫu).
///
/// # Errors
/// Cờ hoặc `trace_id` không được giữ.
pub fn check_unsampled_header(t: &Telemetry) -> Result<()> {
    let _ = t;
    let trace = TraceId::from_hex("0af7651916cd43dd8448eb211c80319c")?;
    let remote = SpanContext::new(
        trace,
        SpanId::from_hex("b7ad6b7169203331")?,
        TraceFlags::default(),
        true,
        TraceState::default(),
    );
    let cx = Context::new().with_remote_span_context(remote);
    let span = tracing::info_span!("unsampled.demo");
    span.set_parent(cx)
        .map_err(|e| anyhow::anyhow!("set_parent: {e}"))?;
    let headers = span.in_scope(current_kafka_headers);
    let value = (0..headers.count())
        .filter_map(|i| headers.try_get(i))
        .find(|h| h.key == "traceparent")
        .and_then(|h| h.value)
        .and_then(|v| std::str::from_utf8(v).ok())
        .context("thiếu header traceparent")?;
    ensure!(
        value.starts_with("00-0af7651916cd43dd8448eb211c80319c-") && value.ends_with("-00"),
        "header {value}"
    );
    println!("header Kafka khi upstream không lấy mẫu: {value}");
    Ok(())
}

/// Sampler: gốc theo tỷ lệ, con theo cờ của cha.
///
/// # Errors
/// Một quyết định lấy mẫu sai.
pub fn check_sampling_decisions() -> Result<()> {
    let remote = |flags: TraceFlags| {
        let sc = SpanContext::new(
            TraceId::from_hex("4bf92f3577b34da6a3ce929d0e0e4736").expect("hex hợp lệ"),
            SpanId::from_hex("00f067aa0ba902b7").expect("hex hợp lệ"),
            flags,
            true,
            TraceState::default(),
        );
        Context::new().with_remote_span_context(sc)
    };
    let exported = |ratio: f64, cx: &Context| -> usize {
        let exporter = InMemorySpanExporter::default();
        let provider = SdkTracerProvider::builder()
            .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
                ratio,
            ))))
            .with_simple_exporter(exporter.clone())
            .build();
        let tracer = provider.tracer("sampling-demo");
        tracer.start_with_context("op", cx).end();
        exporter.get_finished_spans().map_or(0, |s| s.len())
    };
    ensure!(
        exported(0.0, &Context::new()) == 0,
        "gốc với tỷ lệ 0 phải bị bỏ"
    );
    ensure!(
        exported(1.0, &Context::new()) == 1,
        "gốc với tỷ lệ 1 phải được giữ"
    );
    ensure!(
        exported(0.0, &remote(TraceFlags::SAMPLED)) == 1,
        "cha sampled=01 thì con được giữ dù tỷ lệ gốc là 0"
    );
    ensure!(
        exported(1.0, &remote(TraceFlags::default())) == 0,
        "cha sampled=00 thì con bị bỏ dù tỷ lệ gốc là 1"
    );
    println!(
        "sampler ParentBased(TraceIdRatio): gốc 0.0 -> bỏ, gốc 1.0 -> giữ, cha 01 -> giữ, cha 00 -> bỏ"
    );
    Ok(())
}

/// Chi phí cardinality của một label vô hạn (`account_id`) so với label có giới hạn.
#[derive(Debug)]
pub struct CardinalityReport {
    pub accounts: usize,
    pub bounded_series: usize,
    pub unbounded_series: usize,
    pub buckets_per_series: usize,
    pub overflow_series: usize,
}

impl CardinalityReport {
    /// Số giá trị (count, sum, min, max và từng bucket) SDK phải giữ cho một metric.
    pub fn numbers(&self, series: usize) -> usize {
        series * (self.buckets_per_series + 4)
    }
}

/// Ghi `accounts` mẫu vào hai histogram cùng tên cấu trúc: một chỉ có `ledger.outcome`,
/// một thêm `account_id`; rồi đếm số chuỗi thời gian (series) mỗi bên.
///
/// # Errors
/// Không thu được metric.
pub fn cardinality_cost(accounts: usize) -> Result<CardinalityReport> {
    let exporter = InMemoryMetricExporter::default();
    let reader = PeriodicReader::builder(exporter.clone())
        .with_interval(Duration::from_secs(3600))
        .build();
    let provider = SdkMeterProvider::builder().with_reader(reader).build();
    let meter = provider.meter("cardinality-demo");
    let bounds = vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0];
    let bounded = meter
        .f64_histogram("demo.bounded.duration")
        .with_unit("s")
        .with_boundaries(bounds.clone())
        .build();
    let unbounded = meter
        .f64_histogram("demo.unbounded.duration")
        .with_unit("s")
        .with_boundaries(bounds)
        .build();
    for i in 0..accounts {
        bounded.record(0.004, &[KeyValue::new("ledger.outcome", "posted")]);
        unbounded.record(
            0.004,
            &[
                KeyValue::new("ledger.outcome", "posted"),
                KeyValue::new("account_id", format!("customer:CU-{i:06}")),
            ],
        );
    }
    provider
        .force_flush()
        .map_err(|e| anyhow::anyhow!("flush: {e}"))?;
    let series: Vec<Series> = metrics::latest(&exporter);
    let bounded =
        metrics::find(&series, "demo.bounded.duration").context("thiếu metric bounded")?;
    let unbounded =
        metrics::find(&series, "demo.unbounded.duration").context("thiếu metric unbounded")?;
    let overflow_series = unbounded
        .points
        .iter()
        .filter(|p| p.attributes.contains_key("otel.metric.overflow"))
        .count();
    provider
        .shutdown()
        .map_err(|e| anyhow::anyhow!("shutdown: {e}"))?;
    Ok(CardinalityReport {
        accounts,
        bounded_series: bounded.points.len(),
        unbounded_series: unbounded.points.len(),
        buckets_per_series: bounded.points.first().map_or(0, |p| p.buckets),
        overflow_series,
    })
}

/// Kiểm và in chi phí cardinality.
///
/// # Errors
/// Số chuỗi không đúng kỳ vọng.
pub fn check_cardinality() -> Result<()> {
    let r = cardinality_cost(5_000)?;
    ensure!(
        r.bounded_series == 1,
        "histogram có giới hạn chỉ có 1 series, thấy {}",
        r.bounded_series
    );
    ensure!(
        r.unbounded_series > 1_000,
        "histogram theo account phải bùng nổ, thấy {}",
        r.unbounded_series
    );
    ensure!(
        r.overflow_series == 1,
        "SDK gom phần vượt giới hạn vào một series tràn"
    );
    println!(
        "cardinality ({} account): không account_id = {} series ({} số); có account_id = {} series ({} số), trong đó {} series tràn",
        r.accounts,
        r.bounded_series,
        r.numbers(r.bounded_series),
        r.unbounded_series,
        r.numbers(r.unbounded_series),
        r.overflow_series
    );
    Ok(())
}

/// Exporter chỉ đếm số span nhận được.
#[derive(Debug, Clone, Default)]
struct CountingExporter(Arc<AtomicUsize>);

impl SpanExporter for CountingExporter {
    fn export(&self, batch: Vec<SpanData>) -> impl Future<Output = OTelSdkResult> + Send {
        let counter = Arc::clone(&self.0);
        let n = batch.len();
        async move {
            counter.fetch_add(n, Ordering::SeqCst);
            Ok(())
        }
    }
}

/// Span còn đệm trong `BatchSpanProcessor` bị mất nếu không `shutdown`: dựng và đếm.
///
/// # Errors
/// Số span xuất ra không như kỳ vọng.
pub fn check_shutdown_flushes() -> Result<()> {
    let build = || {
        let counting = CountingExporter::default();
        // Chu kỳ 1 giờ: không có lần xuất định kỳ nào chen vào, kết quả không phụ thuộc đồng hồ và tải máy.
        let processor = BatchSpanProcessor::builder(counting.clone())
            .with_batch_config(
                BatchConfigBuilder::default()
                    .with_scheduled_delay(Duration::from_secs(3600))
                    .build(),
            )
            .build();
        let provider = SdkTracerProvider::builder()
            .with_span_processor(processor)
            .build();
        (provider, counting)
    };
    let emit = |provider: &SdkTracerProvider| {
        let tracer = provider.tracer("shutdown-demo");
        for _ in 0..5 {
            tracer.start("ledger.entry.post").end();
        }
    };

    // A. Còn một bản giữ provider (đúng như `global::set_tracer_provider` giữ một bản): bỏ bản cục bộ không flush.
    let (provider, counting) = build();
    let held_by_global = provider.clone();
    emit(&provider);
    ensure!(
        counting.0.load(Ordering::SeqCst) == 0,
        "span còn nằm trong hàng đợi, chưa được xuất"
    );
    drop(provider);
    ensure!(
        counting.0.load(Ordering::SeqCst) == 0,
        "bỏ một bản sao không tắt provider"
    );
    // Tiến trình thoát ở đây thì 5 span mất. Gọi shutdown thì được đẩy nốt.
    held_by_global
        .shutdown()
        .map_err(|e| anyhow::anyhow!("shutdown: {e}"))?;
    let after_shutdown = counting.0.load(Ordering::SeqCst);
    ensure!(
        after_shutdown == 5,
        "shutdown phải đẩy nốt 5 span, thấy {after_shutdown}"
    );

    // B. Bản cuối cùng bị bỏ thì SDK tự shutdown.
    let (provider, counting) = build();
    emit(&provider);
    drop(provider);
    ensure!(
        counting.0.load(Ordering::SeqCst) == 5,
        "bỏ bản cuối phải tự flush"
    );
    println!(
        "shutdown: trước 0 span xuất; giữ một bản sao (như global) rồi bỏ bản kia vẫn 0; shutdown() -> {after_shutdown}; bỏ bản cuối -> 5"
    );
    Ok(())
}

/// Các kịch bản không cần Kafka, Docker hay .NET.
///
/// # Errors
/// Kịch bản nào lỗi cũng dừng ngay.
pub fn run_offline_suite(t: &Telemetry) -> Result<()> {
    println!("== A. W3C Trace Context ==");
    check_w3c_format()?;
    check_unsampled_header(t)?;
    println!("\n== B. Sampler ==");
    check_sampling_decisions()?;
    println!("\n== C. Chi phí cardinality ==");
    check_cardinality()?;
    println!("\n== D. Shutdown ==");
    check_shutdown_flushes()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::shared_in_memory;

    #[test]
    fn w3c_format_round_trips_and_ignores_garbage() {
        check_w3c_format().unwrap();
    }

    #[test]
    fn unsampled_flag_reaches_kafka_header() {
        check_unsampled_header(shared_in_memory()).unwrap();
    }

    #[test]
    fn sampler_follows_parent_flag_and_root_ratio() {
        check_sampling_decisions().unwrap();
    }

    #[test]
    fn unbounded_label_explodes_series_count() {
        let small = cardinality_cost(100).unwrap();
        assert_eq!((small.bounded_series, small.unbounded_series), (1, 100));
        let big = cardinality_cost(3_000).unwrap();
        assert_eq!(big.bounded_series, 1);
        // Giới hạn mặc định của SDK là 2000 series; phần vượt gom vào một series tràn.
        assert_eq!(big.unbounded_series, 2_001);
        assert_eq!(big.overflow_series, 1);
    }

    #[test]
    fn shutdown_flushes_buffered_spans() {
        check_shutdown_flushes().unwrap();
    }

    #[test]
    fn set_parent_after_enter_is_rejected() {
        let _t = shared_in_memory();
        let span = tracing::info_span!("late.parent");
        let _entered = span.enter();
        // Span đã chạy thì không đổi cha được nữa: gọi `set_parent` phải TRƯỚC `enter`.
        let err = span.set_parent(Context::new()).unwrap_err();
        assert!(matches!(
            err,
            tracing_opentelemetry::SetParentError::AlreadyStarted
        ));
    }

    #[test]
    fn kafka_headers_round_trip_context() {
        use crate::propagation::extract_from_kafka;
        let t = shared_in_memory();
        let _ = t;
        let span = tracing::info_span!("kafka.round_trip");
        let (headers, expected) = span.in_scope(|| {
            (
                current_kafka_headers(),
                tracing::Span::current()
                    .context()
                    .span()
                    .span_context()
                    .trace_id(),
            )
        });
        let cx = extract_from_kafka(Some(&headers));
        assert_eq!(cx.span().span_context().trace_id(), expected);
        // Thông điệp không có header: context rỗng, không panic.
        assert!(
            !extract_from_kafka::<rdkafka::message::OwnedHeaders>(None)
                .span()
                .span_context()
                .is_valid()
        );
        let empty = rdkafka::message::OwnedHeaders::new();
        assert!(
            !extract_from_kafka(Some(&empty))
                .span()
                .span_context()
                .is_valid()
        );
    }
}
