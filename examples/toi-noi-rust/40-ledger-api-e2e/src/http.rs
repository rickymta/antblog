//! HTTP: `/healthz`, `/readyz` và `GET /v1/accounts/{id}/balance`, mọi request có `request_id`.

use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header::HeaderName;
use axum::http::{HeaderValue, Request, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tower_http::request_id::{
    MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer,
};
use tower_http::trace::{DefaultOnFailure, DefaultOnResponse, TraceLayer};
use tracing::{Level, error, info_span};

use crate::state::Readiness;
use crate::store::Store;

/// Thứ handler cần: kho dữ liệu và cờ sẵn sàng.
#[derive(Clone)]
pub struct HttpState {
    /// Truy cập PostgreSQL.
    pub store: Store,
    /// Cờ sẵn sàng.
    pub ready: Arc<Readiness>,
}

/// Thân trả về của `GET /v1/accounts/{id}/balance`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceBody {
    /// Mã tài khoản.
    pub account: String,
    /// Số dư, đồng (có dấu).
    pub balance_dong: i64,
    /// Thời điểm đọc, ISO 8601 UTC.
    pub as_of: String,
}

// Sinh `x-request-id` mới khi client không gửi; nếu client gửi thì SetRequestIdLayer giữ nguyên.
#[derive(Clone, Copy)]
struct NewRequestId;

impl MakeRequestId for NewRequestId {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        HeaderValue::from_str(&uuid::Uuid::new_v4().to_string())
            .ok()
            .map(RequestId::new)
    }
}

/// Dựng router. Thứ tự `.layer`: lớp thêm sau nằm ngoài. `SetRequestId` ngoài cùng để span thấy header.
pub fn router(state: HttpState) -> Router {
    let header = HeaderName::from_static("x-request-id");
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/v1/accounts/{id}/balance", get(balance))
        .with_state(state)
        .layer(PropagateRequestIdLayer::new(header.clone()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &Request<Body>| {
                    let request_id = req
                        .headers()
                        .get("x-request-id")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("-");
                    info_span!("http", %request_id, method = %req.method(), path = %req.uri().path())
                })
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                // 503 của /readyz là bình thường lúc khởi động và lúc tắt: chỉ cảnh báo, không phải ERROR.
                .on_failure(DefaultOnFailure::new().level(Level::WARN)),
        )
        .layer(SetRequestIdLayer::new(header, NewRequestId))
}

// Liveness: process còn chạy. Cố ý không hỏi DB hay Kafka: DB chậm không được làm orchestrator giết pod.
async fn healthz() -> &'static str {
    "ok"
}

// Readiness: nhận được việc chưa? Cần DB trả lời và consumer đã có partition, và chưa bắt đầu tắt.
async fn readyz(State(state): State<HttpState>) -> (StatusCode, Json<serde_json::Value>) {
    let mut reasons = Vec::new();
    if state.store.ping().await.is_err() {
        reasons.push("database");
    }
    if !state.ready.is_ready() {
        reasons.push("consumer");
    }
    if reasons.is_empty() {
        (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "ready" })),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "status": "not_ready", "reasons": reasons })),
        )
    }
}

async fn balance(
    State(state): State<HttpState>,
    Path(account): Path<String>,
) -> Result<Json<BalanceBody>, (StatusCode, Json<serde_json::Value>)> {
    match state.store.balance(&account).await {
        Ok(Some(balance_dong)) => Ok(Json(BalanceBody {
            account,
            balance_dong,
            as_of: jiff::Timestamp::now().to_string(),
        })),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "account_not_found", "account": account })),
        )),
        Err(err) => {
            error!(%err, "đọc số dư lỗi");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "internal" })),
            ))
        }
    }
}
