//! Hợp đồng sự kiện của loạt: JSON camelCase, có `schemaVersion`; và giải mã không làm sập consumer.

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Phiên bản schema consumer này hiểu. Producer đổi cấu trúc không tương thích thì tăng số này.
pub const SCHEMA_VERSION: u32 = 1;

/// Chuyến đã hoàn thành (service Booking C# phát, topic `trip.completed`, key = `tripId`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripCompleted {
    pub schema_version: u32,
    pub trip_id: String,
    pub customer_id: String,
    pub driver_id: String,
    pub fare_dong: i64,
    pub occurred_at: Timestamp,
}

/// Một dòng của bút toán đã ghi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostingDto {
    pub account: String,
    pub amount_dong: i64,
}

/// Bút toán đã ghi (ledger phát, topic `ledger.entry-posted`, key = `idempotencyKey`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerEntryPosted {
    pub schema_version: u32,
    pub entry_id: String,
    pub idempotency_key: String,
    pub postings: Vec<PostingDto>,
    pub posted_at: Timestamp,
}

/// Lý do một thông điệp không dùng được. Mỗi lý do là một giá trị, không phải panic.
#[derive(Debug, Error)]
pub enum DecodeError {
    /// Không phải JSON, hoặc thiếu/sai kiểu một trường của schema v1.
    #[error("JSON hỏng hoặc không khớp schema: {0}")]
    Malformed(#[from] serde_json::Error),
    /// JSON hợp lệ nhưng không có `schemaVersion`.
    #[error("thiếu schemaVersion")]
    MissingVersion,
    /// `schemaVersion` mà consumer này chưa biết cách đọc.
    #[error("schemaVersion {found} chưa hỗ trợ (consumer hiểu {SCHEMA_VERSION})")]
    UnsupportedVersion { found: u64 },
}

/// Đọc `TripCompleted`. Kiểm `schemaVersion` TRƯỚC khi đọc phần còn lại, vì cấu trúc v2
/// có thể khác v1 và khi đó lỗi «thiếu trường» sẽ che mất lỗi thật (version lạ).
/// Trường thừa trong v1 được bỏ qua: producer thêm trường tùy chọn không làm hỏng consumer cũ.
pub fn decode_trip_completed(bytes: &[u8]) -> Result<TripCompleted, DecodeError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Header {
        schema_version: Option<u64>,
    }
    let header: Header = serde_json::from_slice(bytes)?;
    match header.schema_version {
        None => Err(DecodeError::MissingVersion),
        Some(v) if v == u64::from(SCHEMA_VERSION) => Ok(serde_json::from_slice(bytes)?),
        Some(found) => Err(DecodeError::UnsupportedVersion { found }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"{"schemaVersion":1,"tripId":"TN-0001","customerId":"CU-001","driverId":"DR-001","fareDong":25500,"occurredAt":"2026-11-06T02:30:00Z"}"#;

    #[test]
    fn decodes_v1_and_ignores_extra_fields() {
        let with_extra = GOOD.replace("\"fareDong\"", "\"promoCode\":\"X\",\"fareDong\"");
        let event = decode_trip_completed(with_extra.as_bytes()).unwrap();
        assert_eq!(event.trip_id, "TN-0001");
        assert_eq!(event.fare_dong, 25_500);
    }

    #[test]
    fn rejects_unknown_version_before_reading_the_body() {
        let err =
            decode_trip_completed(br#"{"schemaVersion":2,"trip":{"id":"TN-0001"}}"#).unwrap_err();
        assert!(matches!(err, DecodeError::UnsupportedVersion { found: 2 }));
    }

    #[test]
    fn rejects_garbage_missing_version_and_missing_field() {
        assert!(matches!(
            decode_trip_completed(b"{khong phai json"),
            Err(DecodeError::Malformed(_))
        ));
        assert!(matches!(
            decode_trip_completed(br#"{"tripId":"TN-0001"}"#),
            Err(DecodeError::MissingVersion)
        ));
        let no_fare = GOOD.replace("\"fareDong\":25500,", "");
        assert!(matches!(
            decode_trip_completed(no_fare.as_bytes()),
            Err(DecodeError::Malformed(_))
        ));
    }

    #[test]
    fn encodes_camel_case_with_version() {
        let event = decode_trip_completed(GOOD.as_bytes()).unwrap();
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, GOOD);
    }
}
