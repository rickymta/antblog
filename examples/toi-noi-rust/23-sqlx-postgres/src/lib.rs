//! Lab bài 23: sqlx với PostgreSQL cho sổ cái Tới Nơi.
//!
//! - [`db`]: đọc `TOI_NOI_PG_URL`, dựng pool, tạo và dọn schema riêng, chạy migration.
//! - [`store`]: ghi và đọc bút toán bằng `query!`/`query_as!` kiểm tra lúc biên dịch.
//! - [`mapping`]: đổi kiểu ở biên giữa PostgreSQL, `time`, `jiff` và `ledger-core`.

pub mod db;
pub mod mapping;
pub mod store;
