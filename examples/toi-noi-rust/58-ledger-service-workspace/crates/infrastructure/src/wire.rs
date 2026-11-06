//! Hợp đồng sự kiện trên dây: JSON camelCase có `schemaVersion`. Đây là lớp chống ăn mòn:
//! kiểu của dây dừng ở đây, application chỉ thấy lệnh và kết quả thuần.

use ledger_application::Error;
use ledger_application::model::PostedEntry;
use ledger_application::service::TripCompletedCmd;
use serde::{Deserialize, Serialize};

/// Phiên bản schema duy nhất mà service hiểu.
pub const SCHEMA_VERSION: u32 = 1;

/// Chuyến đã hoàn thành (Booking C# phát). Trường lạ được bỏ qua để chịu được thêm trường.
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

    /// Dây sang lệnh của use case. `event_id` lấy từ mã chuyến vì hợp đồng chưa có `eventId`.
    ///
    /// # Errors
    /// [`Error::Invalid`] khi `schemaVersion` chưa được hỗ trợ.
    pub fn into_command(self) -> Result<TripCompletedCmd, Error> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(Error::Invalid(format!(
                "schemaVersion {} chưa hỗ trợ",
                self.schema_version
            )));
        }
        Ok(TripCompletedCmd {
            event_id: format!("TripCompleted:{}", self.trip_id),
            trip_id: self.trip_id,
            customer_id: self.customer_id,
            driver_id: self.driver_id,
            fare_dong: self.fare_dong,
        })
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

/// Bút toán đã ghi (ledger phát; Payment và Admin đọc).
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

impl From<&PostedEntry> for LedgerEntryPosted {
    fn from(posted: &PostedEntry) -> LedgerEntryPosted {
        LedgerEntryPosted {
            schema_version: SCHEMA_VERSION,
            entry_id: posted.entry_id.clone(),
            idempotency_key: posted.idempotency_key.clone(),
            postings: posted
                .postings
                .iter()
                .map(|p| PostingDto {
                    account: p.account.clone(),
                    amount_dong: p.amount_dong,
                })
                .collect(),
            posted_at: posted.posted_at.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trip_completed_uses_camel_case_and_ignores_unknown_fields() {
        let json = serde_json::to_value(TripCompleted::sample("TN-0001", 25_500)).unwrap();
        assert_eq!(json["tripId"], "TN-0001");
        assert_eq!(json["fareDong"], 25_500);
        assert_eq!(json["schemaVersion"], 1);
        let raw = r#"{"schemaVersion":1,"tripId":"TN-0001","customerId":"CU-001",
            "driverId":"DR-001","fareDong":25500,"occurredAt":"2026-11-09T01:30:00Z","extra":true}"#;
        let cmd = serde_json::from_str::<TripCompleted>(raw)
            .unwrap()
            .into_command()
            .unwrap();
        assert_eq!(cmd.event_id, "TripCompleted:TN-0001");
    }

    #[test]
    fn unknown_schema_version_is_a_permanent_error() {
        let mut ev = TripCompleted::sample("TN-0001", 25_500);
        ev.schema_version = 2;
        assert!(ev.into_command().unwrap_err().is_permanent());
    }
}
