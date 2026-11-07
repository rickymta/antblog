//! Dạng trên dây của `LedgerEntryPosted`: JSON camelCase, `schemaVersion: 1`.
//!
//! serde nằm ở đây, không nằm ở `domain`: đổi tên trường JSON không động tới luật nghiệp vụ.

use application::PublishError;
use domain::EntryPosted;
use serde::{Deserialize, Serialize};

/// Một dòng của bút toán trong sự kiện.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostingDto {
    /// Mã tài khoản.
    pub account: String,
    /// Số tiền có dấu, đồng.
    pub amount_dong: i64,
}

/// Sự kiện `LedgerEntryPosted` theo hợp đồng chung của loạt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerEntryPostedDto {
    /// Luôn là 1.
    pub schema_version: u32,
    /// Mã bút toán, ví dụ `JE-000001`.
    pub entry_id: String,
    /// Khoá idempotency.
    pub idempotency_key: String,
    /// Các dòng.
    pub postings: Vec<PostingDto>,
    /// Thời điểm ghi, ISO 8601 UTC.
    pub posted_at: String,
}

/// Đổi sự kiện miền sang dạng trên dây.
///
/// # Errors
/// Thời điểm nằm ngoài khoảng `jiff` biểu diễn được.
pub fn to_wire(event: &EntryPosted) -> Result<LedgerEntryPostedDto, PublishError> {
    let posted_at = jiff::Timestamp::from_second(event.posted_at_unix)
        .map_err(|e| PublishError(Box::new(e)))?
        .to_string();
    Ok(LedgerEntryPostedDto {
        schema_version: 1,
        entry_id: event.entry_id.to_string(),
        idempotency_key: event.idempotency_key.clone(),
        postings: event
            .postings
            .iter()
            .map(|p| PostingDto {
                account: p.account.as_str().to_string(),
                amount_dong: p.amount.dong(),
            })
            .collect(),
        posted_at,
    })
}

/// Serialize sang JSON.
///
/// # Errors
/// Như [`to_wire`], hoặc serde lỗi.
pub fn to_json(event: &EntryPosted) -> Result<String, PublishError> {
    serde_json::to_string(&to_wire(event)?).map_err(|e| PublishError(Box::new(e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{AccountId, EntryId, Money, Posting};

    #[test]
    fn json_is_camel_case_with_schema_version() {
        let event = EntryPosted {
            entry_id: EntryId::parse("JE-000001").unwrap(),
            idempotency_key: "trip:TN-0001:capture".to_string(),
            postings: vec![
                Posting::new(
                    AccountId::new("customer:CU-001").unwrap(),
                    Money::from_dong(-25_500),
                ),
                Posting::new(
                    AccountId::new("psp:vixanh:clearing").unwrap(),
                    Money::from_dong(25_500),
                ),
            ],
            posted_at_unix: 0,
        };
        let json: serde_json::Value = serde_json::from_str(&to_json(&event).unwrap()).unwrap();
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["entryId"], "JE-000001");
        assert_eq!(json["postings"][0]["amountDong"], -25_500);
        assert_eq!(json["postedAt"], "1970-01-01T00:00:00Z");
    }
}
