//! Adapter PostgreSQL (sqlx) cho `LedgerRepository`, cùng bộ hạ tầng schema riêng cho lab.

pub mod harness;
pub mod pg;

pub use pg::{PgRepository, PgUow};
