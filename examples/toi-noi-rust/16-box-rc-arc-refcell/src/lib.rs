//! Lab bài 16 (phần P02): smart pointer trong sổ cái Tới Nơi.
//!
//! Năm module đầu (`money`, `account`, `entry`, `error`, `ledger`) copy từ lab bài 10,
//! thêm `checked_sub`/`checked_mul` cho `Money` và vài biến thể lỗi. Năm module mới:
//! `boxed` (stack/heap, kiểu đệ quy), `policy` (`Box<dyn FeePolicy>`), `tree` (`Rc` + `Weak`),
//! `shared` (`RefCell`, `Cell`) và `threads` (`Arc<Mutex<_>>`).

pub mod account;
pub mod boxed;
pub mod entry;
pub mod error;
pub mod ledger;
pub mod money;
pub mod policy;
pub mod shared;
pub mod threads;
pub mod tree;

pub use account::AccountId;
pub use entry::{JournalEntry, Posting};
pub use error::LedgerError;
pub use ledger::{Ledger, Posted};
pub use money::Money;
