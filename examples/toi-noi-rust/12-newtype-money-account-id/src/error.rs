//! Các loại lỗi: lỗi tiền, lỗi mã định danh, lỗi nghiệp vụ của sổ cái.

use std::error::Error;
use std::fmt;

use crate::money::Money;

/// Lỗi của phép tính và chuyển đổi tiền.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoneyError {
    /// Cộng hoặc trừ làm tràn `i64`.
    Overflow,
    /// Giá trị rộng hơn (vd `i128`) không nằm trong phạm vi `i64`.
    OutOfRange,
}

impl fmt::Display for MoneyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoneyError::Overflow => write!(f, "phép tính tiền vượt giới hạn i64"),
            MoneyError::OutOfRange => write!(f, "giá trị nằm ngoài phạm vi i64"),
        }
    }
}

impl Error for MoneyError {}

/// Lỗi khi dựng `AccountId` hoặc `EntryId` từ dữ liệu bên ngoài.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdError {
    Empty,
    TooLong {
        max: usize,
    },
    /// Ký tự ngoài `A-Z a-z 0-9 - :` (kể cả khoảng trắng đầu/cuối).
    InvalidChar {
        ch: char,
    },
    /// Mã tài khoản cần dạng `kind:tên`, mọi đoạn đều không rỗng.
    BadShape,
    /// Mã bút toán cần dạng `JE-000001`, số thứ tự từ 1.
    BadEntryId,
}

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdError::Empty => write!(f, "mã rỗng"),
            IdError::TooLong { max } => write!(f, "mã dài hơn {max} ký tự"),
            IdError::InvalidChar { ch } => write!(f, "ký tự {ch:?} không được phép"),
            IdError::BadShape => write!(f, "mã tài khoản phải dạng kind:tên, không đoạn rỗng"),
            IdError::BadEntryId => write!(f, "mã bút toán phải dạng JE-000001, số thứ tự từ 1"),
        }
    }
}

impl Error for IdError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerError {
    InvalidAccountId(IdError),
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
    /// Số tiền nằm ngoài phạm vi `i64` khi chuyển từ kiểu rộng hơn.
    AmountOutOfRange,
    /// Cùng `idempotency_key` nhưng nội dung bút toán khác bản đã ghi.
    IdempotencyConflict {
        key: String,
    },
}

// `From` cho phép toán tử `?` tự đổi lỗi nhỏ thành lỗi lớn.
impl From<MoneyError> for LedgerError {
    fn from(e: MoneyError) -> LedgerError {
        match e {
            MoneyError::Overflow => LedgerError::Overflow,
            MoneyError::OutOfRange => LedgerError::AmountOutOfRange,
        }
    }
}

impl From<IdError> for LedgerError {
    fn from(e: IdError) -> LedgerError {
        LedgerError::InvalidAccountId(e)
    }
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedgerError::InvalidAccountId(e) => write!(f, "mã tài khoản không hợp lệ: {e}"),
            LedgerError::EmptyIdempotencyKey => write!(f, "idempotency_key rỗng"),
            LedgerError::TooFewPostings { count } => {
                write!(f, "bút toán cần ít nhất 2 posting, có {count}")
            }
            LedgerError::Unbalanced { sum } => write!(f, "bút toán lệch, tổng bằng {sum}"),
            LedgerError::Overflow => write!(f, "tổng tiền vượt giới hạn i64"),
            LedgerError::AmountOutOfRange => write!(f, "số tiền nằm ngoài phạm vi i64"),
            LedgerError::IdempotencyConflict { key } => {
                write!(f, "key {key} đã dùng cho bút toán khác nội dung")
            }
        }
    }
}

impl Error for LedgerError {}
