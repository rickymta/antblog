//! Lab bài 46: làm tròn và chia tiền không mất đồng.
//!
//! - [`naive`]: làm tròn từng phần riêng lẻ, cách làm sai làm lệch đồng.
//! - [`strategies`]: các `RoundingStrategy` của `rust_decimal` đặt cạnh nhau.
//! - [`allocate`]: largest remainder chuẩn, phá hòa tất định, số âm, tính bằng `i128`.
//! - [`book`]: nối với `ledger-core` (`FeePolicy`, `JournalEntry`) và bản ghi quy tắc.
//! - [`search`]: tìm phản ví dụ bằng proptest và vét cạn.

pub mod allocate;
pub mod book;
pub mod naive;
pub mod search;
pub mod strategies;

pub use allocate::{AllocError, Party, Share, allocate, allocate_with_plug};
