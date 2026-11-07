//! DTO của hợp đồng HTTP `/v1`. Chỉ là hình dạng dữ liệu trên dây (camelCase): bất biến của
//! sổ cái (bút toán cân, không tràn số) nằm ở `ledger-domain`, không ở đây.

use serde::{Deserialize, Serialize};

/// Một dòng của bút toán. `amountDong` có dấu: dương là tiền vào, âm là tiền ra.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PostingDto {
    /// Mã tài khoản sổ cái.
    pub account: String,
    /// Số tiền có dấu, đơn vị đồng, khác 0.
    pub amount_dong: i64,
}

/// Body của `POST /v1/entries`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PostEntryRequest {
    /// Khóa idempotency: cùng khóa và cùng nội dung thì lần sau là replay.
    pub idempotency_key: String,
    /// Các dòng bút toán; tổng `amountDong` phải bằng 0.
    pub postings: Vec<PostingDto>,
}

/// Kết quả ghi bút toán.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryResponse {
    /// Mã bút toán do sổ cái cấp.
    pub entry_id: String,
    /// Khóa idempotency của bút toán.
    pub idempotency_key: String,
    /// `true` nếu key đã có và đây là bút toán cũ (HTTP 200); `false` nếu vừa tạo (HTTP 201).
    pub replayed: bool,
}

/// Kết quả `GET /v1/accounts/{id}/balance`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponse {
    /// Mã tài khoản.
    pub account: String,
    /// Số dư, đơn vị đồng.
    pub balance_dong: i64,
    /// Thời điểm đọc, RFC 3339 UTC.
    pub as_of: String,
}

/// Kết quả `GET /v1/accounts/{id}/available`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableResponse {
    /// Mã tài khoản.
    pub account: String,
    /// Số dư sổ cái.
    pub balance_dong: i64,
    /// Tổng hold ACTIVE chưa hết hạn.
    pub held_dong: i64,
    /// `balanceDong - heldDong`.
    pub available_dong: i64,
    /// Thời điểm đọc, RFC 3339 UTC.
    pub as_of: String,
}

/// Body của `POST /v1/holds`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HoldRequest {
    /// Ví bị giữ tiền.
    pub account: String,
    /// Số giữ, dương.
    pub amount_dong: i64,
    /// Khóa idempotency, thường là `trip:<id>:hold`.
    pub idempotency_key: String,
    /// Thời hạn, giây; mặc định 900.
    #[serde(default)]
    pub ttl_seconds: Option<i64>,
}

/// Kết quả tạo hold.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldResponse {
    /// Id hold.
    pub hold_id: i64,
    /// `true` nếu key đã có (HTTP 200).
    pub replayed: bool,
}

/// Body của `POST /v1/trips/{tripId}/refunds`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RefundRequest {
    /// Số thứ tự lần hoàn của chuyến, từ 1; thành đuôi của khóa `trip:<id>:refund:<n>`.
    pub refund_no: u32,
    /// Số hoàn, dương.
    pub amount_dong: i64,
}
