//! Đổi lỗi của sổ cái thành `tonic::Status`. Mỗi biến thể một mã, không có nhánh `_`:
//! thêm biến thể mới vào `LedgerError` thì file này không biên dịch cho tới khi chọn mã cho nó.

use ledger_core::LedgerError;
use tonic::{Code, Status};

use crate::store::StoreError;

/// Mã gRPC cho lỗi nghiệp vụ của `ledger-core`.
pub fn ledger_error_status(e: &LedgerError) -> Status {
    let message = e.to_string();
    match e {
        // Request sai dù sổ ở trạng thái nào: sửa request rồi gửi lại.
        LedgerError::EmptyAccountId
        | LedgerError::EmptyIdempotencyKey
        | LedgerError::TooFewPostings { .. }
        | LedgerError::Unbalanced { .. }
        | LedgerError::Overflow
        | LedgerError::InvalidEntryId { .. } => Status::invalid_argument(message),
        // Key đã gắn với bút toán khác nội dung: xung đột với thứ đã có.
        LedgerError::IdempotencyConflict { .. } => Status::already_exists(message),
        // Không có bút toán với mã này.
        LedgerError::UnknownEntry { .. } => Status::not_found(message),
        // Chính sách phí trả số sai là lỗi của service, không phải của client.
        LedgerError::InvalidFee { .. } => Status::internal("lỗi nội bộ"),
    }
}

impl From<StoreError> for Status {
    fn from(e: StoreError) -> Status {
        match &e {
            StoreError::Domain(inner) => ledger_error_status(inner),
            StoreError::ZeroAmount(_) => Status::invalid_argument(e.to_string()),
            // Request hợp lệ nhưng sổ chưa ở trạng thái nhận nó: mở tài khoản rồi gửi lại y nguyên.
            StoreError::UnknownAccount(_) => Status::failed_precondition(e.to_string()),
            // PostgreSQL đã rollback: client được phép gửi lại cả request.
            StoreError::Aborted(_) => Status::aborted(e.to_string()),
            StoreError::Unavailable(_) => Status::unavailable(e.to_string()),
            // Không đưa chi tiết hạ tầng (tên bảng, SQL) ra ngoài dây.
            StoreError::Storage(_) => Status::internal("lỗi lưu trữ"),
        }
    }
}

/// Bảng đối chiếu (nhãn, mã) cho mọi lỗi mà hai RPC có thể trả: dùng cho test và để in trong demo.
pub fn mapping_table() -> Vec<(&'static str, Code)> {
    use ledger_core::{EntryId, Money};
    let account = "customer:CU-001".to_string();
    let rows: Vec<(&'static str, StoreError)> = vec![
        (
            "Unbalanced",
            LedgerError::Unbalanced {
                sum: Money::from_dong(1),
            }
            .into(),
        ),
        (
            "TooFewPostings",
            LedgerError::TooFewPostings { count: 1 }.into(),
        ),
        (
            "EmptyIdempotencyKey",
            LedgerError::EmptyIdempotencyKey.into(),
        ),
        ("ZeroAmount", StoreError::ZeroAmount(account.clone())),
        (
            "IdempotencyConflict",
            LedgerError::IdempotencyConflict { key: "k".into() }.into(),
        ),
        ("UnknownAccount", StoreError::UnknownAccount(account)),
        ("Aborted (40001)", StoreError::Aborted("40001".into())),
        ("Unavailable", StoreError::Unavailable("pool".into())),
        ("Storage", StoreError::Storage("sql".into())),
        (
            "UnknownEntry",
            LedgerError::UnknownEntry {
                id: EntryId::parse("JE-000001").expect("mã hợp lệ"),
            }
            .into(),
        ),
    ];
    rows.into_iter()
        .map(|(label, e)| (label, Status::from(e).code()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::is_contention;

    #[test]
    fn each_error_gets_the_documented_code() {
        let table: std::collections::BTreeMap<_, _> = mapping_table().into_iter().collect();
        assert_eq!(table["Unbalanced"], Code::InvalidArgument);
        assert_eq!(table["TooFewPostings"], Code::InvalidArgument);
        assert_eq!(table["EmptyIdempotencyKey"], Code::InvalidArgument);
        assert_eq!(table["ZeroAmount"], Code::InvalidArgument);
        assert_eq!(table["IdempotencyConflict"], Code::AlreadyExists);
        assert_eq!(table["UnknownAccount"], Code::FailedPrecondition);
        assert_eq!(table["Aborted (40001)"], Code::Aborted);
        assert_eq!(table["Unavailable"], Code::Unavailable);
        assert_eq!(table["Storage"], Code::Internal);
        assert_eq!(table["UnknownEntry"], Code::NotFound);
    }

    #[test]
    fn internal_status_does_not_leak_cause() {
        let status = Status::from(StoreError::Storage(
            "relation \"secret_table\" missing".into(),
        ));
        assert_eq!(status.message(), "lỗi lưu trữ");
    }

    #[test]
    fn only_serialization_failure_and_deadlock_are_contention() {
        assert!(is_contention("40001"));
        assert!(is_contention("40P01"));
        assert!(
            !is_contention("23505"),
            "unique_violation không phải tranh chấp"
        );
        assert!(
            !is_contention("57014"),
            "query_canceled không phải tranh chấp"
        );
    }
}
