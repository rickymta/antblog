//! HTTP `/v1`: handler mỏng, chỉ giải mã → gọi use case → mã hóa. Mọi request có `request_id`.

use std::sync::Arc;

use axum::body::Body;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{FromRef, FromRequest, OriginalUri, Path, Request, State};
use axum::http::header::HeaderName;
use axum::http::{HeaderValue, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use ledger_application::model::PostOutcome;
use ledger_application::readiness::Readiness;
use ledger_application::{Ledger, LedgerStore};
use serde::de::DeserializeOwned;
use tower_http::request_id::{
    MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer,
};
use tower_http::trace::{DefaultOnFailure, DefaultOnResponse, TraceLayer};
use tracing::{Level, info_span};

use crate::auth::{AuthConfig, Authorized, Readers, Writers};
use crate::dto::{
    AvailableResponse, BalanceResponse, EntryResponse, HoldRequest, HoldResponse, PostEntryRequest,
    RefundRequest,
};
use crate::error_map::problem_from_error;
use crate::problem::{ApiError, Problem};
use crate::validate::{DEFAULT_TTL_SECS, ensure_valid, entry_errors, hold_errors, refund_errors};

/// Thứ handler cần: use case, khóa JWT và cờ sẵn sàng. `Clone` rẻ (chỉ nhân bản `Arc`).
pub struct ApiState<S: LedgerStore> {
    /// Các use case.
    pub ledger: Arc<Ledger<S>>,
    /// Khóa và quy tắc kiểm token.
    pub auth: Arc<AuthConfig>,
    /// Cờ sẵn sàng (consumer đặt, `/readyz` đọc).
    pub ready: Arc<Readiness>,
}

// Viết tay vì `#[derive(Clone)]` đòi `S: Clone`, mà `Arc<Ledger<S>>` không cần.
impl<S: LedgerStore> Clone for ApiState<S> {
    fn clone(&self) -> Self {
        ApiState {
            ledger: self.ledger.clone(),
            auth: self.auth.clone(),
            ready: self.ready.clone(),
        }
    }
}

// Cho phép extractor `Authorized` lấy khóa từ state mà không biết `ApiState`.
impl<S: LedgerStore> FromRef<ApiState<S>> for Arc<AuthConfig> {
    fn from_ref(state: &ApiState<S>) -> Self {
        state.auth.clone()
    }
}

/// Giống `axum::Json<T>` nhưng `Rejection` là Problem Details (bài 32).
struct ApiJson<T>(T);

impl<S, T> FromRequest<S> for ApiJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, ApiError> {
        let instance = req
            .extensions()
            .get::<OriginalUri>()
            .map_or_else(|| req.uri().path().to_string(), |o| o.0.path().to_string());
        match axum::Json::<T>::from_request(req, state).await {
            Ok(axum::Json(value)) => Ok(ApiJson(value)),
            Err(rejection) => {
                let (slug, title) = match &rejection {
                    JsonRejection::JsonSyntaxError(_) => {
                        ("malformed-json", "JSON không đúng cú pháp")
                    }
                    JsonRejection::JsonDataError(_) => {
                        ("invalid-json-body", "JSON không đúng hình dạng")
                    }
                    JsonRejection::MissingJsonContentType(_) => (
                        "unsupported-media-type",
                        "Cần Content-Type: application/json",
                    ),
                    _ => ("unreadable-body", "Không đọc được body"),
                };
                Err(Problem::new(
                    rejection.status(),
                    slug,
                    title,
                    rejection.body_text(),
                    &instance,
                )
                .into())
            }
        }
    }
}

// Sinh `x-request-id` mới khi client không gửi; nếu client gửi thì SetRequestIdLayer giữ nguyên.
#[derive(Clone, Copy)]
struct NewRequestId;

impl MakeRequestId for NewRequestId {
    fn make_request_id<B>(&mut self, _request: &axum::http::Request<B>) -> Option<RequestId> {
        HeaderValue::from_str(&uuid::Uuid::new_v4().to_string())
            .ok()
            .map(RequestId::new)
    }
}

/// Dựng router. Thứ tự `.layer`: lớp thêm sau nằm ngoài. `SetRequestId` ngoài cùng để span thấy header.
pub fn router<S: LedgerStore>(state: ApiState<S>) -> Router {
    let header = HeaderName::from_static("x-request-id");
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz::<S>))
        .route("/v1/entries", post(post_entry::<S>))
        .route("/v1/accounts/{id}/balance", get(get_balance::<S>))
        .route("/v1/accounts/{id}/available", get(get_available::<S>))
        .route("/v1/holds", post(post_hold::<S>))
        .route("/v1/trips/{trip_id}/refunds", post(post_refund::<S>))
        .fallback(route_not_found)
        .method_not_allowed_fallback(method_not_allowed)
        .with_state(state)
        .layer(PropagateRequestIdLayer::new(header.clone()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &axum::http::Request<Body>| {
                    let request_id = req
                        .headers()
                        .get("x-request-id")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("-");
                    info_span!("http", %request_id, method = %req.method(), path = %req.uri().path())
                })
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                // 503 của /readyz là bình thường lúc khởi động và lúc tắt: chỉ cảnh báo.
                .on_failure(DefaultOnFailure::new().level(Level::WARN)),
        )
        .layer(SetRequestIdLayer::new(header, NewRequestId))
}

// Liveness: process còn chạy. Cố ý không hỏi DB hay Kafka: DB chậm không được làm orchestrator giết pod.
async fn healthz() -> &'static str {
    "ok"
}

// Readiness: nhận được việc chưa? Cần DB trả lời, consumer đã có partition, và chưa bắt đầu tắt.
async fn readyz<S: LedgerStore>(
    State(state): State<ApiState<S>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut reasons = Vec::new();
    if state.ledger.ping().await.is_err() {
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

fn created_or_ok(replayed: bool) -> StatusCode {
    if replayed {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    }
}

fn path_problem(rejection: &PathRejection, instance: &str) -> ApiError {
    Problem::new(
        rejection.status(),
        "invalid-path",
        "Tham số đường dẫn sai dạng",
        rejection.body_text(),
        instance,
    )
    .into()
}

/// `POST /v1/entries`: ghi một bút toán thủ công. Khác bài 40: endpoint này có ở bản hoàn chỉnh.
async fn post_entry<S: LedgerStore>(
    _auth: Authorized<Writers>,
    State(state): State<ApiState<S>>,
    OriginalUri(uri): OriginalUri,
    ApiJson(body): ApiJson<PostEntryRequest>,
) -> Result<(StatusCode, Json<EntryResponse>), ApiError> {
    let instance = uri.path();
    ensure_valid(entry_errors(&body), instance)?;
    let lines: Vec<(String, i64)> = body
        .postings
        .iter()
        .map(|p| (p.account.clone(), p.amount_dong))
        .collect();
    let outcome = state
        .ledger
        .post_entry(&body.idempotency_key, &lines)
        .await
        .map_err(|e| problem_from_error(&e, instance))?;
    Ok((
        created_or_ok(outcome.replayed()),
        Json(entry_response(&outcome, &body.idempotency_key)),
    ))
}

fn entry_response(outcome: &PostOutcome, key: &str) -> EntryResponse {
    EntryResponse {
        entry_id: outcome.entry_id().to_string(),
        idempotency_key: key.to_string(),
        replayed: outcome.replayed(),
    }
}

async fn get_balance<S: LedgerStore>(
    _auth: Authorized<Readers>,
    State(state): State<ApiState<S>>,
    OriginalUri(uri): OriginalUri,
    path: Result<Path<String>, PathRejection>,
) -> Result<Json<BalanceResponse>, ApiError> {
    let instance = uri.path();
    let Path(account) = path.map_err(|r| path_problem(&r, instance))?;
    let balance = state
        .ledger
        .balance(&account)
        .await
        .map_err(|e| problem_from_error(&e, instance))?;
    Ok(Json(BalanceResponse {
        account,
        balance_dong: balance.dong(),
        as_of: state.ledger.now().to_string(),
    }))
}

async fn get_available<S: LedgerStore>(
    _auth: Authorized<Readers>,
    State(state): State<ApiState<S>>,
    OriginalUri(uri): OriginalUri,
    path: Result<Path<String>, PathRejection>,
) -> Result<Json<AvailableResponse>, ApiError> {
    let instance = uri.path();
    let Path(account) = path.map_err(|r| path_problem(&r, instance))?;
    let funds = state
        .ledger
        .funds(&account)
        .await
        .map_err(|e| problem_from_error(&e, instance))?;
    Ok(Json(AvailableResponse {
        account,
        balance_dong: funds.balance.dong(),
        held_dong: funds.held.dong(),
        // Khả dụng tính bằng i128 trong domain; cắt về biên i64 khi đưa ra dây.
        available_dong: i64::try_from(funds.available()).unwrap_or(i64::MIN),
        as_of: state.ledger.now().to_string(),
    }))
}

async fn post_hold<S: LedgerStore>(
    _auth: Authorized<Writers>,
    State(state): State<ApiState<S>>,
    OriginalUri(uri): OriginalUri,
    ApiJson(body): ApiJson<HoldRequest>,
) -> Result<(StatusCode, Json<HoldResponse>), ApiError> {
    let instance = uri.path();
    ensure_valid(hold_errors(&body), instance)?;
    let outcome = state
        .ledger
        .place_hold(
            &body.account,
            body.amount_dong,
            &body.idempotency_key,
            body.ttl_seconds.unwrap_or(DEFAULT_TTL_SECS),
        )
        .await
        .map_err(|e| problem_from_error(&e, instance))?;
    Ok((
        created_or_ok(outcome.replayed()),
        Json(HoldResponse {
            hold_id: outcome.hold_id(),
            replayed: outcome.replayed(),
        }),
    ))
}

async fn post_refund<S: LedgerStore>(
    _auth: Authorized<Writers>,
    State(state): State<ApiState<S>>,
    OriginalUri(uri): OriginalUri,
    path: Result<Path<String>, PathRejection>,
    ApiJson(body): ApiJson<RefundRequest>,
) -> Result<(StatusCode, Json<EntryResponse>), ApiError> {
    let instance = uri.path();
    let Path(trip_id) = path.map_err(|r| path_problem(&r, instance))?;
    ensure_valid(refund_errors(&body), instance)?;
    let outcome = state
        .ledger
        .refund(&trip_id, body.refund_no, body.amount_dong)
        .await
        .map_err(|e| problem_from_error(&e, instance))?;
    let key = ledger_domain_refund_key(&trip_id, body.refund_no);
    Ok((
        created_or_ok(outcome.replayed()),
        Json(entry_response(&outcome, &key)),
    ))
}

fn ledger_domain_refund_key(trip_id: &str, n: u32) -> String {
    ledger_application::domain::refund::refund_key(trip_id, n)
}

/// Không có route nào khớp đường dẫn.
async fn route_not_found(OriginalUri(uri): OriginalUri) -> Problem {
    Problem::new(
        StatusCode::NOT_FOUND,
        "route-not-found",
        "Không có đường dẫn này",
        format!("không có route {}", uri.path()),
        uri.path(),
    )
}

/// Đường dẫn có nhưng sai phương thức HTTP.
async fn method_not_allowed(OriginalUri(uri): OriginalUri) -> Problem {
    Problem::new(
        StatusCode::METHOD_NOT_ALLOWED,
        "method-not-allowed",
        "Phương thức không được hỗ trợ",
        format!("phương thức không hợp lệ với {}", uri.path()),
        uri.path(),
    )
}
