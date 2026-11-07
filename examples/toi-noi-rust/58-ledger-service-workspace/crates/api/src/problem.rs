//! Lỗi `application/problem+json` theo RFC 9457 (bài 32).

use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

/// Tiền tố của `type`. RFC 9457 cho phép URI không mở được; miền `.example` chỉ để minh họa.
const TYPE_BASE: &str = "https://toi-noi.example/problems/";

/// Một lỗi theo trường, nằm trong thành phần mở rộng `errors`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldError {
    /// Đường dẫn tới trường, ví dụ `postings[1].amountDong`.
    pub field: String,
    /// Mã máy đọc được, ổn định: client rẽ nhánh theo mã này.
    pub code: String,
    /// Câu mô tả cho người đọc.
    pub message: String,
}

/// Problem Details của RFC 9457 kèm hai thành phần mở rộng: `errors` và `traceId`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Problem {
    /// URI định danh loại lỗi.
    #[serde(rename = "type")]
    pub type_uri: String,
    /// Tóm tắt ngắn, cùng một giá trị cho mọi lần xảy ra cùng loại lỗi.
    pub title: String,
    /// Mã HTTP, lặp lại trong body.
    pub status: u16,
    /// Giải thích riêng cho lần xảy ra này.
    pub detail: String,
    /// Đường dẫn của request gây lỗi.
    pub instance: String,
    /// Mọi lỗi theo trường (chỉ có khi là lỗi validation).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<FieldError>,
    /// Mã để tra log (32 hex).
    pub trace_id: String,
}

impl Problem {
    /// Dựng lỗi: `slug` thành đuôi của `type`; `traceId` sinh mới.
    pub fn new(
        status: StatusCode,
        slug: &str,
        title: &str,
        detail: impl Into<String>,
        instance: &str,
    ) -> Problem {
        Problem {
            type_uri: format!("{TYPE_BASE}{slug}"),
            title: title.to_string(),
            status: status.as_u16(),
            detail: detail.into(),
            instance: instance.to_string(),
            errors: Vec::new(),
            trace_id: uuid::Uuid::new_v4().simple().to_string(),
        }
    }

    /// Gắn danh sách lỗi theo trường.
    #[must_use]
    pub fn with_errors(mut self, errors: Vec<FieldError>) -> Problem {
        self.errors = errors;
        self
    }
}

impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let unauthenticated = status == StatusCode::UNAUTHORIZED;
        let mut response = (status, Json(self)).into_response();
        let headers = response.headers_mut();
        // `Json` đặt `application/json`; RFC 9457 yêu cầu media type riêng.
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        if unauthenticated {
            headers.insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static("Bearer realm=\"ledger\""),
            );
        }
        response
    }
}

/// Kiểu lỗi trả về của handler. `Problem` khá nặng; đóng hộp để mọi đường thành công
/// không phải mang theo chỗ đó (clippy `result_large_err`).
#[derive(Debug)]
pub struct ApiError(Box<Problem>);

impl From<Problem> for ApiError {
    fn from(problem: Problem) -> ApiError {
        ApiError(Box::new(problem))
    }
}

impl std::ops::Deref for ApiError {
    type Target = Problem;

    fn deref(&self) -> &Problem {
        &self.0
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        self.0.into_response()
    }
}
