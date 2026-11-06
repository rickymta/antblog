//! Ánh xạ `ledger_application::Error` sang Problem Details. Các `match` vét cạn, không có nhánh `_`:
//! thêm biến thể mới vào application hoặc domain thì file này không biên dịch cho tới khi chọn mã HTTP.

use axum::http::StatusCode;
use ledger_application::Error;
use ledger_domain::LedgerError;
use ledger_domain::hold::HoldError;
use ledger_domain::refund::RefundError;
use ledger_domain::trip::TripError;
use tracing::error;

use crate::problem::Problem;

/// Mã HTTP và `type` cho từng lỗi.
///
/// | Lỗi | HTTP |
/// |---|---|
/// | dữ liệu sai, bút toán lệch, không đủ khả dụng, hoàn vượt mức | 422 |
/// | `IdempotencyConflict`, key hold/hoàn xung đột | 409 |
/// | tài khoản hoặc chuyến không có | 404 |
/// | hạ tầng tạm thời | 503 |
/// | lỗi lưu trữ, `InvalidFee` | 500 (chi tiết chỉ nằm trong log) |
pub fn problem_from_error(err: &Error, instance: &str) -> Problem {
    let make = |status: StatusCode, slug: &str, title: &str| {
        Problem::new(status, slug, title, err.to_string(), instance)
    };
    let unprocessable =
        |slug: &str, title: &str| make(StatusCode::UNPROCESSABLE_ENTITY, slug, title);
    let conflict = |slug: &str, title: &str| make(StatusCode::CONFLICT, slug, title);
    let internal = || {
        error!(%err, "lỗi nội bộ");
        // Không đưa chi tiết ra ngoài, chỉ traceId để tra log.
        Problem::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "Lỗi nội bộ",
            "xem log với traceId",
            instance,
        )
    };
    match err {
        Error::Invalid(_) => unprocessable("invalid-request", "Yêu cầu không hợp lệ"),
        Error::Ledger(e) => match e {
            LedgerError::EmptyAccountId => unprocessable("empty-account-id", "Mã tài khoản rỗng"),
            LedgerError::EmptyIdempotencyKey => {
                unprocessable("empty-idempotency-key", "Idempotency key rỗng")
            }
            LedgerError::TooFewPostings { .. } => {
                unprocessable("too-few-postings", "Bút toán thiếu posting")
            }
            LedgerError::Unbalanced { .. } => unprocessable("unbalanced-entry", "Bút toán lệch"),
            LedgerError::Overflow => unprocessable("amount-overflow", "Tổng tiền vượt giới hạn"),
            LedgerError::IdempotencyConflict { .. } => conflict(
                "idempotency-conflict",
                "Idempotency key đã dùng cho nội dung khác",
            ),
            LedgerError::UnknownEntry { .. } => make(
                StatusCode::NOT_FOUND,
                "unknown-entry",
                "Không có bút toán này",
            ),
            LedgerError::InvalidEntryId { .. } => make(
                StatusCode::BAD_REQUEST,
                "invalid-entry-id",
                "Mã bút toán sai dạng",
            ),
            LedgerError::InvalidFee { .. } => internal(),
        },
        Error::Trip(e) => match e {
            TripError::NonPositiveFare(_) | TripError::EmptyTripId => {
                unprocessable("invalid-trip", "Dữ liệu chuyến không hợp lệ")
            }
            TripError::Ledger(_) => internal(),
        },
        Error::Hold(e) => match e {
            HoldError::NonPositive(_) | HoldError::EmptyKey | HoldError::NonPositiveTtl => {
                unprocessable("invalid-hold", "Yêu cầu giữ tiền không hợp lệ")
            }
            HoldError::InsufficientAvailable { .. } => {
                unprocessable("insufficient-available", "Không đủ số dư khả dụng")
            }
            HoldError::KeyConflict { .. } => conflict(
                "idempotency-conflict",
                "Idempotency key đã dùng cho nội dung khác",
            ),
        },
        Error::Refund(e) => match e {
            RefundError::NonPositive(_) | RefundError::BadSequence => {
                unprocessable("invalid-refund", "Yêu cầu hoàn tiền không hợp lệ")
            }
            RefundError::ExceedsRefundable { .. } => {
                unprocessable("refund-exceeds-refundable", "Hoàn vượt phần còn hoàn được")
            }
            RefundError::KeyConflict { .. } => conflict(
                "idempotency-conflict",
                "Idempotency key đã dùng cho nội dung khác",
            ),
            RefundError::Ledger(_) => internal(),
        },
        Error::IdempotencyConflict { .. } => conflict(
            "idempotency-conflict",
            "Idempotency key đã dùng cho nội dung khác",
        ),
        Error::UnknownAccount(_) => make(
            StatusCode::NOT_FOUND,
            "unknown-account",
            "Tài khoản chưa có bút toán",
        ),
        Error::UnknownTrip(_) => make(
            StatusCode::NOT_FOUND,
            "unknown-trip",
            "Chuyến chưa có thanh toán",
        ),
        Error::Unavailable(_) => {
            error!(%err, "hạ tầng tạm thời không sẵn sàng");
            Problem::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "service-unavailable",
                "Dịch vụ tạm thời không sẵn sàng",
                "thử lại sau",
                instance,
            )
        }
        Error::Storage(_) => internal(),
    }
}

#[cfg(test)]
mod tests {
    use ledger_domain::Money;

    use super::*;

    fn status(err: Error) -> u16 {
        problem_from_error(&err, "/v1/x").status
    }

    #[test]
    fn domain_errors_get_distinct_http_codes() {
        assert_eq!(
            status(Error::Ledger(LedgerError::Unbalanced {
                sum: Money::from_dong(1)
            })),
            422
        );
        assert_eq!(status(Error::IdempotencyConflict { key: "k".into() }), 409);
        assert_eq!(status(Error::UnknownAccount("a".into())), 404);
        assert_eq!(status(Error::Unavailable("db".into())), 503);
        assert_eq!(status(Error::Storage("secret sql".into())), 500);
    }

    #[test]
    fn internal_errors_do_not_leak_details() {
        let problem = problem_from_error(&Error::Storage("SELECT * FROM secret".into()), "/v1/x");
        assert!(!problem.detail.contains("SELECT"));
    }
}
