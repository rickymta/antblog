//! Port lưu trữ: use case chỉ cần hai việc, đọc aggregate kèm version và commit một đơn vị công việc.
//! Cả việc ghi bút toán, số dư và outbox nằm sau `commit`: adapter phải làm chúng trong một transaction.

use std::error::Error;
use std::future::Future;

use domain::{PlannedEntry, TripPayment};
use thiserror::Error;

/// Aggregate đã lưu kèm version đọc được.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loaded {
    /// Trạng thái.
    pub payment: TripPayment,
    /// Version lúc đọc; commit phải nói đúng version này.
    pub version: i64,
}

/// Một đơn vị công việc: trạng thái mới, version kỳ vọng và các bút toán đi kèm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Work {
    /// Trạng thái mới của aggregate.
    pub payment: TripPayment,
    /// Version đã đọc; 0 nghĩa là aggregate chưa tồn tại.
    pub expected_version: i64,
    /// Bút toán cần ghi, theo thứ tự.
    pub entries: Vec<PlannedEntry>,
}

/// Một bút toán đã ghi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostedEntry {
    /// Mã dạng `JE-000001`.
    pub entry_id: String,
    /// `idempotency_key`.
    pub key: String,
}

/// Kết quả commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Committed {
    /// Version mới của aggregate.
    pub version: i64,
    /// Bút toán mới ghi (bút toán trùng key và trùng nội dung không tính).
    pub posted: Vec<PostedEntry>,
}

/// Lỗi của kho. Tách ba nhóm: tranh chấp (thử lại), sai nghiệp vụ (đừng thử lại), hạ tầng (thử lại sau).
#[derive(Debug, Error)]
pub enum StoreError {
    /// Có người commit trước: version đã khác hoặc aggregate đã được tạo.
    #[error("aggregate đã bị bên khác đổi, đọc lại rồi quyết định lại")]
    VersionConflict,
    /// Ngân sách campaign không đủ.
    #[error("{account} còn {available} đồng, cần {requested}")]
    BudgetExhausted {
        /// Tài khoản ngân sách.
        account: String,
        /// Số cần.
        requested: i64,
        /// Số còn.
        available: i64,
    },
    /// Cùng `idempotency_key` nhưng khác nội dung bản đã ghi.
    #[error("idempotency_key {key} đã dùng cho bút toán khác nội dung")]
    IdempotencyConflict {
        /// Key bị trùng.
        key: String,
    },
    /// Lỗi hạ tầng (database, mạng): thử lại được.
    #[error("lỗi hạ tầng: {0}")]
    Infra(Box<dyn Error + Send + Sync>),
}

impl StoreError {
    /// Gói một lỗi hạ tầng bất kỳ.
    pub fn infra(err: impl Error + Send + Sync + 'static) -> StoreError {
        StoreError::Infra(Box::new(err))
    }
}

/// Kho lưu trữ aggregate và sổ. Trả `impl Future + Send` để dùng được với `tokio::spawn`
/// mà crate này không cần biết tokio.
pub trait PaymentStore {
    /// Đọc aggregate của một chuyến; `None` nếu chưa có dữ kiện nào.
    fn load(
        &self,
        trip_id: &str,
    ) -> impl Future<Output = Result<Option<Loaded>, StoreError>> + Send;

    /// Ghi aggregate (kiểm `expected_version`), bút toán, số dư và outbox trong MỘT transaction.
    fn commit(&self, work: Work) -> impl Future<Output = Result<Committed, StoreError>> + Send;
}
