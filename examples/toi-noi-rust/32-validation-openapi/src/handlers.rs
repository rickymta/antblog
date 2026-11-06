//! Handler `/v1` và trạng thái chia sẻ (sổ cái trong bộ nhớ).

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use axum::Json;
use axum::extract::rejection::PathRejection;
use axum::extract::{OriginalUri, Path, State};
use axum::http::StatusCode;
use ledger_core::{AccountId, Ledger};

use crate::dto::{BalanceResponse, EntryResponse, PostEntryRequest, PostingDto};
use crate::error_map::problem_from_ledger;
use crate::extract::{ApiJson, problem_from_path};
use crate::problem::{ApiError, Problem};
use crate::validate::into_entry;

/// Trạng thái dùng chung. Lab dùng sổ cái trong bộ nhớ vì bài chỉ bàn tầng HTTP;
/// hợp đồng HTTP không phụ thuộc nơi lưu bút toán.
#[derive(Clone, Default)]
pub struct AppState {
    ledger: Arc<Mutex<Ledger>>,
}

impl AppState {
    // Sổ cái chỉ thêm vào cuối, nên dữ liệu vẫn nhất quán dù một luồng từng panic khi giữ khóa.
    fn ledger(&self) -> MutexGuard<'_, Ledger> {
        self.ledger.lock().unwrap_or_else(PoisonError::into_inner)
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
        (status = 200, description = "Replay: cùng key, cùng nội dung", body = EntryResponse),
        (status = 400, description = "JSON sai cú pháp", body = Problem, content_type = "application/problem+json"),
        (status = 409, description = "Key đã dùng cho nội dung khác", body = Problem, content_type = "application/problem+json"),
        (status = 413, description = "Body vượt giới hạn 2 MB của axum", body = Problem, content_type = "application/problem+json"),
        (status = 415, description = "Thiếu Content-Type: application/json", body = Problem, content_type = "application/problem+json"),
        (status = 422, description = "Sai hình dạng hoặc bút toán lệch", body = Problem, content_type = "application/problem+json"),
    )
)]
pub async fn post_entry(
    State(state): State<AppState>,
    OriginalUri(uri): OriginalUri,
    ApiJson(body): ApiJson<PostEntryRequest>,
) -> Result<(StatusCode, Json<EntryResponse>), ApiError> {
    let instance = uri.path();
    let entry = into_entry(body, instance)?;
    let mut ledger = state.ledger();
    let posted = ledger
        .post(entry)
        .map_err(|e| problem_from_ledger(&e, instance))?;
    let response = EntryResponse {
        entry_id: posted.id.to_string(),
        idempotency_key: posted.entry.idempotency_key().to_string(),
        postings: posted
            .entry
            .postings()
            .iter()
            .map(|p| PostingDto {
                account: p.account.to_string(),
                amount_dong: p.amount.dong(),
            })
            .collect(),
        replayed: posted.replayed,
    };
    let status = if posted.replayed {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };
    Ok((status, Json(response)))
}

/// Số dư của một tài khoản đã có bút toán.
#[utoipa::path(
    get,
    path = "/v1/accounts/{id}/balance",
    tag = "ledger",
    params(("id" = String, Path, description = "Mã tài khoản", example = "driver:DR-001")),
    responses(
        (status = 200, description = "Số dư hiện tại", body = BalanceResponse),
        (status = 400, description = "Mã tài khoản trên đường dẫn sai dạng", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Tài khoản chưa có bút toán nào", body = Problem, content_type = "application/problem+json"),
    )
)]
pub async fn get_balance(
    State(state): State<AppState>,
    OriginalUri(uri): OriginalUri,
    path: Result<Path<String>, PathRejection>,
) -> Result<Json<BalanceResponse>, ApiError> {
    let instance = uri.path();
    let Path(id) = path.map_err(|r| problem_from_path(&r, instance))?;
    // Trên đường dẫn thì mã rỗng là tham số sai dạng (400), giống bài 31; trong body nó là 422.
    let account = AccountId::new(&id).map_err(|e| {
        Problem::new(
            StatusCode::BAD_REQUEST,
            "invalid-account-id",
            "Mã tài khoản sai dạng",
            e.to_string(),
            instance,
        )
    })?;
    let balances = state
        .ledger()
        .balances()
        .map_err(|e| problem_from_ledger(&e, instance))?;
    let Some(balance) = balances.get(&account) else {
        return Err(Problem::new(
            StatusCode::NOT_FOUND,
            "unknown-account",
            "Tài khoản chưa có bút toán",
            format!("không có bút toán nào chạm tài khoản {account}"),
            instance,
        )
        .into());
    };
    Ok(Json(BalanceResponse {
        account: account.to_string(),
        balance_dong: balance.dong(),
        as_of: jiff::Timestamp::now().to_string(),
    }))
}

/// Không có route nào khớp đường dẫn.
pub async fn route_not_found(OriginalUri(uri): OriginalUri) -> Problem {
    Problem::new(
        StatusCode::NOT_FOUND,
        "route-not-found",
        "Không có đường dẫn này",
        format!("không có route {}", uri.path()),
        uri.path(),
    )
}

/// Đường dẫn có nhưng sai phương thức HTTP.
pub async fn method_not_allowed(OriginalUri(uri): OriginalUri) -> Problem {
    Problem::new(
        StatusCode::METHOD_NOT_ALLOWED,
        "method-not-allowed",
        "Phương thức không được hỗ trợ",
        format!("phương thức không hợp lệ với {}", uri.path()),
        uri.path(),
    )
}
