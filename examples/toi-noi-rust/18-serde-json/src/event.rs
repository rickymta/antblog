//! Hợp đồng JSON của sự kiện sổ cái: `LedgerEntryPosted`, `LedgerEntryReversed`.

use jiff::Timestamp;
use rust_decimal::Decimal;
use serde::de::{self, Deserializer};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

use crate::entry::JournalEntry;

/// Phiên bản schema. Chỉ chấp nhận số 1: ai gửi `schemaVersion: 2` bị từ chối
/// ngay lúc đọc, thay vì bị hiểu nhầm theo luật của phiên bản 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaV1;

impl Serialize for SchemaV1 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(1)
    }
}

impl<'de> Deserialize<'de> for SchemaV1 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<SchemaV1, D::Error> {
        match u32::deserialize(deserializer)? {
            1 => Ok(SchemaV1),
            other => Err(de::Error::custom(format!(
                "schemaVersion {other} không được hỗ trợ, cần 1"
            ))),
        }
    }
}

/// Payload của `LedgerEntryPosted`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EntryPosted {
    pub schema_version: SchemaV1,
    pub entry_id: String,
    /// ISO 8601 UTC, ví dụ `2026-11-06T03:15:00Z`.
    pub posted_at: Timestamp,
    /// Tỷ lệ phí nền tảng, viết thành chuỗi: `"0.20"`, không phải số thực.
    #[serde(with = "rust_decimal::serde::str")]
    pub fee_rate: Decimal,
    pub entry: JournalEntry,
    /// Thiếu trong JSON thì là `None`; `None` thì không ghi ra.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    /// Bản cũ chưa có trường này: thiếu thì là danh sách rỗng, rỗng thì không ghi ra.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
}

/// Payload của `LedgerEntryReversed`: bút toán đảo trỏ về bút toán gốc.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EntryReversed {
    pub schema_version: SchemaV1,
    pub entry_id: String,
    pub reverses_entry_id: String,
    pub reason: String,
}

/// Sự kiện sổ cái. `tag = "type"`: tên biến thể thành trường `"type"` trong JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LedgerEvent {
    LedgerEntryPosted(EntryPosted),
    LedgerEntryReversed(EntryReversed),
}
