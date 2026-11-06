//! Lỗi API dạng `application/problem+json` (rút gọn từ bài 32: chỉ bốn trường cốt lõi).

use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::auth::Role;

/// Nội dung một Problem Details.
#[derive(Debug, Serialize)]
pub struct Problem {
    /// URI định danh loại lỗi (ở đây là đường dẫn tương đối, đủ cho lab).
    #[serde(rename = "type")]
    pub kind: &'static str,
    /// Tiêu đề ngắn, không đổi giữa các lần lỗi cùng loại.
    pub title: &'static str,
    /// Mã HTTP lặp lại trong body.
    pub status: u16,
    /// Mô tả riêng cho lần lỗi này. Không bao giờ chứa token hay lý do chi tiết của việc từ chối.
    pub detail: String,
}

/// Mọi lỗi handler và extractor có thể trả ra.
#[derive(Debug)]
pub enum ApiError {
    /// Thiếu hoặc sai token: 401. Lý do cụ thể chỉ vào log, không vào response.
    Unauthenticated,
    /// Token đúng nhưng vai trò không đủ: 403.
    Forbidden {
        /// Vai trò tối thiểu được phép làm việc này.
        needs: &'static [Role],
    },
    /// Bút toán không hợp lệ về nghiệp vụ (lệch tổng, thiếu posting): 422.
    Unprocessable(String),
    /// Cùng `idempotencyKey` nhưng nội dung khác: 409.
    Conflict(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, kind, title, detail) = match &self {
            ApiError::Unauthenticated => (
                StatusCode::UNAUTHORIZED,
                "https://toi-noi.example/problems/unauthenticated",
                "Chưa xác thực",
                "thiếu hoặc không hợp lệ: Authorization Bearer token".to_string(),
            ),
            ApiError::Forbidden { needs } => (
                StatusCode::FORBIDDEN,
                "https://toi-noi.example/problems/forbidden",
                "Không đủ quyền",
                format!("cần một trong các vai trò: {needs:?}"),
            ),
            ApiError::Unprocessable(d) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "https://toi-noi.example/problems/invalid-entry",
                "Bút toán không hợp lệ",
                d.clone(),
            ),
            ApiError::Conflict(d) => (
                StatusCode::CONFLICT,
                "https://toi-noi.example/problems/idempotency-conflict",
                "Xung đột idempotency key",
                d.clone(),
            ),
        };
        let problem = Problem {
            kind,
            title,
            status: status.as_u16(),
            detail,
        };
        let mut response = (status, Json(problem)).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        if matches!(self, ApiError::Unauthenticated) {
            response.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static("Bearer realm=\"ledger\""),
            );
        }
        response
    }
}
