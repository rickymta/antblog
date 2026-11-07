//! Client giả lập phía C# bằng Rust: tạo span gốc, span CLIENT và gắn `traceparent` vào metadata.
//! Dùng cho `--verify` và `--with-collector`; client C# thật nằm ở `dotnet-client/`.

use anyhow::Result;
use opentelemetry::trace::TraceContextExt;
use tonic::{Request, Status};
use tracing::{Instrument, field};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::pb::ledger_client::LedgerClient;
use crate::pb::{PostEntryRequest, PostEntryResponse, Posting};
use crate::propagation::inject_current_into_metadata;
use crate::service::{POST_ENTRY, code_name};

/// Kết quả một lần gọi: `trace_id` của trace do upstream mở và kết quả RPC.
pub struct Call {
    pub trace_id: String,
    pub result: Result<PostEntryResponse, Status>,
}

/// Bút toán capture chuẩn của chuyến `TN-0001` (25.500 đồng, qua clearing).
pub fn capture_request() -> PostEntryRequest {
    PostEntryRequest {
        idempotency_key: "trip:TN-0001:capture".to_owned(),
        postings: vec![
            Posting {
                account: "customer:CU-001".to_owned(),
                amount_dong: -25_500,
            },
            Posting {
                account: "psp:vixanh:clearing".to_owned(),
                amount_dong: 25_500,
            },
        ],
    }
}

/// Gọi `PostEntry` như service Payment: span gốc `payment.capture-trip`, span CLIENT cho RPC.
///
/// # Errors
/// Không kết nối được server.
pub async fn capture_trip(addr: std::net::SocketAddr, request: PostEntryRequest) -> Result<Call> {
    let mut client = LedgerClient::connect(format!("http://{addr}")).await?;
    let root = tracing::info_span!("payment.capture-trip");
    let trace_id = root.context().span().span_context().trace_id().to_string();
    let result = async {
        let rpc = tracing::info_span!(
            "ledger.v1.Ledger/PostEntry",
            otel.kind = "client",
            rpc.system.name = "grpc",
            rpc.method = POST_ENTRY,
            rpc.status_code = field::Empty,
            server.address = %addr.ip(),
            server.port = i64::from(addr.port()),
            error.type = field::Empty,
        );
        async {
            let mut req = Request::new(request);
            // Span CLIENT đang là span hiện tại, nên nó là cha của span SERVER bên kia.
            inject_current_into_metadata(req.metadata_mut());
            let result = client
                .post_entry(req)
                .await
                .map(tonic::Response::into_inner);
            let code = result.as_ref().err().map_or(tonic::Code::Ok, Status::code);
            let span = tracing::Span::current();
            span.record("rpc.status_code", code_name(code));
            // Phía client, mọi mã khác OK đều là lỗi (semantic conventions gRPC).
            if code != tonic::Code::Ok {
                span.record("error.type", code_name(code));
            }
            result
        }
        .instrument(rpc)
        .await
    }
    .instrument(root)
    .await;
    Ok(Call { trace_id, result })
}

/// Gọi với header `traceparent` viết tay, như một client không dùng SDK của ta. `None` = không gửi header.
///
/// # Errors
/// Không kết nối được server.
pub async fn call_with_raw_traceparent(
    addr: std::net::SocketAddr,
    traceparent: Option<&str>,
    request: PostEntryRequest,
) -> Result<Result<PostEntryResponse, Status>> {
    let mut client = LedgerClient::connect(format!("http://{addr}")).await?;
    let mut req = Request::new(request);
    if let Some(value) = traceparent {
        req.metadata_mut().insert("traceparent", value.parse()?);
    }
    Ok(client
        .post_entry(req)
        .await
        .map(tonic::Response::into_inner))
}
