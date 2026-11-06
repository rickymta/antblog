//! Thư viện sổ cái rút gọn của Tới Nơi (bài 12, phần P02).
//!
//! Từ bài 10, ba kiểu giá trị được làm chặt hơn: `Money<C>` mang loại tiền ở kiểu và có
//! `Add`/`Sub` trả `Result`, `AccountId` parse có kiểm tra, `EntryId` dạng `JE-000001`.

pub mod account;
pub mod entry;
pub mod entry_id;
pub mod error;
pub mod km;
pub mod ledger;
pub mod money;

pub use account::AccountId;
pub use entry::{JournalEntry, Posting};
pub use entry_id::EntryId;
pub use error::{IdError, LedgerError, MoneyError};
pub use km::{Km, trip_fare};
pub use ledger::{Ledger, Posted};
pub use money::{Currency, Money, Usd, Vnd};
