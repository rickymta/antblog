//! Bộ đếm để test và vận hành quan sát; không phải metric chuẩn (metric thuộc phần vận hành).

use std::sync::atomic::{AtomicU64, Ordering};

/// Bộ đếm hoạt động của consumer và relay.
#[derive(Debug, Default)]
pub struct Stats {
    /// `TripCompleted` đã ghi thành bút toán mới.
    pub processed: AtomicU64,
    /// Sự kiện đã có trong inbox, bị bỏ qua.
    pub duplicates: AtomicU64,
    /// Sự kiện sai nội dung, bị bỏ qua (chưa có DLQ).
    pub poison: AtomicU64,
    /// Dòng outbox relay đã phát và đánh dấu.
    pub relayed: AtomicU64,
}

impl Stats {
    /// Đọc một bộ đếm.
    pub fn get(counter: &AtomicU64) -> u64 {
        counter.load(Ordering::SeqCst)
    }

    /// Tăng một bộ đếm.
    pub fn bump(counter: &AtomicU64, by: u64) {
        counter.fetch_add(by, Ordering::SeqCst);
    }
}
