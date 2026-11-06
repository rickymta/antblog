//! Lab bài 33: middleware với tower cho API sổ cái.

pub mod api;
pub mod auth;
pub mod capture;
pub mod client;
pub mod limits;
pub mod order;
pub mod problem;
pub mod stack;
pub mod verify;

use axum::Router;

pub use api::AppState;
pub use stack::Settings;

/// App hoàn chỉnh: route nghiệp vụ + chồng middleware.
pub fn build_app(state: AppState, settings: &Settings) -> Router {
    stack::apply(api::routes(state), settings)
}
