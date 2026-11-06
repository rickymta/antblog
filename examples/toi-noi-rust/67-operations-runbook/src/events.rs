//! Hợp đồng sự kiện của loạt: JSON camelCase, có `schemaVersion`.

use serde::{Deserialize, Serialize};

/// Phiên bản schema đầu tiên.
pub const SCHEMA_VERSION: u32 = 1;

/// Sự kiện service C# phát khi chuyến hoàn thành.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripCompleted {
    /// Phiên bản schema.
    pub schema_version: u32,
    /// Mã chuyến, ví dụ `TN-0001`.
    pub trip_id: String,
    /// Mã khách.
    pub customer_id: String,
    /// Mã tài xế.
    pub driver_id: String,
    /// Cước, đồng.
    pub fare_dong: i64,
    /// Thời điểm hoàn thành, ISO 8601 UTC.
    pub occurred_at: String,
}

impl TripCompleted {
    /// Chuyến mẫu: khách `CU-001`, tài xế `DR-001`, cước 25.500 đồng.
    pub fn sample(trip_id: &str) -> TripCompleted {
        TripCompleted {
            schema_version: SCHEMA_VERSION,
            trip_id: trip_id.to_string(),
            customer_id: "CU-001".to_string(),
            driver_id: "DR-001".to_string(),
            fare_dong: 25_500,
            occurred_at: "2026-11-06T01:30:00Z".to_string(),
        }
    }

    /// JSON của sự kiện.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("sự kiện luôn serialize được")
    }

    /// Cùng chuyến nhưng producer đã nâng lên `schemaVersion: 2` và thêm trường `channel`.
    /// Mọi trường cũ còn nguyên, nên consumer biết v2 đọc được bằng cùng struct.
    pub fn sample_v2_json(trip_id: &str) -> String {
        let mut value = serde_json::to_value(TripCompleted::sample(trip_id)).expect("hợp lệ");
        value["schemaVersion"] = 2.into();
        value["channel"] = "app".into();
        value.to_string()
    }
}

/// Một dòng của bút toán trong sự kiện phát ra.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostingDto {
    /// Tài khoản.
    pub account: String,
    /// Số tiền có dấu, đồng.
    pub amount_dong: i64,
}

/// Bút toán đã ghi (ledger phát).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerEntryPosted {
    /// Phiên bản schema.
    pub schema_version: u32,
    /// Mã hiển thị `JE-000001`.
    pub entry_id: String,
    /// Khoá idempotency.
    pub idempotency_key: String,
    /// Các dòng.
    pub postings: Vec<PostingDto>,
    /// Thời điểm ghi, ISO 8601 UTC.
    pub posted_at: String,
}

/// Lỗi đọc payload: dữ liệu sai, thử lại cũng không hết.
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    /// Không phải JSON hợp lệ theo schema.
    #[error("payload không đọc được: {0}")]
    Malformed(#[from] serde_json::Error),
    /// JSON đúng hình nhưng phiên bản chưa được bản consumer này hỗ trợ.
    #[error("schemaVersion {0} chưa được hỗ trợ")]
    UnsupportedVersion(u32),
}

/// Đọc `TripCompleted`; `accepted` là các `schemaVersion` mà bản consumer đang chạy hiểu.
///
/// # Errors
/// [`EventError`] khi JSON sai hoặc phiên bản không nằm trong `accepted`.
pub fn parse_trip_completed(bytes: &[u8], accepted: &[u32]) -> Result<TripCompleted, EventError> {
    let event: TripCompleted = serde_json::from_slice(bytes)?;
    if !accepted.contains(&event.schema_version) {
        return Err(EventError::UnsupportedVersion(event.schema_version));
    }
    Ok(event)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v2_is_rejected_until_the_consumer_accepts_it() {
        let v2 = TripCompleted::sample_v2_json("TN-0103");
        let err = parse_trip_completed(v2.as_bytes(), &[1]).unwrap_err();
        assert_eq!(err.to_string(), "schemaVersion 2 chưa được hỗ trợ");
        let ok = parse_trip_completed(v2.as_bytes(), &[1, 2]).unwrap();
        assert_eq!(ok.trip_id, "TN-0103");
        assert!(matches!(
            parse_trip_completed(b"{not json", &[1, 2]),
            Err(EventError::Malformed(_))
        ));
    }
}
