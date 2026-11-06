//! Lab bài 34: API ledger với axum, test không cần socket (`oneshot`), test với PostgreSQL thật,
//! snapshot OpenAPI và hợp đồng JSON dùng chung với client C#.
//!
//! Đây là bản thu nhỏ của `ledger-api` (bài 31-33): đủ route, auth và lỗi để có thứ mà test kiểm.

pub mod api;
pub mod auth;
pub mod problem;
pub mod rawhttp;
pub mod store;
pub mod testkit;
pub mod validate;
