//! Chồng middleware thật của service: một `ServiceBuilder`, đọc từ trên xuống là thứ tự request đi qua.

use std::time::Duration;

use axum::BoxError;
use axum::Router;
use axum::error_handling::HandleErrorLayer;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE, HeaderName};
use axum::http::{HeaderValue, Method, Request, StatusCode};
use tower::ServiceBuilder;
use tower::limit::GlobalConcurrencyLimitLayer;
use tower::load_shed::LoadShedLayer;
use tower::load_shed::error::Overloaded;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::sensitive_headers::SetSensitiveRequestHeadersLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing::Level;

/// Tên header mang request id.
pub const REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

/// Mã trả khi quá hạn. `tower-http` mặc định 408; ta chọn 504 và nói lý do trong bài.
pub const TIMEOUT_STATUS: StatusCode = StatusCode::GATEWAY_TIMEOUT;

/// Thông số của chồng middleware.
#[derive(Debug, Clone)]
pub struct Settings {
    /// Thời gian tối đa cho một request, tính từ lúc qua lớp timeout.
    pub timeout: Duration,
    /// Số request chạy đồng thời tối đa trong cả service.
    pub max_in_flight: usize,
    /// Origin trình duyệt được phép gọi thẳng; `None` nghĩa là không gắn CORS.
    pub cors_origin: Option<String>,
    /// Đưa toàn bộ header request vào span (chỉ để chứng minh `Authorization` bị che).
    pub trace_headers: bool,
    /// Đánh dấu `Authorization` là nhạy cảm để TraceLayer không in giá trị.
    pub hide_authorization: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            timeout: Duration::from_secs(2),
            max_in_flight: 64,
            cors_origin: None,
            trace_headers: false,
            hide_authorization: true,
        }
    }
}

// Lỗi duy nhất có thể xuất hiện là Overloaded của LoadShed; đổi nó thành 503 để service vẫn Infallible.
async fn overloaded(err: BoxError) -> (StatusCode, &'static str) {
    if err.is::<Overloaded>() {
        (StatusCode::SERVICE_UNAVAILABLE, "quá tải, thử lại sau")
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "lỗi nội bộ")
    }
}

/// Gắn chồng middleware vào router.
pub fn apply(router: Router, settings: &Settings) -> Router {
    let sensitive: Vec<HeaderName> = if settings.hide_authorization {
        vec![AUTHORIZATION]
    } else {
        vec![]
    };
    let cors = settings.cors_origin.as_deref().map(|origin| {
        CorsLayer::new()
            .allow_origin(AllowOrigin::list([
                HeaderValue::from_str(origin).expect("origin hợp lệ")
            ]))
            .allow_methods([Method::GET, Method::POST])
            .allow_headers([AUTHORIZATION, CONTENT_TYPE])
    });
    let log_headers = settings.trace_headers;
    let make_span = move |req: &Request<_>| {
        let id = req
            .headers()
            .get(&REQUEST_ID)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("-");
        // Chỉ ghi path, không ghi query: query là nơi người ta hay lỡ tay đặt token.
        if log_headers {
            tracing::info_span!("http", method = %req.method(), path = %req.uri().path(), request_id = %id, headers = ?req.headers())
        } else {
            tracing::info_span!("http", method = %req.method(), path = %req.uri().path(), request_id = %id)
        }
    };

    let stack = ServiceBuilder::new()
        .layer(SetSensitiveRequestHeadersLayer::new(sensitive)) // 1. che Authorization trước mọi lớp ghi log
        .layer(SetRequestIdLayer::new(REQUEST_ID, MakeRequestUuid)) // 2. gán id trước khi mở span
        .layer(
            TraceLayer::new_for_http() // 3. span chứa request id; thấy cả 401, 504, 503 bên dưới
                .make_span_with(make_span)
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(PropagateRequestIdLayer::new(REQUEST_ID)) // 4. chép id sang response, kể cả response lỗi
        .option_layer(cors) // 5. CORS (nếu cấu hình), trả lời preflight trước khi tới handler
        .layer(TimeoutLayer::with_status_code(
            TIMEOUT_STATUS,
            settings.timeout,
        )) // 6. hạn chót cho mọi thứ bên dưới
        .layer(HandleErrorLayer::new(overloaded)) // 7. đổi lỗi Overloaded thành 503
        .layer(LoadShedLayer::new()) // 8. hết chỗ thì từ chối ngay, không xếp hàng
        .layer(GlobalConcurrencyLimitLayer::new(settings.max_in_flight)); // 9. một semaphore cho mọi route
    // `with_state(())` dựng route MỘT lần. Router chưa có nó thì layer được gọi lại mỗi request (`oneshot`)
    // hoặc mỗi kết nối (`axum::serve` trực tiếp); layer tự tạo trạng thái khi đó mất tác dụng.
    router.with_state(()).layer(stack)
}
