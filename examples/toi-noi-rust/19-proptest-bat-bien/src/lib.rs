//! Thư viện sổ cái rút gọn của Tới Nơi (bài 19, phần P02).
//!
//! Giữ nguyên mô hình của bài 10 (`Money`, `AccountId`, `Posting`, `JournalEntry`,
//! `Ledger`) và thêm ba thứ phục vụ property test: `split` (chia tiền không mất đồng),
//! `strategies` (bộ sinh dữ liệu ngẫu nhiên) và `shrink_demo` (tìm phản ví dụ nhỏ nhất).

pub mod account;
pub mod entry;
pub mod error;
pub mod ledger;
pub mod money;
pub mod shrink_demo;
pub mod split;
pub mod strategies;

pub use account::AccountId;
pub use entry::{JournalEntry, Posting};
pub use error::LedgerError;
pub use ledger::{Ledger, Posted};
pub use money::Money;
pub use split::{SplitError, split};
