//! Một `tower::Service` viết tay: cho thấy handler async của axum cũng chỉ là Service.

use std::convert::Infallible;
use std::future::{Ready, ready};
use std::task::{Context, Poll};

use axum::Json;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::json;
use tower::Service;

/// Service kiểm tra sống: luôn trả 200 `{"status":"ok"}`, không đụng cơ sở dữ liệu.
#[derive(Debug, Clone, Copy, Default)]
pub struct Healthz;

impl Service<Request<Body>> for Healthz {
    type Response = Response;
    type Error = Infallible;
    type Future = Ready<Result<Response, Infallible>>;

    // Không có gì để chờ (không giới hạn đồng thời, không kết nối): luôn sẵn sàng.
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Infallible>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _request: Request<Body>) -> Self::Future {
        ready(Ok(
            (StatusCode::OK, Json(json!({ "status": "ok" }))).into_response()
        ))
    }
}
