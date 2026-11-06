//! DTO của hợp đồng HTTP `/v1`. Chỉ là hình dạng dữ liệu trên dây: bất biến của sổ cái
//! (bút toán cân, không tràn số) nằm ở `ledger-core`, không ở đây.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Một dòng của bút toán. `amountDong` có dấu: dương là tiền vào, âm là tiền ra.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PostingDto {
    /// Mã tài khoản sổ cái.
    #[schema(example = "customer:CU-001")]
    pub account: String,
    /// Số tiền có dấu, đơn vị đồng, khác 0.
    #[schema(example = -25_500)]
    pub amount_dong: i64,
}

/// Body của `POST /v1/entries`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PostEntryRequest {
    /// Khóa idempotency: cùng khóa và cùng nội dung thì lần sau là replay.
    #[schema(example = "trip:TN-0001:capture")]
    pub idempotency_key: String,
    /// Các dòng bút toán; tổng `amountDong` phải bằng 0.
    pub postings: Vec<PostingDto>,
}

/// Kết quả ghi bút toán.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EntryResponse {
    /// Mã bút toán do sổ cái cấp.
    #[schema(example = "JE-000001")]
    pub entry_id: String,
    /// Khóa idempotency của bút toán.
    pub idempotency_key: String,
    /// Các dòng đã ghi.
    pub postings: Vec<PostingDto>,
    /// `true` nếu key đã có và đây là bút toán cũ (HTTP 200); `false` nếu vừa tạo (HTTP 201).
    pub replayed: bool,
}

/// Kết quả `GET /v1/accounts/{id}/balance`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponse {
    /// Mã tài khoản.
    #[schema(example = "driver:DR-001")]
    pub account: String,
    /// Số dư, đơn vị đồng.
    #[schema(example = 20_400)]
    pub balance_dong: i64,
    /// Thời điểm đọc, RFC 3339 UTC.
    #[schema(example = "2026-11-06T03:00:00Z")]
    pub as_of: String,
}
