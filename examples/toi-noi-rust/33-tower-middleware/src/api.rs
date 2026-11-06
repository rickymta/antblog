//! Hai endpoint của sổ cái (rút gọn từ bài 31, sổ cái trong bộ nhớ của `ledger-core`).

use std::sync::{Arc, Mutex};

use axum::extract::{FromRef, Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use ledger_core::{AccountId, JournalEntry, Ledger, LedgerError, Money, Posting};
use serde::{Deserialize, Serialize};

use crate::auth::{AuthConfig, Authorized, Policy, Role};
use crate::problem::ApiError;

/// Trạng thái dùng chung của app.
#[derive(Clone)]
pub struct AppState {
    /// Sổ cái trong bộ nhớ. Khoá `std::sync::Mutex` không bao giờ giữ qua `.await`.
    pub ledger: Arc<Mutex<Ledger>>,
    /// Khoá và quy tắc kiểm token.
    pub auth: Arc<AuthConfig>,
}

// Cho phép extractor `AuthUser` lấy `Arc<AuthConfig>` từ state mà không biết `AppState`.
impl FromRef<AppState> for Arc<AuthConfig> {
    fn from_ref(state: &AppState) -> Self {
        state.auth.clone()
    }
}

impl AppState {
    /// State mới: sổ rỗng, khoá lab sinh tại chỗ.
    pub fn for_lab() -> AppState {
        AppState {
            ledger: Arc::new(Mutex::new(Ledger::new())),
            auth: Arc::new(AuthConfig::for_lab()),
        }
    }
}

/// Ghi bút toán: chỉ `finance`.
pub struct Writers;
impl Policy for Writers {
    const ALLOWED: &'static [Role] = &[Role::Finance];
}

/// Đọc số dư: `finance` và `cs`.
pub struct Readers;
impl Policy for Readers {
    const ALLOWED: &'static [Role] = &[Role::Finance, Role::Cs];
}

/// Body `POST /v1/entries`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostEntryBody {
    idempotency_key: String,
    postings: Vec<PostingBody>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostingBody {
    account: String,
    amount_dong: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EntryCreated {
    entry_id: String,
    replayed: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BalanceView {
    account: String,
    balance_dong: i64,
    as_of: String,
}

fn invalid(err: &LedgerError) -> ApiError {
    match err {
        LedgerError::IdempotencyConflict { .. } => ApiError::Conflict(err.to_string()),
        _ => ApiError::Unprocessable(err.to_string()),
    }
}

/// Ghi bút toán: chỉ `finance`. 201 khi tạo mới, 200 khi replay.
async fn post_entry(
    Authorized { user, .. }: Authorized<Writers>, // đứng trước `Json`: người không đủ quyền không buộc server parse body
    State(state): State<AppState>,
    Json(body): Json<PostEntryBody>,
) -> Result<(StatusCode, Json<EntryCreated>), ApiError> {
    let postings = body
        .postings
        .into_iter()
        .map(|p| {
            Ok(Posting::new(
                AccountId::new(&p.account)?,
                Money::from_dong(p.amount_dong),
            ))
        })
        .collect::<Result<Vec<_>, LedgerError>>()
        .map_err(|e| invalid(&e))?;
    let entry = JournalEntry::try_new(&body.idempotency_key, postings).map_err(|e| invalid(&e))?;
    let mut ledger = state
        .ledger
        .lock()
        .expect("mutex không bị poison trong lab");
    let posted = ledger.post(entry).map_err(|e| invalid(&e))?;
    let (id, replayed) = (posted.id.to_string(), posted.replayed);
    tracing::info!(user = %user.sub, entry = %id, replayed, "ghi bút toán");
    let status = if replayed {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };
    Ok((
        status,
        Json(EntryCreated {
            entry_id: id,
            replayed,
        }),
    ))
}

/// Tra số dư: `finance` và `cs`.
async fn get_balance(
    Authorized { user, .. }: Authorized<Readers>,
    State(state): State<AppState>,
    Path(account): Path<String>,
) -> Result<Json<BalanceView>, ApiError> {
    let id = AccountId::new(&account).map_err(|e| invalid(&e))?;
    let balance = state
        .ledger
        .lock()
        .expect("mutex không bị poison trong lab")
        .balance(&id)
        .map_err(|e| invalid(&e))?;
    tracing::info!(user = %user.sub, %account, "tra số dư");
    Ok(Json(BalanceView {
        account,
        balance_dong: balance.dong(),
        as_of: jiff::Timestamp::now().to_string(),
    }))
}

async fn healthz() -> &'static str {
    "ok" // không cần token: probe của hạ tầng không có danh tính
}

/// Các route nghiệp vụ, chưa gắn middleware.
pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/entries", post(post_entry))
        .route("/v1/accounts/{id}/balance", get(get_balance))
        .with_state(state)
}
