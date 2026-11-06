//! Lỗi nghiệp vụ của sổ cái, dựng bằng `thiserror`.

use thiserror::Error;

use crate::entry::EntryId;
use crate::money::Money;

/// Mọi lý do mà sổ cái từ chối một thao tác. Mỗi biến thể là một lỗi nghiệp vụ;
/// lỗi hạ tầng (đĩa, mạng) chưa có vì thư viện chưa chạm tới chúng.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LedgerError {
    /// Mã tài khoản rỗng hoặc toàn khoảng trắng.
    #[error("mã tài khoản rỗng")]
    EmptyAccountId,
    /// `idempotency_key` rỗng hoặc toàn khoảng trắng.
    #[error("idempotency_key rỗng")]
    EmptyIdempotencyKey,
    /// Bút toán phải có ít nhất 2 posting.
    #[error("bút toán cần ít nhất 2 posting, có {count}")]
    TooFewPostings {
        /// Số posting đã nhận.
        count: usize,
    },
    /// Tổng số tiền có dấu của bút toán khác 0.
    #[error("bút toán lệch, tổng bằng {sum}")]
    Unbalanced {
        /// Tổng có dấu của các posting.
        sum: Money,
    },
    /// Một phép cộng, trừ, nhân hay đổi dấu làm tràn `i64`.
    #[error("tổng tiền vượt giới hạn i64")]
    Overflow,
    /// Chính sách phí trả phí âm hoặc lớn hơn cước (vi phạm hợp đồng của `FeePolicy`).
    #[error("phí {fee} không hợp lệ với cước {fare}")]
    InvalidFee {
        /// Phí chính sách đã trả.
        fee: Money,
        /// Cước được chia.
        fare: Money,
    },
    /// Cùng `idempotency_key` nhưng nội dung bút toán khác bản đã ghi.
    #[error("key {key} đã dùng cho bút toán khác nội dung")]
    IdempotencyConflict {
        /// Key bị trùng.
        key: String,
    },
    /// Không có bút toán với mã này trong sổ.
    #[error("không có bút toán {id} trong sổ")]
    UnknownEntry {
        /// Mã bút toán cần tìm.
        id: EntryId,
    },
    /// Chuỗi không phải mã bút toán dạng `JE-000001`.
    #[error("mã bút toán không hợp lệ: {raw:?}")]
    InvalidEntryId {
        /// Chuỗi đã nhận.
        raw: String,
    },
}
