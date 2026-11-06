//! Thư viện sổ cái rút gọn của Tới Nơi (bài 18, phần P02): thêm hợp đồng JSON bằng serde.
//!
//! Gom mô hình của bài 03–09: `Money(i64)` có checked arithmetic, `AccountId`,
//! `Posting`, `JournalEntry::try_new` (phải cân, không tràn số), `LedgerError`,
//! `Ledger::post` idempotent và `Ledger::balance`.

// Mỗi `pub mod` tương ứng một file trong src/ (edition 2024 không cần mod.rs).
pub mod account;
pub mod entry;
pub mod error;
pub mod event;
pub mod ledger;
pub mod money;

// Re-export: người dùng thư viện viết `ledger_lab::Money` thay vì `ledger_lab::money::Money`.
pub use account::AccountId;
pub use entry::{JournalEntry, Posting};
pub use error::LedgerError;
pub use event::{EntryPosted, EntryReversed, LedgerEvent, SchemaV1};
pub use ledger::{Ledger, Posted};
pub use money::Money;
