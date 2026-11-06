//! Các port ra ngoài: kho sổ cái, nơi phát sự kiện, đồng hồ.

use std::error::Error as StdError;
use std::future::Future;

use domain::{AccountId, EntryId, EntryPosted, JournalEntry, LedgerError, Money};
use thiserror::Error;

/// Lỗi của kho sổ cái. Lỗi của công nghệ cụ thể (sqlx, ...) được gói vào `Storage`.
#[derive(Debug, Error)]
pub enum StoreError {
    /// Cùng `idempotency_key` nhưng nội dung khác bút toán đã ghi.
    #[error("idempotency_key `{0}` đã dùng cho một bút toán khác")]
    IdempotencyConflict(String),
    /// Bút toán nhắc tới tài khoản chưa có trong sổ.
    #[error("tài khoản `{0}` chưa tồn tại")]
    UnknownAccount(String),
    /// Lỗi bất biến sổ cái (tràn số, ...).
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    /// Lỗi hạ tầng: application không biết và không cần biết nó là lỗi gì.
    #[error("lỗi kho lưu trữ: {0}")]
    Storage(Box<dyn StdError + Send + Sync>),
}

/// Lỗi khi phát sự kiện.
#[derive(Debug, Error)]
#[error("không phát được sự kiện: {0}")]
pub struct PublishError(pub Box<dyn StdError + Send + Sync>);

/// Kết quả ghi một bút toán.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PostOutcome {
    /// Mã bút toán trong sổ.
    pub id: EntryId,
    /// `true` nếu `idempotency_key` đã có và ta nhận lại bút toán cũ.
    pub replayed: bool,
}

/// Port kho sổ cái. Adapter: `MemoryStore`, `PgStore`.
pub trait LedgerStore: Send + Sync {
    /// Ghi một bút toán, idempotent theo `idempotency_key`; cập nhật số dư cùng lúc.
    fn post(
        &self,
        entry: &JournalEntry,
    ) -> impl Future<Output = Result<PostOutcome, StoreError>> + Send;

    /// Số dư đã ghi của một tài khoản (0 nếu chưa có dòng nào).
    fn balance(
        &self,
        account: &AccountId,
    ) -> impl Future<Output = Result<Money, StoreError>> + Send;
}

/// Port phát sự kiện miền. Adapter: `MemoryPublisher`, `KafkaPublisher`.
pub trait EventPublisher: Send + Sync {
    /// Phát một sự kiện; trả về khi nơi nhận đã xác nhận.
    fn publish(&self, event: &EntryPosted)
    -> impl Future<Output = Result<(), PublishError>> + Send;
}

/// Port đồng hồ. Thời gian đi vào use case qua trait này nên test dùng đồng hồ cố định.
pub trait Clock: Send + Sync {
    /// Giây Unix UTC hiện tại.
    fn now_unix(&self) -> i64;
}
