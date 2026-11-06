//! Hợp đồng sự kiện của loạt: JSON camelCase, có `schemaVersion`.

use serde::{Deserialize, Serialize};

/// Phiên bản schema duy nhất mà service hiểu.
pub const SCHEMA_VERSION: u32 = 1;

/// Chuyến đã hoàn thành (Booking C# phát). Trường lạ được bỏ qua để consumer chịu được thêm trường.
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
    /// Cước, đồng.
    pub fare_dong: i64,
    /// Thời điểm hoàn thành, ISO 8601 UTC.
    pub occurred_at: String,
}

impl TripCompleted {
    /// Sự kiện mẫu của chuyến `trip_id` với cước `fare_dong`.
    pub fn sample(trip_id: &str, fare_dong: i64) -> TripCompleted {
        TripCompleted {
            schema_version: SCHEMA_VERSION,
            trip_id: trip_id.to_string(),
            customer_id: "CU-001".to_string(),
            driver_id: "DR-001".to_string(),
            fare_dong,
            occurred_at: "2026-11-09T01:30:00Z".to_string(),
        }
    }

    /// Định danh để khử trùng ở inbox. Sự kiện này chưa có `eventId`, nên dùng mã chuyến.
    pub fn event_id(&self) -> String {
        format!("TripCompleted:{}", self.trip_id)
    }
}

/// Một dòng của bút toán trong sự kiện phát ra.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostingDto {
    /// Mã tài khoản.
    pub account: String,
    /// Số tiền có dấu, đồng.
    pub amount_dong: i64,
}

/// Bút toán đã ghi (ledger phát; Payment/Admin đọc).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerEntryPosted {
    /// Luôn là 1 ở bản này.
    pub schema_version: u32,
    /// Mã bút toán, ví dụ `JE-000001`.
    pub entry_id: String,
    /// Khoá idempotency của bút toán.
    pub idempotency_key: String,
    /// Các dòng của bút toán.
    pub postings: Vec<PostingDto>,
    /// Thời điểm ghi, ISO 8601 UTC.
    pub posted_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trip_completed_uses_camel_case() {
        let json = serde_json::to_value(TripCompleted::sample("TN-0001", 25_500)).unwrap();
        assert_eq!(json["tripId"], "TN-0001");
        assert_eq!(json["fareDong"], 25_500);
        assert_eq!(json["schemaVersion"], 1);
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let raw = r#"{"schemaVersion":1,"tripId":"TN-0001","customerId":"CU-001",
            "driverId":"DR-001","fareDong":25500,"occurredAt":"2026-11-09T01:30:00Z","extra":true}"#;
        assert!(serde_json::from_str::<TripCompleted>(raw).is_ok());
    }
}
