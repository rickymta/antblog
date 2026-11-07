//! Lab bài 56: đổi schema không dừng sổ (expand/contract), feature flag cho quy tắc phí mới,
//! `policy_version` trong sự kiện để replay đúng số cũ, rollback và cấu hình kiểu an toàn.

pub mod app;
pub mod config;
pub mod flags;
pub mod lab;
pub mod migrate;
pub mod policy;
pub mod replay;
pub mod scenarios;
