//! Validate thủ công ở biên HTTP: gom MỌI lỗi theo trường rồi mới trả một Problem.
//! Bất biến miền (tổng bằng 0, không tràn số) không kiểm ở đây: `JournalEntry::try_new` lo.

use axum::http::StatusCode;
use ledger_core::{AccountId, JournalEntry, Money, Posting};

use crate::dto::PostEntryRequest;
use crate::error_map::problem_from_ledger;
use crate::problem::{ApiError, FieldError, Problem};

/// Giới hạn của lớp API (không phải bất biến sổ cái): chặn payload quá lớn.
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

/// Từ body JSON sang `JournalEntry`, hoặc một Problem.
///
/// Lỗi hình dạng: 422 `validation-failed` kèm `errors`. Hình dạng đạt thì để `ledger-core`
/// quyết: lệch, thiếu posting, tràn số đi qua `problem_from_ledger`.
///
/// # Errors
/// Trả `Problem` (422) như mô tả ở trên.
pub fn into_entry(req: PostEntryRequest, instance: &str) -> Result<JournalEntry, ApiError> {
    let errors = field_errors(&req);
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

    fn request(key: &str, postings: Vec<PostingDto>) -> PostEntryRequest {
        PostEntryRequest {
            idempotency_key: key.to_string(),
            postings,
        }
    }

    #[test]
    fn valid_capture_becomes_entry() {
        let req = request(
            "trip:TN-0001:capture",
            vec![
                posting("customer:CU-001", -25_500),
                posting("platform:revenue", 5_100),
                posting("driver:DR-001", 20_400),
            ],
        );
        let entry = into_entry(req, "/v1/entries").unwrap();
        assert_eq!(entry.postings().len(), 3);
    }

    #[test]
    fn collects_every_field_error_not_just_the_first() {
        let req = request(
            "  ",
            vec![
                posting("", -100),
                posting("driver:DR-001", 0),
                posting("ok", 100),
            ],
        );
        let fields: Vec<(String, String)> = field_errors(&req)
            .into_iter()
            .map(|e| (e.field, e.code))
            .collect();
        assert_eq!(
            fields,
            vec![
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
    fn domain_invariant_is_not_checked_here() {
        // Hình dạng đạt nhưng lệch: field_errors rỗng, `ledger-core` mới từ chối.
        let req = request(
            "k",
            vec![
                posting("customer:CU-001", -25_500),
                posting("driver:DR-001", 20_400),
            ],
        );
        assert!(field_errors(&req).is_empty());
        let problem = into_entry(req, "/v1/entries").unwrap_err();
        assert_eq!(problem.status, 422);
        assert!(problem.type_uri.ends_with("unbalanced-entry"));
    }

    #[test]
    fn too_long_and_too_many_are_reported() {
        let long = "x".repeat(129);
        let many = (0..101).map(|i| posting(&format!("a:{i}"), 1)).collect();
        let codes: Vec<String> = field_errors(&request(&long, many))
            .into_iter()
            .map(|e| e.code)
            .collect();
        assert_eq!(codes, vec!["too_long", "too_many"]);
    }
}
