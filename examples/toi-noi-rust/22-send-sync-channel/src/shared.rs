//! Chia sẻ một `Ledger` giữa nhiều task bằng khóa.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use ledger_core::{JournalEntry, Ledger, LedgerError};

use crate::{PostOutcome, post_outcome};

/// Sổ cái dùng chung: `Arc` cho nhiều chủ, `Mutex` cho quyền ghi tuần tự.
pub type SharedLedger = Arc<Mutex<Ledger>>;

/// Lỗi của đường ghi qua khóa.
#[derive(Debug, PartialEq, Eq)]
pub enum SharedError {
    /// Một task khác từng panic khi giữ khóa (poisoning, xem bài 16).
    Poisoned,
    /// Lỗi nghiệp vụ của sổ cái.
    Ledger(LedgerError),
}

/// Tạo sổ cái rỗng đặt sau `Arc<Mutex<_>>`.
pub fn new_shared() -> SharedLedger {
    Arc::new(Mutex::new(Ledger::new()))
}

/// Ghi một bút toán. Guard sống trong hàm đồng bộ này và không bao giờ qua `.await`.
///
/// # Errors
/// [`SharedError::Poisoned`] hoặc lỗi của [`Ledger::post`].
pub fn post_shared(shared: &SharedLedger, entry: JournalEntry) -> Result<PostOutcome, SharedError> {
    let mut ledger = shared.lock().map_err(|_| SharedError::Poisoned)?;
    post_outcome(&mut ledger, entry).map_err(SharedError::Ledger)
    // `ledger` (guard) nhả khóa ở đây.
}

/// Gọi VíXanh giả: chỉ chờ một chút, không có I/O thật.
async fn call_vixanh() {
    tokio::time::sleep(Duration::from_millis(1)).await;
}

/// Ghi xong mới gọi VíXanh: guard nằm trong khối riêng nên hết sống trước `.await`.
/// Nhờ vậy future vẫn `Send` và `tokio::spawn` nhận được.
///
/// # Errors
/// Như [`post_shared`].
pub async fn post_then_notify(
    shared: &SharedLedger,
    entry: JournalEntry,
) -> Result<PostOutcome, SharedError> {
    let outcome = {
        let mut ledger = shared.lock().map_err(|_| SharedError::Poisoned)?;
        post_outcome(&mut ledger, entry).map_err(SharedError::Ledger)?
    };
    call_vixanh().await;
    Ok(outcome)
}

/// Biến thể giữ khóa suốt lúc gọi VíXanh: cần `tokio::sync::Mutex`, có guard `Send`.
/// Mọi task khác phải chờ cả thời gian gọi mạng; chỉ dùng khi thật sự cần.
///
/// # Errors
/// Lỗi của [`Ledger::post`].
pub async fn post_holding_across_await(
    shared: &tokio::sync::Mutex<Ledger>,
    entry: JournalEntry,
) -> Result<PostOutcome, LedgerError> {
    let mut ledger = shared.lock().await;
    let outcome = post_outcome(&mut ledger, entry)?;
    call_vixanh().await; // guard vẫn sống qua đây
    Ok(outcome)
}

/// Kết quả của một loạt task cùng ghi một key.
#[derive(Debug, PartialEq, Eq)]
pub struct Race {
    /// Số bút toán trong sổ sau cùng.
    pub entries: usize,
    /// Số task nhận kết quả «ghi mới».
    pub created: usize,
    /// Số task nhận kết quả «key đã có».
    pub replayed: usize,
    /// Mã bút toán mà các task nhận được, sắp xếp, bỏ trùng.
    pub ids: Vec<String>,
}

/// `tasks` task `tokio::spawn` cùng ghi bút toán `trip:TN-0001:capture` vào một sổ.
///
/// # Errors
/// Lỗi dựng bút toán, lỗi ghi hoặc task bị panic.
pub async fn race_same_key(tasks: usize) -> Result<Race, Box<dyn std::error::Error>> {
    let shared = new_shared();
    let mut handles = Vec::new();
    for _ in 0..tasks {
        let shared = Arc::clone(&shared);
        let entry = crate::fixtures::tn_0001_entry()?;
        handles.push(tokio::spawn(async move {
            post_then_notify(&shared, entry).await
        }));
    }
    let mut outcomes = Vec::new();
    for handle in handles {
        outcomes.push(handle.await?.map_err(|e| format!("{e:?}"))?);
    }
    let entries = shared.lock().map_err(|_| "poisoned")?.len();
    let created = outcomes.iter().filter(|o| !o.replayed).count();
    let mut ids: Vec<String> = outcomes.iter().map(|o| o.id.to_string()).collect();
    ids.sort();
    ids.dedup();
    Ok(Race {
        entries,
        created,
        replayed: outcomes.len() - created,
        ids,
    })
}
