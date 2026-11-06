//! Router axum, DTO và tài liệu OpenAPI của ledger API. Lỗi nằm ở `problem.rs`.

use std::sync::Arc;

use axum::extract::rejection::PathRejection;
use axum::extract::{FromRequest, OriginalUri, Path, Request, State};
use axum::http::{StatusCode, Uri};
use axum::routing::{get, post};
use axum::{Json, Router};
use ledger_core::AccountId;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi, ToSchema};

use crate::auth::{AuthConfig, Caller, SCOPE_READ, Writer};
use crate::problem::{
    ApiError, FieldError, Problem, problem_from_path, problem_from_post, problem_from_rejection,
};
use crate::store::PgLedger;
use crate::validate::into_entry;

/// Đường dẫn của tài liệu OpenAPI dạng JSON, như bài 32.
pub const OPENAPI_PATH: &str = "/api-docs/openapi.json";

/// Trạng thái dùng chung của mọi handler.
#[derive(Clone)]
pub struct AppState {
    /// Sổ cái trên PostgreSQL.
    pub ledger: PgLedger,
    /// Cấu hình kiểm tra JWT.
    pub auth: Arc<AuthConfig>,
}

/// Một dòng bút toán trên dây: số tiền có dấu, dương là vào, âm là ra.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PostingDto {
    /// Mã tài khoản, ví dụ `driver:DR-001`.
    #[schema(example = "driver:DR-001")]
    pub account: String,
    /// Số tiền đồng, có dấu, khác 0.
    #[schema(example = 20400)]
    pub amount_dong: i64,
}

/// Body của `POST /v1/entries`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PostEntryRequest {
    /// Khoá idempotency của bút toán.
    #[schema(example = "trip:TN-0001:capture")]
    pub idempotency_key: String,
    /// Ít nhất 2 dòng, tổng số tiền bằng 0.
    pub postings: Vec<PostingDto>,
}

/// Kết quả ghi bút toán.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EntryResponse {
    /// Mã bút toán `JE-000001`.
    #[schema(example = "JE-000001")]
    pub entry_id: String,
    /// Khoá idempotency đã dùng.
    pub idempotency_key: String,
    /// `true` nếu key đã có và nội dung trùng (HTTP 200); `false` khi vừa tạo (HTTP 201).
    pub replayed: bool,
    /// Các dòng đã ghi, theo thứ tự gửi.
    pub postings: Vec<PostingDto>,
}

/// Kết quả `GET /v1/accounts/{id}/balance`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponse {
    /// Mã tài khoản.
    pub account: String,
    /// Số dư đồng, có dấu.
    pub balance_dong: i64,
    /// Thời điểm đọc, RFC 3339 UTC.
    #[schema(example = "2026-11-06T09:30:00Z")]
    pub as_of: String,
}

/// Bọc `axum::Json` để lỗi đọc body cũng ra problem+json thay vì chuỗi trơn của axum.
pub struct ApiJson<T>(pub T);

impl<S, T> FromRequest<S> for ApiJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, ApiError> {
        let instance = req.uri().path().to_string();
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(ApiJson(value)),
            Err(rejection) => Err(problem_from_rejection(&rejection, &instance).into()),
        }
    }
}

/// Ghi một bút toán.
#[utoipa::path(
    post,
    path = "/v1/entries",
    tag = "ledger",
    request_body = PostEntryRequest,
    responses(
        (status = 201, description = "Bút toán mới được ghi", body = EntryResponse),
        (status = 200, description = "Replay: key đã có, nội dung trùng", body = EntryResponse),
        (status = 400, description = "JSON hỏng", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Thiếu hoặc sai token", body = Problem, content_type = "application/problem+json"),
        (status = 403, description = "Thiếu quyền ledger:write", body = Problem, content_type = "application/problem+json"),
        (status = 409, description = "Cùng key, khác nội dung", body = Problem, content_type = "application/problem+json"),
        (status = 413, description = "Body vượt giới hạn 2 MB của axum", body = Problem, content_type = "application/problem+json"),
        (status = 415, description = "Không phải application/json", body = Problem, content_type = "application/problem+json"),
        (status = 422, description = "Bút toán lệch, thiếu dòng, sai hình dạng hoặc tài khoản chưa có", body = Problem, content_type = "application/problem+json"),
    ),
    security(("bearer" = []))
)]
pub async fn post_entry(
    State(state): State<AppState>,
    OriginalUri(uri): OriginalUri,
    Writer(_caller): Writer,
    ApiJson(request): ApiJson<PostEntryRequest>,
) -> Result<(StatusCode, Json<EntryResponse>), ApiError> {
    let instance = uri.path();
    let entry = into_entry(&request, instance)?;
    let posted = state
        .ledger
        .post(&entry)
        .await
        .map_err(|e| problem_from_post(&e, instance))?;
    let status = if posted.replayed {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };
    Ok((
        status,
        Json(EntryResponse {
            entry_id: posted.display_id(),
            idempotency_key: request.idempotency_key,
            replayed: posted.replayed,
            postings: request.postings,
        }),
    ))
}

/// Đọc số dư một tài khoản.
#[utoipa::path(
    get,
    path = "/v1/accounts/{id}/balance",
    tag = "ledger",
    params(("id" = String, Path, description = "Mã tài khoản, ví dụ driver:DR-001")),
    responses(
        (status = 200, description = "Số dư hiện tại", body = BalanceResponse),
        (status = 400, description = "Mã tài khoản trên đường dẫn sai dạng", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Thiếu hoặc sai token", body = Problem, content_type = "application/problem+json"),
        (status = 403, description = "Thiếu quyền ledger:read", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Tài khoản chưa tồn tại", body = Problem, content_type = "application/problem+json"),
    ),
    security(("bearer" = []))
)]
pub async fn get_balance(
    State(state): State<AppState>,
    OriginalUri(uri): OriginalUri,
    caller: Caller,
    path: Result<Path<String>, PathRejection>,
) -> Result<Json<BalanceResponse>, ApiError> {
    let instance = uri.path();
    caller.require(SCOPE_READ, instance)?;
    let Path(id) = path.map_err(|r| problem_from_path(&r, instance))?;
    // Trên đường dẫn thì mã rỗng là tham số sai dạng (400), giống bài 32; trong body nó là 422.
    let account = AccountId::new(&id).map_err(|e| {
        Problem::new(
            StatusCode::BAD_REQUEST,
            "invalid-account-id",
            "Mã tài khoản sai dạng",
            e.to_string(),
            instance,
        )
    })?;
    let row = state
        .ledger
        .balance(&account.to_string())
        .await
        .map_err(|e| Problem::internal(&e.to_string(), instance))?
        .ok_or_else(|| {
            Problem::new(
                StatusCode::NOT_FOUND,
                "unknown-account",
                "Tài khoản chưa tồn tại",
                format!("tài khoản {account} chưa tồn tại"),
                instance,
            )
        })?;
    Ok(Json(BalanceResponse {
        account: account.to_string(),
        balance_dong: row.balance_dong,
        as_of: row.as_of,
    }))
}

/// Không có route nào khớp đường dẫn.
async fn route_not_found(uri: Uri) -> Problem {
    Problem::new(
        StatusCode::NOT_FOUND,
        "route-not-found",
        "Không có đường dẫn này",
        format!("không có route {}", uri.path()),
        uri.path(),
    )
}

/// Đường dẫn có nhưng sai phương thức HTTP.
async fn method_not_allowed(uri: Uri) -> Problem {
    Problem::new(
        StatusCode::METHOD_NOT_ALLOWED,
        "method-not-allowed",
        "Phương thức không được hỗ trợ",
        format!("phương thức không hợp lệ với {}", uri.path()),
        uri.path(),
    )
}

/// Thêm security scheme `bearer` vào tài liệu.
struct BearerScheme;

impl Modify for BearerScheme {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

/// Tài liệu OpenAPI sinh từ code.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Tới Nơi Ledger API",
        version = "1.0.0",
        description = "API sổ cái double-entry của Tới Nơi. Lỗi trả application/problem+json (RFC 9457)."
    ),
    paths(post_entry, get_balance),
    components(schemas(
        PostEntryRequest,
        PostingDto,
        EntryResponse,
        BalanceResponse,
        Problem,
        FieldError
    )),
    tags((name = "ledger", description = "Ghi bút toán và tra số dư")),
    modifiers(&BearerScheme)
)]
pub struct ApiDoc;

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

/// Dựng router. Không bind cổng: test gọi thẳng, binary mới đem ra socket.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/v1/entries", post(post_entry))
        .route("/v1/accounts/{id}/balance", get(get_balance))
        .route(OPENAPI_PATH, get(openapi_json))
        .fallback(route_not_found)
        .method_not_allowed_fallback(method_not_allowed)
        .with_state(state)
}
