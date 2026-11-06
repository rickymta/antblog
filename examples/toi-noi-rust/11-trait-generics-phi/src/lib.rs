//! Thư viện sổ cái rút gọn của Tới Nơi (bài 11, phần P02).
//!
//! Từ bài 10 thêm hai trait chính sách (`RoundingPolicy`, `FeePolicy`) và
//! `capture_entry`, hàm dựng bút toán thu cước chỉ phụ thuộc vào trait.

pub mod account;
pub mod capture;
pub mod entry;
pub mod error;
pub mod ledger;
pub mod money;
pub mod policy;

pub use account::AccountId;
pub use capture::{TripCharge, capture_entry, fare_for_km};
pub use entry::{JournalEntry, Posting};
pub use error::LedgerError;
pub use ledger::{Ledger, Posted};
pub use money::Money;
pub use policy::{
    FeePolicy, FeeSplit, FlatFee, HalfAwayFromZero, NearestEven, PercentageFee, RoundingPolicy,
    default_policy,
};
