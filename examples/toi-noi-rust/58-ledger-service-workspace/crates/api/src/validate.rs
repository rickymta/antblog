//! Validate hình dạng ở biên HTTP: gom MỌI lỗi theo trường rồi mới trả một Problem (bài 32).
//! Bất biến miền (tổng bằng 0, không tràn số, đủ khả dụng) không kiểm ở đây: use case lo.

use axum::http::StatusCode;

use crate::dto::{HoldRequest, PostEntryRequest, RefundRequest};
use crate::problem::{ApiError, FieldError, Problem};

/// Giới hạn của lớp API (không phải bất biến sổ cái): chặn payload quá lớn.
const MAX_TEXT_CHARS: usize = 128;
const MAX_POSTINGS: usize = 100;
/// Hạn tối đa của một hold: 24 giờ.
const MAX_TTL_SECS: i64 = 86_400;
/// Hạn mặc định của một hold: 15 phút.
pub const DEFAULT_TTL_SECS: i64 = 900;

fn field_error(field: impl Into<String>, code: &str, message: &str) -> FieldError {
    FieldError {
        field: field.into(),
        code: code.to_string(),
        message: message.to_string(),
    }
}

// Kiểm một chuỗi định danh: không rỗng, không quá dài.
fn check_text(field: impl Into<String>, raw: &str, errors: &mut Vec<FieldError>) {
    if raw.trim().is_empty() {
        errors.push(field_error(field, "blank", "không được để trống"));
    } else if raw.chars().count() > MAX_TEXT_CHARS {
        errors.push(field_error(field, "too_long", "tối đa 128 ký tự"));
    }
}

/// Lỗi hình dạng của `POST /v1/entries`.
pub fn entry_errors(req: &PostEntryRequest) -> Vec<FieldError> {
    let mut errors = Vec::new();
    check_text("idempotencyKey", &req.idempotency_key, &mut errors);
    if req.postings.len() > MAX_POSTINGS {
        errors.push(field_error("postings", "too_many", "tối đa 100 dòng"));
    }
    for (i, posting) in req.postings.iter().enumerate() {
        check_text(
            format!("postings[{i}].account"),
            &posting.account,
            &mut errors,
        );
        if posting.amount_dong == 0 {
            errors.push(field_error(
                format!("postings[{i}].amountDong"),
                "zero_amount",
                "số tiền phải khác 0",
            ));
        }
    }
    errors
}

/// Lỗi hình dạng của `POST /v1/holds`.
pub fn hold_errors(req: &HoldRequest) -> Vec<FieldError> {
    let mut errors = Vec::new();
    check_text("account", &req.account, &mut errors);
    check_text("idempotencyKey", &req.idempotency_key, &mut errors);
    if req.amount_dong <= 0 {
        errors.push(field_error(
            "amountDong",
            "not_positive",
            "số tiền giữ phải dương",
        ));
    }
    if let Some(ttl) = req.ttl_seconds
        && !(1..=MAX_TTL_SECS).contains(&ttl)
    {
        errors.push(field_error(
            "ttlSeconds",
            "out_of_range",
            "từ 1 đến 86400 giây",
        ));
    }
    errors
}

/// Lỗi hình dạng của `POST /v1/trips/{id}/refunds`.
pub fn refund_errors(req: &RefundRequest) -> Vec<FieldError> {
    let mut errors = Vec::new();
    if req.refund_no == 0 {
        errors.push(field_error("refundNo", "not_positive", "bắt đầu từ 1"));
    }
    if req.amount_dong <= 0 {
        errors.push(field_error(
            "amountDong",
            "not_positive",
            "số hoàn phải dương",
        ));
    }
    errors
}

/// Có lỗi thì đóng thành một Problem 422 `validation-failed`.
///
/// # Errors
/// Problem 422 kèm danh sách `errors`.
pub fn ensure_valid(errors: Vec<FieldError>, instance: &str) -> Result<(), ApiError> {
    if errors.is_empty() {
        return Ok(());
    }
    let detail = format!("{} trường không hợp lệ", errors.len());
    Err(Problem::new(
        StatusCode::UNPROCESSABLE_ENTITY,
        "validation-failed",
        "Dữ liệu không hợp lệ",
        detail,
        instance,
    )
    .with_errors(errors)
    .into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::PostingDto;

    fn posting(account: &str, amount_dong: i64) -> PostingDto {
        PostingDto {
            account: account.to_string(),
            amount_dong,
        }
    }

    #[test]
    fn collects_every_field_error_not_just_the_first() {
        let req = PostEntryRequest {
            idempotency_key: "  ".to_string(),
            postings: vec![
                posting("", -100),
                posting("driver:DR-001", 0),
                posting("ok", 100),
            ],
        };
        let fields: Vec<(String, String)> = entry_errors(&req)
            .into_iter()
            .map(|e| (e.field, e.code))
            .collect();
        assert_eq!(
            fields,
            [
                ("idempotencyKey".to_string(), "blank".to_string()),
                ("postings[0].account".to_string(), "blank".to_string()),
                (
                    "postings[1].amountDong".to_string(),
                    "zero_amount".to_string()
                ),
            ]
        );
    }

    #[test]
    fn hold_and_refund_shapes() {
        let hold = HoldRequest {
            account: "customer:CU-001".to_string(),
            amount_dong: 0,
            idempotency_key: "trip:TN-0001:hold".to_string(),
            ttl_seconds: Some(0),
        };
        let codes: Vec<String> = hold_errors(&hold).into_iter().map(|e| e.field).collect();
        assert_eq!(codes, ["amountDong", "ttlSeconds"]);
        let refund = RefundRequest {
            refund_no: 0,
            amount_dong: -1,
        };
        assert_eq!(refund_errors(&refund).len(), 2);
    }
}
