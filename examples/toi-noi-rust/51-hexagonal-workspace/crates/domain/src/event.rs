//! Sự kiện miền: một bút toán đã được ghi. Chưa có JSON, serde hay Kafka ở đây.

use ledger_core::{EntryId, Posting};

/// Bút toán `entry_id` đã vào sổ. Dạng trên dây (JSON camelCase) do adapter dựng, không do domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryPosted {
    /// Mã bút toán, ví dụ `JE-000001`.
    pub entry_id: EntryId,
    /// Khoá idempotency của bút toán.
    pub idempotency_key: String,
    /// Các dòng của bút toán.
    pub postings: Vec<Posting>,
    /// Thời điểm ghi, giây Unix UTC, do `Clock` của application cung cấp.
    pub posted_at_unix: i64,
}
