//! Lab bài 31: HTTP với axum cho sổ cái Tới Nơi.
//!
//! - [`app`]: `Router`, handler và DTO của hai endpoint.
//! - [`error`]: `AppError` và `IntoResponse` (JSON lỗi đơn giản, Problem Details là bài 32).
//! - [`store`]: kho PostgreSQL tối thiểu (rút gọn từ adapter bài 26).
//! - [`lab_db`]: schema `lab_<uuid>` riêng cho mỗi lần chạy.
//! - [`healthz`]: một `tower::Service` viết tay, gắn vào router bằng `route_service`.
//! - [`scenario`]: chuỗi request thật qua TCP dùng cho `cargo run` và `--verify`.

pub mod app;
pub mod error;
pub mod healthz;
pub mod lab_db;
pub mod scenario;
pub mod store;
