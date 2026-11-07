//! Aggregate nhỏ: bút toán thu cước của một chuyến. Nó gom sự kiện miền trong bộ nhớ,
//! không biết outbox, transaction hay Kafka.

use ledger_core::{FeePolicy, JournalEntry, LedgerError, TripCharge, capture_entry};

use crate::events::{DomainEvent, LedgerEntryPosted, PostingDto, SCHEMA_VERSION};

/// Aggregate: một bút toán thu cước hợp lệ cùng các sự kiện nó đã sinh.
#[derive(Debug)]
pub struct TripEntry {
    entry: JournalEntry,
    events: Vec<DomainEvent>,
}

impl TripEntry {
    /// Dựng bút toán thu cước bằng `ledger-core`; chưa sinh sự kiện nào.
    ///
    /// # Errors
    /// Mọi lỗi của `capture_entry` (chính sách phí, tràn số, bút toán lệch).
    pub fn capture<F>(trip: &TripCharge, policy: &F) -> Result<TripEntry, LedgerError>
    where
        F: FeePolicy + ?Sized,
    {
        Ok(TripEntry {
            entry: capture_entry(trip, policy)?,
            events: Vec::new(),
        })
    }

    /// Bút toán bên trong, chỉ đọc.
    pub fn entry(&self) -> &JournalEntry {
        &self.entry
    }

    /// Gọi sau khi lưu trữ đã cấp mã bút toán: ghi nhận sự kiện `EntryPosted`.
    /// Chỉ ghi nhận trong bộ nhớ; không gửi đi đâu.
    pub fn mark_posted(&mut self, entry_id: &str, posted_at: &str) {
        let postings = self
            .entry
            .postings()
            .iter()
            .map(|p| PostingDto {
                account: p.account.to_string(),
                amount_dong: p.amount.dong(),
            })
            .collect();
        self.events
            .push(DomainEvent::EntryPosted(LedgerEntryPosted {
                schema_version: SCHEMA_VERSION,
                entry_id: entry_id.to_string(),
                idempotency_key: self.entry.idempotency_key().to_string(),
                postings,
                posted_at: posted_at.to_string(),
            }));
    }

    /// Lấy và xóa các sự kiện đã ghi nhận.
    pub fn take_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.events)
    }
}
