//! Hợp đồng sự kiện của loạt: JSON camelCase, có `schemaVersion: 1`.
//! Quy ước của lab này: `event_id` không nằm trong payload mà đi theo header Kafka `event-id`.

use serde::{Deserialize, Serialize};

/// Phiên bản schema mà lab này hiểu. Phiên bản khác bị từ chối (đi DLQ).
pub const SCHEMA_VERSION: u8 = 1;

/// Tên header Kafka mang `event_id`.
pub const EVENT_ID_HEADER: &str = "event-id";

/// Sự kiện service C# phát khi chuyến hoàn thành.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripCompleted {
    /// Luôn là 1.
    pub schema_version: u8,
    /// Mã chuyến, ví dụ `TN-0001`.
    pub trip_id: String,
    /// Mã khách, ví dụ `CU-001`.
    pub customer_id: String,
    /// Mã tài xế, ví dụ `DR-001`.
    pub driver_id: String,
    /// Cước khách trả, đơn vị đồng.
    pub fare_dong: i64,
    /// Thời điểm hoàn thành, ISO 8601 UTC.
    pub occurred_at: String,
}

impl TripCompleted {
    /// Chuyến mẫu `TN-0001` của khách `CU-001`, tài xế `DR-001`, cước 25.500 đồng.
    pub fn sample(trip_id: &str, driver_id: &str) -> TripCompleted {
        TripCompleted {
            schema_version: SCHEMA_VERSION,
            trip_id: trip_id.to_string(),
            customer_id: "CU-001".to_string(),
            driver_id: driver_id.to_string(),
            fare_dong: 25_500,
            occurred_at: "2026-11-06T02:15:00Z".to_string(),
        }
    }
}

/// Một dòng của bút toán trong sự kiện.
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
    /// Thời điểm ghi, ISO 8601 UTC, do PostgreSQL cấp.
    pub posted_at: String,
}

/// Lỗi đọc payload: không thử lại được vì dữ liệu sai chứ không phải hạ tầng.
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    /// Không phải JSON hợp lệ theo schema.
    #[error("payload không đọc được: {0}")]
    Malformed(#[from] serde_json::Error),
    /// JSON đúng hình nhưng phiên bản schema lạ.
    #[error("schemaVersion {0} chưa được hỗ trợ")]
    UnsupportedVersion(u8),
}

/// Đọc `TripCompleted` và kiểm phiên bản.
///
/// # Errors
/// [`EventError`] khi JSON sai hoặc `schemaVersion` khác 1.
pub fn parse_trip_completed(bytes: &[u8]) -> Result<TripCompleted, EventError> {
    let event: TripCompleted = serde_json::from_slice(bytes)?;
    if event.schema_version != SCHEMA_VERSION {
        return Err(EventError::UnsupportedVersion(event.schema_version));
    }
    Ok(event)
}

/// Đọc `LedgerEntryPosted` và kiểm phiên bản.
///
/// # Errors
/// [`EventError`] khi JSON sai hoặc `schemaVersion` khác 1.
pub fn parse_entry_posted(bytes: &[u8]) -> Result<LedgerEntryPosted, EventError> {
    let event: LedgerEntryPosted = serde_json::from_slice(bytes)?;
    if event.schema_version != SCHEMA_VERSION {
        return Err(EventError::UnsupportedVersion(event.schema_version));
    }
    Ok(event)
}
