//! Thư viện sổ cái rút gọn của Tới Nơi (bài 13, phần P02).
//!
//! Mở rộng lab bài 10: chiều tiền (`Flow`), chart of accounts (`AccountKind`),
//! `JournalEntry` chỉ tạo được khi cân (`try_new`, `JournalEntryBuilder`),
//! bút toán đảo (`Ledger::reverse`) và số dư là projection (`Ledger::balances`).

// Mỗi `pub mod` tương ứng một file trong src/ (edition 2024 không cần mod.rs).
pub mod account;
pub mod chart;
pub mod entry;
pub mod entry_id;
pub mod error;
pub mod ledger;
pub mod money;
pub mod naive;

// Re-export: người dùng thư viện viết `ledger_lab::Money` thay vì `ledger_lab::money::Money`.
pub use account::AccountId;
pub use chart::{AccountKind, kind_of};
pub use entry::{Flow, JournalEntry, JournalEntryBuilder, Posting};
pub use entry_id::EntryId;
pub use error::LedgerError;
pub use ledger::{Ledger, Posted};
pub use money::Money;
