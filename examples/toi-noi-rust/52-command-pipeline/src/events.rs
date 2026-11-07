//! Sự kiện miền và hợp đồng JSON của loạt: camelCase, có `schemaVersion: 1`.

use serde::{Deserialize, Serialize};

/// Phiên bản schema của hợp đồng sự kiện.
pub const SCHEMA_VERSION: u8 = 1;

/// Một dòng posting trong sự kiện.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostingDto {
    /// Tài khoản, ví dụ `driver:DR-001`.
    pub account: String,
    /// Số tiền có dấu, đơn vị đồng.
    pub amount_dong: i64,
}

/// Sự kiện ledger phát sau khi ghi một bút toán.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerEntryPosted {
    /// Luôn là 1.
    pub schema_version: u8,
    /// Mã hiển thị `JE-000001`.
    pub entry_id: String,
    /// Khóa idempotency của bút toán.
    pub idempotency_key: String,
    /// Các dòng posting.
    pub postings: Vec<PostingDto>,
    /// Thời điểm ghi, ISO 8601 UTC, lấy từ clock của `Ctx` (không đọc đồng hồ máy).
    pub posted_at: String,
}

/// Sự kiện miền do aggregate sinh ra. Chỉ có một loại trong lab này.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    /// Bút toán đã được ghi (chưa chắc đã commit: đó là việc của pipeline).
    EntryPosted(LedgerEntryPosted),
}

impl DomainEvent {
    /// Topic dự kiến khi relay gửi lên Kafka.
    pub fn topic(&self) -> &'static str {
        match self {
            DomainEvent::EntryPosted(_) => "ledger.entry-posted",
        }
    }

    /// Key Kafka: giữ thứ tự theo khóa idempotency.
    pub fn key(&self) -> &str {
        match self {
            DomainEvent::EntryPosted(e) => &e.idempotency_key,
        }
    }

    /// Payload JSON.
    pub fn payload(&self) -> String {
        match self {
            DomainEvent::EntryPosted(e) => {
                serde_json::to_string(e).expect("struct thuần serialize không lỗi")
            }
        }
    }
}
