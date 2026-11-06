//! `Arc<Mutex<_>>`: nhiều chủ giữa các luồng, một luồng ghi tại một thời điểm.
//! Chỉ một ví dụ nhỏ; lab này chưa dùng channel hay `RwLock`.

use std::sync::{Arc, Mutex};
use std::thread;

use crate::entry::JournalEntry;
use crate::error::LedgerError;
use crate::ledger::Ledger;
use crate::money::Money;
use crate::policy::{PercentFee, capture_entry};

/// Bút toán thu cước chuyến thứ `n`: `TN-000n`, khách `CU-00n`, tài xế `DR-00n`, cước 25.500 VND.
pub fn trip_entry(n: usize) -> Result<JournalEntry, LedgerError> {
    capture_entry(
        &format!("trip:TN-{n:04}:capture"),
        &format!("customer:CU-{n:03}"),
        &format!("driver:DR-{n:03}"),
        Money::from_dong(25_500),
        &PercentFee { percent: 20 },
    )
}

/// Mỗi chuyến ghi từ một luồng riêng vào cùng một sổ cái.
/// `Arc::clone` tăng bộ đếm nguyên tử; `move` chuyển bản clone vào luồng.
pub fn capture_in_threads(trips: usize) -> Result<Ledger, LedgerError> {
    let shared = Arc::new(Mutex::new(Ledger::new()));
    let mut handles = Vec::new();
    for n in 1..=trips {
        let shared = Arc::clone(&shared);
        handles.push(thread::spawn(move || -> Result<(), LedgerError> {
            let entry = trip_entry(n)?;
            // Guard của khóa tự nhả khi hết khối; `lock` lỗi nếu luồng khác đã panic khi giữ khóa.
            let mut ledger = shared.lock().map_err(|_| LedgerError::Poisoned)?;
            ledger.post(entry)?;
            Ok(())
        }));
    }
    for handle in handles {
        // `join` lỗi nghĩa là luồng đã panic; lớp `Result` trong là lỗi nghiệp vụ.
        handle.join().map_err(|_| LedgerError::Poisoned)??;
    }
    // Mọi luồng đã join nên chỉ còn một `Arc`; `try_unwrap` thất bại là bug.
    let mutex = Arc::try_unwrap(shared).expect("mọi luồng đã join, chỉ còn một chủ");
    mutex.into_inner().map_err(|_| LedgerError::Poisoned)
}

// Lỗi E0277 nếu thay `Arc` bằng `Rc` ở trên:
//   let shared = Rc::new(Mutex::new(Ledger::new()));
//   thread::spawn(move || { shared.lock() ... });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::AccountId;

    #[test]
    fn four_threads_post_four_entries() -> Result<(), LedgerError> {
        let ledger = capture_in_threads(4)?;
        assert_eq!(ledger.len(), 4);
        assert_eq!(ledger.grand_total()?, Money::ZERO);
        assert_eq!(
            ledger.balance(&AccountId::new("platform:revenue")?)?,
            Money::from_dong(20_400)
        );
        assert_eq!(
            ledger.balance(&AccountId::new("customer:CU-001")?)?,
            Money::from_dong(-25_500)
        );
        Ok(())
    }
}
