//! Sự kiện chung của loạt: JSON camelCase, có `schemaVersion`.

use ledger_core::fare_for_km;
use serde::{Deserialize, Serialize};

/// Chuyến đã hoàn thành (Booking C# phát, ledger tiêu thụ).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripCompleted {
    /// Luôn là 1 ở bản này.
    pub schema_version: u32,
    /// Mã chuyến, ví dụ `TN-0001`.
    pub trip_id: String,
    /// Mã khách, ví dụ `CU-001`.
    pub customer_id: String,
    /// Mã tài xế, ví dụ `DR-001`.
    pub driver_id: String,
    /// Cước đã tính, đơn vị đồng.
    pub fare_dong: i64,
    /// Thời điểm hoàn thành, ISO 8601 UTC.
    pub occurred_at: String,
}

impl TripCompleted {
    /// Chuyến mẫu thứ `n`: `TN-0001` đi 5 km (25.500 đồng), các chuyến sau đi 1 đến 7 km.
    pub fn sample(n: u32) -> TripCompleted {
        let km = if n == 1 { 5 } else { 1 + n % 7 };
        TripCompleted {
            schema_version: 1,
            trip_id: format!("TN-{n:04}"),
            customer_id: "CU-001".to_string(),
            driver_id: "DR-001".to_string(),
            fare_dong: fare_for_km(km).dong(),
            occurred_at: "2026-11-06T08:00:00Z".to_string(),
        }
    }
}

/// Một dòng của bút toán trong sự kiện phát ra.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostingDto {
    /// Mã tài khoản.
    pub account: String,
    /// Số tiền có dấu theo dòng tiền, đơn vị đồng.
    pub amount_dong: i64,
}

/// Bút toán đã ghi (ledger phát qua outbox).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerEntryPosted {
    /// Luôn là 1 ở bản này.
    pub schema_version: u32,
    /// Mã hiển thị `JE-000001`.
    pub entry_id: String,
    /// Khoá idempotency, cũng là khoá partition.
    pub idempotency_key: String,
    /// Các dòng bút toán.
    pub postings: Vec<PostingDto>,
    /// Thời điểm ghi, ISO 8601 UTC.
    pub posted_at: String,
}
