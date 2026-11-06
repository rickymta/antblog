//! Lỗi nghiệp vụ của sổ cái: một enum, mỗi biến thể là một lý do từ chối.

use std::error::Error;
use std::fmt;

use crate::money::Money;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerError {
    EmptyAccountId,
    EmptyIdempotencyKey,
    /// Bút toán phải có ít nhất 2 posting.
    TooFewPostings {
        count: usize,
    },
    /// Tổng số tiền có dấu khác 0.
    Unbalanced {
        sum: Money,
    },
    /// Cộng các posting làm tràn `i64`.
    Overflow,
    /// Chính sách phí trả phí âm hoặc lớn hơn cước (vi phạm hợp đồng của `FeePolicy`).
    InvalidFee {
        fee: Money,
        fare: Money,
    },
    /// Cùng `idempotency_key` nhưng nội dung bút toán khác bản đã ghi.
    IdempotencyConflict {
        key: String,
    },
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedgerError::EmptyAccountId => write!(f, "mã tài khoản rỗng"),
            LedgerError::EmptyIdempotencyKey => write!(f, "idempotency_key rỗng"),
            LedgerError::TooFewPostings { count } => {
                write!(f, "bút toán cần ít nhất 2 posting, có {count}")
            }
            LedgerError::Unbalanced { sum } => write!(f, "bút toán lệch, tổng bằng {sum}"),
            LedgerError::Overflow => write!(f, "tổng tiền vượt giới hạn i64"),
            LedgerError::InvalidFee { fee, fare } => {
                write!(f, "phí {fee} không hợp lệ với cước {fare}")
            }
            LedgerError::IdempotencyConflict { key } => {
                write!(f, "key {key} đã dùng cho bút toán khác nội dung")
            }
        }
    }
}

impl Error for LedgerError {}
