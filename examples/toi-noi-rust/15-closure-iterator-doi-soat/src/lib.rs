//! Thư viện sổ cái rút gọn của Tới Nơi (bài 15, phần P02).
//!
//! Giữ mô hình của bài 10 (`Money`, `AccountId`, `JournalEntry`, `Ledger`) và thêm
//! hai module cho đối soát trong bộ nhớ: `statement` đọc sao kê VíXanh giả,
//! `reconcile` so sổ cái với sao kê bằng closure và iterator adapter.

pub mod account;
pub mod entry;
pub mod error;
pub mod ledger;
pub mod money;
pub mod reconcile;
pub mod statement;

pub use account::AccountId;
pub use entry::{JournalEntry, Posting};
pub use error::LedgerError;
pub use ledger::{Ledger, Posted};
pub use money::Money;
pub use reconcile::{Finding, LedgerRecord, PspRecord, Report};
pub use statement::StatementError;
