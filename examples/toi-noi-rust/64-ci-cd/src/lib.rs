//! Lab bài 64: crate sổ cái nhỏ để pipeline CI có thứ để chạy.
//! - [`store`]: ghi bút toán vào PostgreSQL bằng `query!` (CI biên dịch với `SQLX_OFFLINE=true`).
//! - [`trip`]: hai bút toán chuẩn của chuyến `TN-0001` (capture rồi chia).
//! - [`testdb`]: schema `lab_<uuid>` riêng cho mỗi ca, dọn cả khi test panic.
//! - [`pipeline`]: kiểm tra tĩnh các workflow mẫu và `deny.toml`.

pub mod pipeline;
pub mod store;
pub mod testdb;
pub mod trip;

pub use store::{MIGRATOR, PgLedger, PostError, Posted};
