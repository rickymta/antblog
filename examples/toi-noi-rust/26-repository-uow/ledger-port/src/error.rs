//! Lỗi của cổng: nghiệp vụ tách khỏi hạ tầng.

use ledger_core::{AccountId, LedgerError};
use thiserror::Error;

/// Mọi lý do một thao tác trên repository thất bại.
#[derive(Debug, Error)]
pub enum RepoError {
    /// Lỗi nghiệp vụ của `ledger-core` (cùng key khác nội dung, tràn số...).
    #[error(transparent)]
    Domain(#[from] LedgerError),
    /// Bút toán nhắc tới tài khoản chưa được mở.
    #[error("tài khoản {0} chưa mở")]
    UnknownAccount(AccountId),
    /// Posting có số tiền bằng 0 (sổ cái không lưu dòng rỗng).
    #[error("posting tiền bằng 0 ở tài khoản {account}")]
    ZeroAmount {
        /// Tài khoản có posting bằng 0.
        account: AccountId,
    },
    /// Tính năng chưa có trong lab (bút toán đảo).
    #[error("chưa hỗ trợ: {0}")]
    Unsupported(&'static str),
    /// Lỗi hạ tầng của adapter (mất kết nối, SQL lỗi...). Cổng không biết kiểu lỗi cụ thể.
    #[error("lỗi lưu trữ")]
    Storage(#[source] Box<dyn std::error::Error + Send + Sync>),
}
