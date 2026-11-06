//! `Router`, handler và DTO: lớp HTTP mỏng bọc quanh `ledger-core` và kho PostgreSQL.

use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::{Method, StatusCode, Uri};
use axum::routing::{get, post};
use axum::{Json, Router};
use ledger_core::{AccountId, JournalEntry, LedgerError, Money, Posting};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::healthz::Healthz;
use crate::store::PgLedger;

/// State dùng chung của mọi handler. `Clone` rẻ: `PgPool` bên trong là một `Arc`.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Kho sổ cái.
    pub ledger: PgLedger,
}

/// Một dòng của bút toán trên dây: `{ "account": "...", "amountDong": -25500 }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PostingDto {
    /// Mã tài khoản.
    pub account: String,
    /// Số đồng có dấu: dương là vào, âm là ra.
    pub amount_dong: i64,
}

/// Body của `POST /v1/entries`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PostEntryRequest {
    /// Khóa idempotency, ví dụ `trip:TN-0001:capture`.
    pub idempotency_key: String,
    /// Các dòng bút toán.
    pub postings: Vec<PostingDto>,
}

impl PostEntryRequest {
    // DTO sang `JournalEntry`: chỗ duy nhất bất biến (≥ 2 dòng, tổng 0) được kiểm.
    fn into_entry(self) -> Result<JournalEntry, LedgerError> {
        let postings = self
            .postings
            .into_iter()
            .map(|p| {
                Ok(Posting::new(
                    AccountId::new(&p.account)?,
                    Money::from_dong(p.amount_dong),
                ))
            })
            .collect::<Result<Vec<_>, LedgerError>>()?;
        JournalEntry::try_new(&self.idempotency_key, postings)
    }
}

/// Thân trả về của `POST /v1/entries`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostEntryResponse {
    /// Mã bút toán `JE-000001`.
    pub entry_id: String,
    /// Khóa idempotency đã dùng.
    pub idempotency_key: String,
    /// `true` nếu đây là lần gửi lại: bút toán cũ được trả lại.
    pub replayed: bool,
}

/// Thân trả về của `GET /v1/accounts/{id}/balance`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponse {
    /// Mã tài khoản.
    pub account: String,
    /// Số dư tính bằng đồng, có dấu.
    pub balance_dong: i64,
    /// Thời điểm đọc theo đồng hồ PostgreSQL, UTC.
    pub as_of: String,
}

/// Dựng router. Mỗi `route` ghép một đường dẫn với một handler theo phương thức.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/v1/entries", post(post_entry))
        .route("/v1/accounts/{id}/balance", get(get_balance))
        // Một Service viết tay đứng cạnh handler: cả hai cùng là Service trong mắt Router.
        .route_service("/healthz", Healthz)
        .fallback(not_found)
        .with_state(state)
}

// Thứ tự tham số là thứ tự extractor chạy. `State` không đọc body; `Json` tiêu thụ body nên phải đứng cuối.
// Dùng `Result<Json<_>, JsonRejection>` để lỗi body đi qua `AppError` thay vì response mặc định của axum.
async fn post_entry(
    State(state): State<AppState>,
    body: Result<Json<PostEntryRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<PostEntryResponse>), AppError> {
    let Json(request) = body?;
    let idempotency_key = request.idempotency_key.clone();
    let entry = request.into_entry()?;
    let outcome = state.ledger.post(&entry).await?;
    // 201 khi tạo mới, 200 khi gửi lại cùng key cùng nội dung.
    let status = if outcome.replayed {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };
    Ok((
        status,
        Json(PostEntryResponse {
            entry_id: outcome.id.to_string(),
            idempotency_key,
            replayed: outcome.replayed,
        }),
    ))
}

async fn get_balance(
    State(state): State<AppState>,
    path: Result<Path<String>, PathRejection>,
) -> Result<Json<BalanceResponse>, AppError> {
    let Path(id) = path?;
    let account = AccountId::new(&id).map_err(AppError::BadAccountId)?;
    let read = state
        .ledger
        .balance(&account)
        .await?
        .ok_or_else(|| AppError::AccountNotFound(id))?;
    Ok(Json(BalanceResponse {
        account: account.to_string(),
        balance_dong: read.balance.dong(),
        as_of: read.as_of,
    }))
}

// Fallback: route không khớp cũng trả JSON lỗi cùng dạng, thay vì thân rỗng của axum.
async fn not_found(method: Method, uri: Uri) -> AppError {
    AppError::RouteNotFound {
        method: method.to_string(),
        path: uri.path().to_string(),
    }
}
