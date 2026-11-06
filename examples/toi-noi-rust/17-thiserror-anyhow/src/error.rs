//! Lỗi nghiệp vụ (domain) của sổ cái. `thiserror` sinh `Display` và `Error` thay cho
//! hai khối `impl` viết tay ở lab 10.

use thiserror::Error;

use crate::money::Money;

/// Mỗi biến thể là một lý do sổ cái từ chối bút toán. Không chứa lỗi hạ tầng.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LedgerError {
    #[error("mã tài khoản rỗng")]
    EmptyAccountId,
    #[error("idempotency_key rỗng")]
    EmptyIdempotencyKey,
    /// Bút toán phải có ít nhất 2 posting.
    #[error("bút toán cần ít nhất 2 posting, có {count}")]
    TooFewPostings { count: usize },
    /// Vượt giới hạn posting lấy từ cấu hình.
    #[error("bút toán có {count} posting, vượt giới hạn {max}")]
    TooManyPostings { count: usize, max: usize },
    /// Tổng số tiền có dấu khác 0.
    #[error("bút toán lệch, tổng bằng {sum}")]
    Unbalanced { sum: Money },
    /// Cộng các posting làm tràn `i64`.
    #[error("tổng tiền vượt giới hạn i64")]
    Overflow,
    /// Cùng `idempotency_key` nhưng nội dung bút toán khác bản đã ghi.
    #[error("key {key} đã dùng cho bút toán khác nội dung")]
    IdempotencyConflict { key: String },
}

impl LedgerError {
    /// Mã HTTP tương ứng. Lab chưa có HTTP; bảng này để sau này API dùng lại.
    /// `match` không có nhánh `_`: thêm biến thể mới là compiler bắt buộc chọn mã.
    pub fn http_status(&self) -> u16 {
        match self {
            LedgerError::IdempotencyConflict { .. } => 409,
            LedgerError::EmptyAccountId
            | LedgerError::EmptyIdempotencyKey
            | LedgerError::TooFewPostings { .. }
            | LedgerError::TooManyPostings { .. }
            | LedgerError::Unbalanced { .. }
            | LedgerError::Overflow => 422,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as _;

    #[test]
    fn display_matches_hand_written_messages() {
        assert_eq!(LedgerError::EmptyAccountId.to_string(), "mã tài khoản rỗng");
        assert_eq!(
            LedgerError::Unbalanced {
                sum: Money::from_dong(-500)
            }
            .to_string(),
            "bút toán lệch, tổng bằng -500 VND"
        );
        assert_eq!(
            LedgerError::TooFewPostings { count: 1 }.to_string(),
            "bút toán cần ít nhất 2 posting, có 1"
        );
    }

    #[test]
    fn domain_errors_have_no_source() {
        assert!(LedgerError::Overflow.source().is_none());
    }

    #[test]
    fn http_status_table() {
        assert_eq!(LedgerError::Overflow.http_status(), 422);
        assert_eq!(
            LedgerError::IdempotencyConflict { key: "k".into() }.http_status(),
            409
        );
    }
}
