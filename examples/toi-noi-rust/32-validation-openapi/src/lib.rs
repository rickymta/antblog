//! Lab bài 32: validation, Problem Details (RFC 9457) và OpenAPI cho API sổ cái `/v1`.

pub mod dto;
pub mod error_map;
pub mod extract;
pub mod handlers;
pub mod openapi;
pub mod probe;
pub mod problem;
pub mod validate;
pub mod verify;

use axum::Router;
use axum::routing::{get, post};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::AppState;
use crate::openapi::ApiDoc;

/// Đường dẫn của tài liệu OpenAPI dạng JSON.
pub const OPENAPI_PATH: &str = "/api-docs/openapi.json";
/// Đường dẫn của giao diện Swagger UI.
pub const SWAGGER_PATH: &str = "/swagger-ui";

/// Router đầy đủ: nhóm `/v1`, tài liệu OpenAPI, Swagger UI và các lỗi 404/405 dạng Problem.
pub fn app(state: AppState) -> Router {
    // Mọi route của phiên bản 1 nằm dưới một tiền tố; v2 sau này là một `nest` khác.
    let v1 = Router::new()
        .route("/entries", post(handlers::post_entry))
        .route("/accounts/{id}/balance", get(handlers::get_balance));
    Router::new()
        .nest("/v1", v1)
        .merge(SwaggerUi::new(SWAGGER_PATH).url(OPENAPI_PATH, ApiDoc::openapi()))
        .fallback(handlers::route_not_found)
        .method_not_allowed_fallback(handlers::method_not_allowed)
        .with_state(state)
}
