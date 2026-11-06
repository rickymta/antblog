//! Trạng thái dùng chung giữa các task: bộ đếm và cờ sẵn sàng.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Bộ đếm để test và `/readyz` quan sát; không phải metric chuẩn (bài 66 mới làm metric).
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

/// Hai cờ quyết định `/readyz`. `/healthz` không đọc cờ nào: process còn sống là đủ.
#[derive(Debug, Default)]
pub struct Readiness {
    assigned: AtomicBool,
    shutting_down: AtomicBool,
}

impl Readiness {
    /// Consumer vừa được gán (hoặc mất) partition.
    pub fn set_assigned(&self, value: bool) {
        self.assigned.store(value, Ordering::SeqCst);
    }

    /// Bắt đầu tắt: từ đây `/readyz` trả 503.
    pub fn begin_shutdown(&self) {
        self.shutting_down.store(true, Ordering::SeqCst);
    }

    /// Consumer đã có partition và chưa bắt đầu tắt.
    pub fn is_ready(&self) -> bool {
        self.assigned.load(Ordering::SeqCst) && !self.shutting_down.load(Ordering::SeqCst)
    }
}
