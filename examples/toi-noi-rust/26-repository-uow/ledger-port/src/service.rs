//! Service ứng dụng: biết nghiệp vụ "thu cước một chuyến", không biết kho lưu trữ nào.

use ledger_core::{FeePolicy, TripCharge, capture_entry};

use crate::error::RepoError;
use crate::repository::{LedgerRepository, LedgerUnitOfWork, PostOutcome};

/// Thu cước một chuyến: dựng bút toán, ghi trong một Unit of Work rồi commit.
///
/// Generic theo `R`: dùng được với bộ nhớ lẫn PostgreSQL. Nếu `post` lỗi, `?` trả lỗi và
/// `uow` bị thả trước `commit`, tức rollback; hàm không cần code dọn dẹp.
///
/// # Errors
/// Lỗi dựng bút toán (`RepoError::Domain`) hoặc lỗi của repository.
pub async fn post_trip_capture<R, P>(
    repo: &R,
    trip: &TripCharge,
    policy: &P,
) -> Result<PostOutcome, RepoError>
where
    R: LedgerRepository,
    P: FeePolicy + ?Sized,
{
    let entry = capture_entry(trip, policy)?;
    let mut uow = repo.begin().await?;
    let outcome = uow.post(&entry).await?;
    uow.commit().await?;
    Ok(outcome)
}
