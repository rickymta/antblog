//! Ánh xạ `LedgerError` (lỗi miền) sang Problem Details. Một `match` vét cạn, không có nhánh `_`:
//! thêm biến thể mới vào `ledger-core` thì file này không biên dịch cho tới khi chọn mã HTTP.

use axum::http::StatusCode;
use ledger_core::LedgerError;

use crate::problem::Problem;

/// Mã HTTP và `type` cho từng lỗi miền.
///
/// | Lỗi miền | HTTP |
/// |---|---|
/// | lệch, thiếu posting, tràn số, mã rỗng | 422 |
/// | `IdempotencyConflict` | 409 |
/// | `UnknownEntry` | 404 |
/// | `InvalidEntryId` | 400 |
/// | `InvalidFee` | 500 (lỗi của chính sách phí, không phải của client) |
pub fn problem_from_ledger(err: &LedgerError, instance: &str) -> Problem {
    let unprocessable = |slug: &str, title: &str| {
        Problem::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            slug,
            title,
            err.to_string(),
            instance,
        )
    };
    match err {
        LedgerError::EmptyAccountId => unprocessable("empty-account-id", "Mã tài khoản rỗng"),
        LedgerError::EmptyIdempotencyKey => {
            unprocessable("empty-idempotency-key", "Idempotency key rỗng")
        }
        LedgerError::TooFewPostings { .. } => {
            unprocessable("too-few-postings", "Bút toán thiếu posting")
        }
        LedgerError::Unbalanced { .. } => unprocessable("unbalanced-entry", "Bút toán lệch"),
        LedgerError::Overflow => unprocessable("amount-overflow", "Tổng tiền vượt giới hạn"),
        LedgerError::IdempotencyConflict { .. } => Problem::new(
            StatusCode::CONFLICT,
            "idempotency-conflict",
            "Idempotency key đã dùng cho nội dung khác",
            err.to_string(),
            instance,
        ),
        LedgerError::UnknownEntry { .. } => Problem::new(
            StatusCode::NOT_FOUND,
            "unknown-entry",
            "Không có bút toán này",
            err.to_string(),
            instance,
        ),
        LedgerError::InvalidEntryId { .. } => Problem::new(
            StatusCode::BAD_REQUEST,
            "invalid-entry-id",
            "Mã bút toán sai dạng",
            err.to_string(),
            instance,
        ),
        // Lỗi nội bộ: không đưa chi tiết ra ngoài, chỉ traceId để tra log.
        LedgerError::InvalidFee { .. } => Problem::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "Lỗi nội bộ",
            "xem log với traceId",
            instance,
        ),
    }
}
