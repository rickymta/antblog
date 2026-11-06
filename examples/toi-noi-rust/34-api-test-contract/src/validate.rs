//! Validate thủ công ở biên HTTP (bài 32): gom MỌI lỗi theo trường rồi mới trả một Problem.

use axum::http::StatusCode;
use ledger_core::{AccountId, JournalEntry, Money, Posting};

use crate::api::PostEntryRequest;
use crate::problem::{ApiError, FieldError, Problem, problem_from_ledger};

const MAX_TEXT_CHARS: usize = 128;
const MAX_POSTINGS: usize = 100;

fn field_error(field: impl Into<String>, code: &str, message: &str) -> FieldError {
    FieldError {
        field: field.into(),
        code: code.to_string(),
        message: message.to_string(),
    }
}

// Kiểm một chuỗi định danh: không rỗng, không quá dài.
fn check_text(field: String, raw: &str, errors: &mut Vec<FieldError>) {
    if raw.trim().is_empty() {
        errors.push(field_error(field, "blank", "không được để trống"));
    } else if raw.chars().count() > MAX_TEXT_CHARS {
        errors.push(field_error(field, "too_long", "tối đa 128 ký tự"));
    }
}

/// Kiểm hình dạng của request, trả mọi lỗi tìm được (rỗng nghĩa là đạt).
pub fn field_errors(req: &PostEntryRequest) -> Vec<FieldError> {
    let mut errors = Vec::new();
    check_text(
        "idempotencyKey".to_string(),
        &req.idempotency_key,
        &mut errors,
    );
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

/// Từ body JSON sang `JournalEntry`: lỗi hình dạng là 422 `validation-failed` kèm `errors`,
/// hình dạng đạt thì để `ledger-core` quyết.
pub fn into_entry(req: &PostEntryRequest, instance: &str) -> Result<JournalEntry, ApiError> {
    let errors = field_errors(req);
    if !errors.is_empty() {
        let detail = format!("{} trường không hợp lệ", errors.len());
        return Err(Problem::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation-failed",
            "Dữ liệu không hợp lệ",
            detail,
            instance,
        )
        .with_errors(errors)
        .into());
    }
    let postings = req
        .postings
        .iter()
        .map(|p| {
            let account = AccountId::new(&p.account)?;
            Ok(Posting::new(account, Money::from_dong(p.amount_dong)))
        })
        .collect::<Result<Vec<_>, ledger_core::LedgerError>>()
        .map_err(|e| problem_from_ledger(&e, instance))?;
    let entry = JournalEntry::try_new(&req.idempotency_key, postings)
        .map_err(|e| problem_from_ledger(&e, instance))?;
    Ok(entry)
}
