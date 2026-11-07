//! `domain`: luật nghiệp vụ của một chuyến đi, không biết async, mạng hay cơ sở dữ liệu.
//!
//! Crate này dùng thẳng `ledger-core` (bài 20) làm lõi: `Money`, `AccountId`, `JournalEntry`
//! đã mang bất biến "bút toán cân", bọc lại chỉ tạo thêm một bản sao của bất biến đó.
//! Ngoài `thiserror`, phụ thuộc duy nhất của crate là `ledger-core`; test kiểm đồ thị phụ thuộc nằm ở crate `api`.

mod event;
mod trip;

// Re-export những kiểu lõi để crate khác chỉ cần `domain::Money`, không cần tự thêm `ledger-core`.
pub use event::EntryPosted;
pub use ledger_core::{AccountId, EntryId, JournalEntry, LedgerError, Money, Posting};
pub use trip::{Settlement, Trip, TripError};
