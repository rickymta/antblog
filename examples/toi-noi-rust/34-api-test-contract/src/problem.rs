//! Lỗi `application/problem+json` theo RFC 9457: cùng hợp đồng với bài 32 (`type`, slug, `instance`,
//! `errors`, `traceId`, mã trạng thái). Thêm 401/403 của bài 33 và lỗi tài khoản của PostgreSQL.

use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use ledger_core::LedgerError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::store::PostError;

/// Tiền tố của `type`. RFC 9457 cho phép URI không mở được; miền `.example` chỉ để minh họa.
const TYPE_BASE: &str = "https://toi-noi.example/problems/";

/// Một lỗi theo trường, nằm trong thành phần mở rộng `errors`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct FieldError {
    /// Đường dẫn tới trường, ví dụ `postings[1].amountDong`.
    #[schema(example = "postings[1].amountDong")]
    pub field: String,
    /// Mã máy đọc được, ổn định: client rẽ nhánh theo mã này.
    #[schema(example = "zero_amount")]
    pub code: String,
    /// Câu mô tả cho người đọc.
    #[schema(example = "số tiền phải khác 0")]
    pub message: String,
}

/// Problem Details của RFC 9457 kèm hai thành phần mở rộng: `errors` và `traceId`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Problem {
    /// URI định danh loại lỗi.
    #[serde(rename = "type")]
    #[schema(example = "https://toi-noi.example/problems/unbalanced-entry")]
    pub type_uri: String,
    /// Tóm tắt ngắn, cùng một giá trị cho mọi lần xảy ra cùng loại lỗi.
    #[schema(example = "Bút toán lệch")]
    pub title: String,
    /// Mã HTTP, lặp lại trong body.
    #[schema(example = 422)]
    pub status: u16,
    /// Giải thích riêng cho lần xảy ra này.
    #[schema(example = "bút toán lệch, tổng bằng -5.100 VND")]
    pub detail: String,
    /// Đường dẫn của request gây lỗi.
    #[schema(example = "/v1/entries")]
    pub instance: String,
    /// Mọi lỗi theo trường (chỉ có khi là lỗi validation).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<FieldError>,
    /// Mã để tra log, sinh tại chỗ (32 hex, dạng W3C trace-id).
    #[schema(example = "4bf92f3577b34da6a3ce929d0e0e4736")]
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

    /// 401: không chứng minh được người gọi là ai. Mọi lý do cùng một thân (bài 33).
    pub fn unauthenticated(instance: &str) -> Problem {
        Problem::new(
            StatusCode::UNAUTHORIZED,
            "unauthenticated",
            "Chưa xác thực",
            "thiếu hoặc sai token",
            instance,
        )
    }

    /// 403: biết người gọi nhưng thiếu quyền.
    pub fn forbidden(scope: &str, instance: &str) -> Problem {
        Problem::new(
            StatusCode::FORBIDDEN,
            "forbidden",
            "Không đủ quyền",
            format!("cần quyền {scope}"),
            instance,
        )
    }

    /// 500: chi tiết hạ tầng chỉ vào log kèm `traceId`, không ra ngoài.
    pub fn internal(cause: &str, instance: &str) -> Problem {
        let problem = Problem::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "Lỗi nội bộ",
            "xem log với traceId",
            instance,
        );
        eprintln!("lỗi nội bộ traceId={}: {cause}", problem.trace_id);
        problem
    }
}

impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let mut response = (status, Json(self)).into_response();
        // `Json` đặt `application/json`; RFC 9457 yêu cầu media type riêng.
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        if status == StatusCode::UNAUTHORIZED {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        response
    }
}

/// Kiểu lỗi trả về của handler; đóng hộp vì `Problem` lớn (clippy `result_large_err`, như bài 32).
#[derive(Debug)]
pub struct ApiError(Box<Problem>);

impl From<Problem> for ApiError {
    fn from(problem: Problem) -> ApiError {
        ApiError(Box::new(problem))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        self.0.into_response()
    }
}

/// Bảng lỗi miền của bài 32: một `match` vét cạn, không có nhánh `_`.
pub fn problem_from_ledger(err: &LedgerError, instance: &str) -> Problem {
    let unprocessable = |slug: &str, title: &str| {
        Problem::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            slug,
            title,
            err.to_string(),
            instance,
        )
    };
    match err {
        LedgerError::EmptyAccountId => unprocessable("empty-account-id", "Mã tài khoản rỗng"),
        LedgerError::EmptyIdempotencyKey => {
            unprocessable("empty-idempotency-key", "Idempotency key rỗng")
        }
        LedgerError::TooFewPostings { .. } => {
            unprocessable("too-few-postings", "Bút toán thiếu posting")
        }
        LedgerError::Unbalanced { .. } => unprocessable("unbalanced-entry", "Bút toán lệch"),
        LedgerError::Overflow => unprocessable("amount-overflow", "Tổng tiền vượt giới hạn"),
        LedgerError::IdempotencyConflict { .. } => Problem::new(
            StatusCode::CONFLICT,
            "idempotency-conflict",
            "Idempotency key đã dùng cho nội dung khác",
            err.to_string(),
            instance,
        ),
        LedgerError::UnknownEntry { .. } => Problem::new(
            StatusCode::NOT_FOUND,
            "unknown-entry",
            "Không có bút toán này",
            err.to_string(),
            instance,
        ),
        LedgerError::InvalidEntryId { .. } => Problem::new(
            StatusCode::BAD_REQUEST,
            "invalid-entry-id",
            "Mã bút toán sai dạng",
            err.to_string(),
            instance,
        ),
        LedgerError::InvalidFee { .. } => Problem::internal(&err.to_string(), instance),
    }
}

/// Lỗi khi ghi xuống PostgreSQL. Bài 32 dùng sổ trong bộ nhớ nên không có ca tài khoản chưa mở;
/// ở đây bảng `accounts` có khoá ngoại nên `POST` có thêm 422 `unknown-account`.
pub fn problem_from_post(err: &PostError, instance: &str) -> Problem {
    match err {
        PostError::Ledger(e) => problem_from_ledger(e, instance),
        PostError::UnknownAccount(account) => Problem::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "unknown-account",
            "Tài khoản chưa tồn tại",
            format!("tài khoản {account} chưa tồn tại"),
            instance,
        ),
        PostError::Db(e) => Problem::internal(&e.to_string(), instance),
    }
}

/// Giữ nguyên mã trạng thái axum đã chọn (400, 415, 422, 413), chỉ đổi cách trình bày.
pub fn problem_from_rejection(rejection: &JsonRejection, instance: &str) -> Problem {
    let (slug, title) = match rejection {
        JsonRejection::JsonSyntaxError(_) => ("malformed-json", "JSON không đúng cú pháp"),
        JsonRejection::JsonDataError(_) => ("invalid-json-body", "JSON không đúng hình dạng"),
        JsonRejection::MissingJsonContentType(_) => (
            "unsupported-media-type",
            "Cần Content-Type: application/json",
        ),
        _ => ("unreadable-body", "Không đọc được body"),
    };
    Problem::new(
        rejection.status(),
        slug,
        title,
        rejection.body_text(),
        instance,
    )
}

/// Extractor `Path` từ chối (ví dụ `%FF` không giải mã ra UTF-8): cũng phải là Problem.
pub fn problem_from_path(rejection: &PathRejection, instance: &str) -> Problem {
    Problem::new(
        rejection.status(),
        "invalid-account-id",
        "Mã tài khoản sai dạng",
        rejection.body_text(),
        instance,
    )
}
