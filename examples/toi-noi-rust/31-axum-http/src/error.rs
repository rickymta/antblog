//! `AppError`: một kiểu lỗi cho mọi handler, và `IntoResponse` đổi nó thành HTTP.
//!
//! Bài 32 thay JSON lỗi này bằng `application/problem+json`; bảng ánh xạ status giữ nguyên.

use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use ledger_core::LedgerError;
use serde::Serialize;
use thiserror::Error;

use crate::store::StoreError;

/// Mọi lý do một request thất bại.
#[derive(Debug, Error)]
pub enum AppError {
    /// Body không đọc được thành JSON đúng hình dạng, hoặc thiếu `Content-Type`.
    #[error("{message}")]
    Body {
        /// Status do axum chọn theo loại lỗi: 400, 413, 415 hoặc 422.
        status: StatusCode,
        /// Thông điệp của extractor.
        message: String,
    },
    /// Phần `Path` hợp lệ về cú pháp HTTP nhưng không phải mã tài khoản.
    #[error("mã tài khoản không hợp lệ: {0}")]
    BadAccountId(LedgerError),
    /// Extractor `Path` từ chối, ví dụ `%FF` giải mã ra byte không phải UTF-8.
    #[error("{message}")]
    BadPath {
        /// Status do axum chọn.
        status: StatusCode,
        /// Thông điệp của extractor.
        message: String,
    },
    /// Tài khoản chưa mở (chỉ với GET số dư).
    #[error("không có tài khoản {0}")]
    AccountNotFound(String),
    /// Không route nào khớp.
    #[error("không có route {method} {path}")]
    RouteNotFound {
        /// Phương thức HTTP của request.
        method: String,
        /// Đường dẫn của request.
        path: String,
    },
    /// Lỗi từ kho: nghiệp vụ hoặc hạ tầng.
    #[error(transparent)]
    Store(#[from] StoreError),
}

// Lỗi dựng bút toán từ body (không cân, thiếu posting...) là lỗi nghiệp vụ: đi cùng nhánh với lỗi của kho.
impl From<LedgerError> for AppError {
    fn from(error: LedgerError) -> AppError {
        AppError::Store(StoreError::Domain(error))
    }
}

// Extractor `Json` báo lỗi bằng `JsonRejection`; ta đổi sang `AppError` để chỉ có một dạng lỗi ra ngoài.
impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> AppError {
        // Giữ mã axum đã chọn: cú pháp hỏng 400, sai hình dạng 422, thiếu Content-Type 415, quá 2 MB 413.
        AppError::Body {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

// `Path` cũng có rejection riêng; không bắt thì client nhận `text/plain`.
impl From<PathRejection> for AppError {
    fn from(rejection: PathRejection) -> AppError {
        AppError::BadPath {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

/// Thân JSON của mọi lỗi: `{"code": "...", "message": "..."}`.
#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

impl AppError {
    // (status, code ổn định cho máy đọc, thông điệp cho người đọc).
    fn parts(&self) -> (StatusCode, &'static str, String) {
        match self {
            AppError::Body { status, message } => (*status, "bad_body", message.clone()),
            AppError::BadAccountId(_) => {
                (StatusCode::BAD_REQUEST, "bad_account_id", self.to_string())
            }
            AppError::BadPath { status, message } => (*status, "bad_account_id", message.clone()),
            AppError::AccountNotFound(_) => {
                (StatusCode::NOT_FOUND, "account_not_found", self.to_string())
            }
            AppError::RouteNotFound { .. } => {
                (StatusCode::NOT_FOUND, "route_not_found", self.to_string())
            }
            AppError::Store(StoreError::Domain(e)) => match e {
                LedgerError::IdempotencyConflict { .. } => {
                    (StatusCode::CONFLICT, "idempotency_conflict", e.to_string())
                }
                LedgerError::Unbalanced { .. } => (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "unbalanced",
                    e.to_string(),
                ),
                LedgerError::TooFewPostings { .. } => (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "too_few_postings",
                    e.to_string(),
                ),
                LedgerError::EmptyIdempotencyKey => (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "empty_idempotency_key",
                    e.to_string(),
                ),
                LedgerError::EmptyAccountId => (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "empty_account_id",
                    e.to_string(),
                ),
                LedgerError::Overflow => {
                    (StatusCode::UNPROCESSABLE_ENTITY, "overflow", e.to_string())
                }
                LedgerError::UnknownEntry { .. } => {
                    (StatusCode::NOT_FOUND, "entry_not_found", e.to_string())
                }
                LedgerError::InvalidEntryId { .. } => {
                    (StatusCode::BAD_REQUEST, "invalid_entry_id", e.to_string())
                }
                // Lỗi của chính sách phí, không phải của client.
                LedgerError::InvalidFee { .. } => internal(),
            },
            AppError::Store(StoreError::UnknownAccount(_)) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "unknown_account",
                self.to_string(),
            ),
            AppError::Store(StoreError::ZeroAmount(_)) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "zero_amount",
                self.to_string(),
            ),
            AppError::Store(StoreError::Storage(_)) => internal(),
        }
    }
}

// Lỗi nội bộ: thông điệp chung chung, chi tiết chỉ đi vào log của server.
fn internal() -> (StatusCode, &'static str, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal",
        "lỗi nội bộ".to_string(),
    )
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = self.parts();
        if status.is_server_error() {
            // Lab dùng eprintln; bài 33 đưa việc này vào tracing/TraceLayer.
            eprintln!("lỗi 5xx: {self:?}");
        }
        (status, Json(ErrorBody { code, message })).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use ledger_core::{AccountId, Money};

    async fn status_and_body(error: AppError) -> (StatusCode, serde_json::Value) {
        let response = error.into_response();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("đọc body");
        (
            status,
            serde_json::from_slice(&bytes).expect("body là JSON"),
        )
    }

    #[tokio::test]
    async fn status_table() {
        let key = || "k".to_string();
        let account = || AccountId::new("a").expect("hợp lệ");
        let cases: Vec<(AppError, StatusCode, &str)> = vec![
            (
                LedgerError::IdempotencyConflict { key: key() }.into(),
                StatusCode::CONFLICT,
                "idempotency_conflict",
            ),
            (
                LedgerError::Unbalanced {
                    sum: Money::from_dong(-100),
                }
                .into(),
                StatusCode::UNPROCESSABLE_ENTITY,
                "unbalanced",
            ),
            (
                StoreError::UnknownAccount(account()).into(),
                StatusCode::UNPROCESSABLE_ENTITY,
                "unknown_account",
            ),
            (
                StoreError::ZeroAmount(account()).into(),
                StatusCode::UNPROCESSABLE_ENTITY,
                "zero_amount",
            ),
            (
                AppError::AccountNotFound("x".into()),
                StatusCode::NOT_FOUND,
                "account_not_found",
            ),
            (
                AppError::BadAccountId(LedgerError::EmptyAccountId),
                StatusCode::BAD_REQUEST,
                "bad_account_id",
            ),
        ];
        for (error, status, code) in cases {
            let (got_status, body) = status_and_body(error).await;
            assert_eq!(got_status, status, "{code}");
            assert_eq!(body["code"], code);
        }
    }

    // Lỗi hạ tầng không được rò chi tiết ra response.
    #[tokio::test]
    async fn storage_error_is_500_without_details() {
        let error = AppError::from(StoreError::from(sqlx::Error::PoolTimedOut));
        let (status, body) = status_and_body(error).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body["code"], "internal");
        assert_eq!(body["message"], "lỗi nội bộ");
        assert!(!body.to_string().contains("pool"));
    }
}
