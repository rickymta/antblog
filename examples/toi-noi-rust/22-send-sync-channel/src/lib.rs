//! Lab bài 22: concurrency quanh `Ledger` của `ledger-core` (bài 20).
//!
//! - `send_sync`: `Send`/`Sync` kiểm lúc biên dịch, lỗi compile thật (doc test `compile_fail`).
//! - `shared`: `Arc<Mutex<Ledger>>` và `tokio::sync::Mutex`, nhiều task ghi cùng key.
//! - `deadlock`: giữ guard của `std::sync::Mutex` qua `.await` trên runtime một luồng.
//! - `readers`: `Mutex` so với `RwLock` khi đọc nhiều (chỉ chạy qua `--bench`).
//! - `actor`: một task sở hữu `Ledger`, `mpsc` bounded + `oneshot` + `broadcast`.

pub mod actor;
pub mod deadlock;
pub mod fixtures;
pub mod readers;
pub mod send_sync;
pub mod shared;

pub use ledger_core::{
    AccountId, EntryId, JournalEntry, Ledger, LedgerError, Money, Posting, TripCharge,
};

/// Kết quả ghi một bút toán, tách khỏi `Posted<'_>` (vốn mượn sổ cái) để mang qua khóa và channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PostOutcome {
    /// Mã bút toán (`JE-000001`).
    pub id: EntryId,
    /// `true` nếu key đã có và bút toán cũ được trả lại, không ghi thêm.
    pub replayed: bool,
}

/// Ghi vào sổ và copy kết quả ra khỏi phạm vi mượn.
///
/// # Errors
/// Mọi lỗi của [`Ledger::post`].
pub fn post_outcome(ledger: &mut Ledger, entry: JournalEntry) -> Result<PostOutcome, LedgerError> {
    let posted = ledger.post(entry)?;
    Ok(PostOutcome {
        id: posted.id,
        replayed: posted.replayed,
    })
}
