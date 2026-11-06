//! Giới hạn tải bằng `tower::limit`: concurrency (đơn giản) và rate limit (cần `Buffer`).

use std::time::Duration;

use axum::BoxError;
use axum::Router;
use axum::error_handling::HandleErrorLayer;
use axum::http::StatusCode;
use tower::ServiceBuilder;
use tower::buffer::BufferLayer;
use tower::limit::{GlobalConcurrencyLimitLayer, RateLimitLayer};
use tower::load_shed::LoadShedLayer;
use tower::load_shed::error::Overloaded;

async fn to_503(err: BoxError) -> (StatusCode, &'static str) {
    if err.is::<Overloaded>() {
        (StatusCode::SERVICE_UNAVAILABLE, "vượt hạn mức, thử lại sau")
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "lỗi nội bộ")
    }
}

/// Bẫy: `Router::layer` gọi layer cho TỪNG route, nên mỗi route một worker và một bucket riêng.
///
/// `RateLimit` giữ một bộ hẹn giờ nên không `Clone`; `Router::layer` cần `Clone`.
/// Lỗi E0277 nếu viết thẳng `router.layer(RateLimitLayer::new(num, per))`.
/// `Buffer` đặt một worker giữ service thật, còn phía ngoài chỉ giữ một tay cầm `Clone`.
pub fn rate_limited_per_route(router: Router, num: u64, per: Duration) -> Router {
    router.with_state(()).layer(
        ServiceBuilder::new()
            .layer(HandleErrorLayer::new(to_503)) // Buffer và LoadShed trả BoxError
            .layer(BufferLayer::new(64)) // tay cầm Clone tới worker
            .layer(LoadShedLayer::new()) // hết hạn mức: từ chối thay vì xếp hàng chờ
            .layer(RateLimitLayer::new(num, per)),
    )
}

/// Rate limit TOÀN CỤC: tối đa `num` request mỗi `per`, cho mọi route và mọi client cộng lại.
/// Bọc cả `Router` thành MỘT service rồi gắn làm fallback, nên chỉ có một worker và một bucket.
pub fn rate_limited(router: Router, num: u64, per: Duration) -> Router {
    let service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(to_503))
        .layer(BufferLayer::new(64))
        .layer(LoadShedLayer::new())
        .layer(RateLimitLayer::new(num, per))
        .service(router.with_state(()));
    Router::new().fallback_service(service)
}

/// Concurrency limit thuần: request thứ `n + 1` XẾP HÀNG chờ chỗ, không bị từ chối.
/// `ConcurrencyLimit` chia sẻ semaphore qua `Arc` nên `Clone` được, không cần `Buffer`.
/// Dùng `GlobalConcurrencyLimitLayer`: `ConcurrencyLimitLayer` tạo một semaphore riêng cho TỪNG route.
pub fn concurrency_queued(router: Router, n: usize) -> Router {
    router
        .with_state(())
        .layer(GlobalConcurrencyLimitLayer::new(n))
}
