//! Miền của `ledger-service`. Chỉ có luật nghiệp vụ thuần: dựng bút toán hai bước, hold, hoàn tiền.
//!
//! Crate này không biết PostgreSQL, Kafka, HTTP hay tokio. Muốn thêm `use tokio` ở đây thì phải
//! sửa `Cargo.toml`, và bước kiểm kiến trúc của `ledger-service --verify` sẽ đỏ.

pub mod chart;
pub mod clock;
pub mod hold;
pub mod refund;
pub mod trip;

pub use ledger_core::{
    AccountId, FeePolicy, JournalEntry, LedgerError, Money, Posting, default_policy, fare_for_km,
};
