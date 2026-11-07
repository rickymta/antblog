//! Sự kiện `LedgerEntryPosted` của hợp đồng chung (JSON camelCase, có `schemaVersion`).

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

/// Phiên bản schema mà lab này hiểu.
pub const SCHEMA_VERSION: u32 = 1;

/// Một dòng của bút toán đã ghi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostingDto {
    pub account: String,
    pub amount_dong: i64,
}

/// Bút toán đã ghi (ledger phát, key = `idempotencyKey`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerEntryPosted {
    pub schema_version: u32,
    pub entry_id: String,
    pub idempotency_key: String,
    pub postings: Vec<PostingDto>,
    pub posted_at: Timestamp,
}

/// Giải mã payload; version lạ là lỗi rõ.
///
/// # Errors
/// JSON hỏng hoặc `schemaVersion` khác phiên bản lab hiểu.
pub fn decode(payload: &[u8]) -> anyhow::Result<LedgerEntryPosted> {
    let event: LedgerEntryPosted = serde_json::from_slice(payload)?;
    anyhow::ensure!(
        event.schema_version == SCHEMA_VERSION,
        "schemaVersion {} chưa hỗ trợ",
        event.schema_version
    );
    Ok(event)
}
