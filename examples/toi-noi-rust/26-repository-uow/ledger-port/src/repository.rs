//! Hai trait của cổng: `LedgerRepository` mở giao dịch, `LedgerUnitOfWork` là giao dịch đang mở.

use std::future::Future;

use ledger_core::{AccountId, EntryId, JournalEntry, Money};

use crate::error::RepoError;

/// Kết quả ghi một bút toán.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PostOutcome {
    /// Mã `JE-xxxxxx` của bút toán trong sổ.
    pub id: EntryId,
    /// `true` nếu `idempotency_key` đã có và ta trả lại bút toán cũ.
    pub replayed: bool,
}

/// Cổng ra kho lưu trữ sổ cái. Adapter: bộ nhớ (`MemoryRepository`), PostgreSQL (`ledger-pg`).
///
/// Trả về `impl Future + Send` thay vì `async fn` trần: trait nói thẳng "future này gửi
/// được sang luồng khác", nên `tokio::spawn` dùng được với kiểu generic `R`.
pub trait LedgerRepository: Send + Sync {
    /// Giao dịch đang mở của adapter này.
    type Uow: LedgerUnitOfWork;

    /// Mở một Unit of Work (với sqlx: `BEGIN`).
    fn begin(&self) -> impl Future<Output = Result<Self::Uow, RepoError>> + Send;

    /// Số dư đã commit của một tài khoản, đọc ngoài giao dịch.
    fn balance(&self, account: &AccountId)
    -> impl Future<Output = Result<Money, RepoError>> + Send;
}

/// Một đơn vị công việc: mọi `post` trong cùng Uow cùng thành công hoặc cùng bị hủy.
///
/// Thả (`drop`) Uow mà chưa `commit` là rollback.
pub trait LedgerUnitOfWork: Send {
    /// Ghi bút toán, idempotent theo `idempotency_key`, và cập nhật số dư trong cùng giao dịch.
    fn post(
        &mut self,
        entry: &JournalEntry,
    ) -> impl Future<Output = Result<PostOutcome, RepoError>> + Send;

    /// Số dư của tài khoản, kể cả những gì Uow này vừa ghi mà chưa commit.
    fn balance(
        &mut self,
        account: &AccountId,
    ) -> impl Future<Output = Result<Money, RepoError>> + Send;

    /// Chốt giao dịch. Nhận `self` theo giá trị: sau commit không dùng lại Uow được.
    fn commit(self) -> impl Future<Output = Result<(), RepoError>> + Send;
}

/// Kiểm tra mà MỌI adapter phải làm giống nhau trước khi chạm kho lưu trữ.
///
/// # Errors
/// [`RepoError::Unsupported`] với bút toán đảo; [`RepoError::ZeroAmount`] với posting bằng 0.
pub fn validate_entry(entry: &JournalEntry) -> Result<(), RepoError> {
    if entry.reverses().is_some() {
        return Err(RepoError::Unsupported("bút toán đảo"));
    }
    if let Some(p) = entry.postings().iter().find(|p| p.amount == Money::ZERO) {
        return Err(RepoError::ZeroAmount {
            account: p.account.clone(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod plain_async_demo {
    use ledger_core::{AccountId, Money};

    // `async fn` trần trong trait (ổn định từ Rust 1.75): biên dịch được và dùng generic được.
    trait PlainRepo {
        async fn balance(&self, account: &AccountId) -> Money;
    }

    struct Zero;

    impl PlainRepo for Zero {
        async fn balance(&self, _account: &AccountId) -> Money {
            Money::ZERO
        }
    }

    async fn use_plain<R: PlainRepo>(repo: &R) -> Money {
        repo.balance(&AccountId::new("customer:CU-001").unwrap())
            .await
    }

    // Lỗi E0038 nếu bỏ comment (trait có `async fn` không dyn-compatible):
    // fn boxed() -> Box<dyn PlainRepo> { Box::new(Zero) }
    //
    // "future cannot be sent between threads safely" nếu bỏ comment (trait không hứa `Send`):
    // fn spawn_it<R: PlainRepo + Send + Sync + 'static>(repo: std::sync::Arc<R>) {
    //     tokio::spawn(async move { use_plain(&*repo).await });
    // }

    #[tokio::test]
    async fn plain_async_fn_works_when_not_spawned() {
        assert_eq!(use_plain(&Zero).await, Money::ZERO);
    }
}
