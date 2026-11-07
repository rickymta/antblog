//! Hợp đồng sự kiện của loạt: JSON camelCase, có `schemaVersion`. Trường lạ được bỏ qua để
//! consumer chịu được việc producer thêm trường.

use serde::{Deserialize, Serialize};

/// Phiên bản schema duy nhất mà service hiểu.
pub const SCHEMA_VERSION: u32 = 1;

/// Chuyến đã hoàn thành (Booking C# phát).
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
    /// Sự kiện mẫu cho test và kịch bản.
    pub fn sample(trip_id: &str, customer: &str, driver: &str, fare_dong: i64) -> TripCompleted {
        TripCompleted {
            schema_version: SCHEMA_VERSION,
            trip_id: trip_id.to_string(),
            customer_id: customer.to_string(),
            driver_id: driver.to_string(),
            fare_dong,
            occurred_at: "2026-11-09T01:30:00Z".to_string(),
        }
    }
}

/// VíXanh đã thu tiền của chuyến (Payment C# phát).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentCaptured {
    /// Luôn là 1 ở bản này.
    pub schema_version: u32,
    /// Mã thanh toán phía Payment, ví dụ `PAY-0001`.
    pub payment_id: String,
    /// Mã chuyến.
    pub trip_id: String,
    /// Số tiền VíXanh đã thu thật, đồng (có thể thấp hơn cước nếu có khuyến mãi).
    pub amount_dong: i64,
    /// Mã giao dịch của VíXanh, ví dụ `vx_ch_0001`.
    pub external_ref: String,
    /// Thời điểm thu, ISO 8601 UTC.
    pub occurred_at: String,
}

impl PaymentCaptured {
    /// Sự kiện mẫu cho test và kịch bản.
    pub fn sample(
        trip_id: &str,
        payment_id: &str,
        external_ref: &str,
        amount: i64,
    ) -> PaymentCaptured {
        PaymentCaptured {
            schema_version: SCHEMA_VERSION,
            payment_id: payment_id.to_string(),
            trip_id: trip_id.to_string(),
            amount_dong: amount,
            external_ref: external_ref.to_string(),
            occurred_at: "2026-11-09T01:31:00Z".to_string(),
        }
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
    fn events_use_camel_case() {
        let trip =
            serde_json::to_value(TripCompleted::sample("TN-0001", "CU-001", "DR-001", 25_500))
                .unwrap();
        assert_eq!(trip["tripId"], "TN-0001");
        assert_eq!(trip["fareDong"], 25_500);
        assert_eq!(trip["schemaVersion"], 1);
        let paid = serde_json::to_value(PaymentCaptured::sample(
            "TN-0001",
            "PAY-0001",
            "vx_ch_0001",
            25_500,
        ))
        .unwrap();
        assert_eq!(paid["externalRef"], "vx_ch_0001");
        assert_eq!(paid["amountDong"], 25_500);
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let raw = r#"{"schemaVersion":1,"paymentId":"PAY-0001","tripId":"TN-0001","amountDong":25500,
            "externalRef":"vx_ch_0001","occurredAt":"2026-11-09T01:31:00Z","extra":true}"#;
        assert!(serde_json::from_str::<PaymentCaptured>(raw).is_ok());
    }
}
