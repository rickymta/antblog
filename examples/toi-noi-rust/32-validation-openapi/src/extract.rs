//! Extractor `ApiJson<T>`: bọc `axum::Json<T>` để mọi lỗi đọc body cũng ra Problem Details.

use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{FromRequest, OriginalUri, Request};
use serde::de::DeserializeOwned;

use crate::problem::{ApiError, Problem};

/// Giống `axum::Json<T>` nhưng `Rejection` là [`Problem`].
#[derive(Debug)]
pub struct ApiJson<T>(pub T);

/// Đường dẫn gốc của request (trước khi `nest("/v1", …)` cắt tiền tố).
pub fn instance_of(req: &Request) -> String {
    req.extensions()
        .get::<OriginalUri>()
        .map_or_else(|| req.uri().path().to_string(), |o| o.0.path().to_string())
}

impl<S, T> FromRequest<S> for ApiJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, ApiError> {
        let instance = instance_of(&req);
        match axum::Json::<T>::from_request(req, state).await {
            Ok(axum::Json(value)) => Ok(ApiJson(value)),
            Err(rejection) => Err(problem_from_rejection(&rejection, &instance).into()),
        }
    }
}

/// Mỗi biến thể của `JsonRejection` có mã HTTP riêng (axum đã chọn); ta giữ nguyên mã
/// đó và chỉ đổi cách trình bày sang Problem Details.
fn problem_from_rejection(rejection: &JsonRejection, instance: &str) -> Problem {
    let (slug, title) = match rejection {
        JsonRejection::JsonSyntaxError(_) => ("malformed-json", "JSON không đúng cú pháp"),
        JsonRejection::JsonDataError(_) => ("invalid-json-body", "JSON không đúng hình dạng"),
        JsonRejection::MissingJsonContentType(_) => (
            "unsupported-media-type",
            "Cần Content-Type: application/json",
        ),
        _ => ("unreadable-body", "Không đọc được body"),
    };
    // Mã trạng thái do axum chọn: 400, 415, 422 hoặc 413. Cùng bảng mã với bài 31.
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
