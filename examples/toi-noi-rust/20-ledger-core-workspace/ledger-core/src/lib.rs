// Mọi item `pub` phải có tài liệu: thiếu là lỗi biên dịch, không chỉ cảnh báo.
#![deny(missing_docs)]
// README của crate cũng là trang đầu của `cargo doc`; các khối code `rust` trong đó chạy như doc test.
#![doc = include_str!("../README.md")]

pub mod account;
pub mod capture;
pub mod entry;
pub mod error;
pub mod ledger;
pub mod money;
pub mod policy;
pub mod state;

// Re-export: người dùng viết `ledger_core::Money` thay vì `ledger_core::money::Money`.
pub use account::AccountId;
pub use capture::{TripCharge, capture_entry, fare_for_km};
pub use entry::{EntryId, JournalEntry, JournalEntryBuilder, Posting};
pub use error::LedgerError;
pub use ledger::{Ledger, Posted};
pub use money::Money;
pub use policy::{
    FeePolicy, FeeSplit, FlatFee, HalfAwayFromZero, NearestEven, PercentageFee, RoundingPolicy,
    default_policy,
};
pub use state::{PaymentError, PaymentEvent, PaymentState, transition};
