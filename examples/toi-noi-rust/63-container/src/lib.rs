//! Lab bài 63: `ledger-service` nhỏ để đóng gói vào container.
//!
//! Service chỉ có ba việc: trả sống/sẵn sàng cho orchestrator, tính cước cho một quãng đường (số tiền
//! `i64` đồng, không float) và tắt có trật tự khi nhận SIGTERM. PostgreSQL là tuỳ chọn: có cấu hình thì
//! `/readyz` kiểm thật bằng `SELECT 1`, không có thì service chạy ở chế độ không DB.

pub mod config;
pub mod fare;
pub mod http;
pub mod probe;
pub mod server;

#[cfg(feature = "lab")]
pub mod docker;
#[cfg(feature = "lab")]
pub mod lab;

/// Nhãn bản build. Bài tập đổi đúng một dòng này để đo cache của `cargo-chef`.
pub const BUILD_TAG: &str = "lab-63";
