//! Cờ sẵn sàng dùng chung giữa adapter đặt cờ (consumer) và adapter đọc cờ (HTTP `/readyz`).
//! `/healthz` không đọc cờ nào: process còn sống là đủ.

use std::sync::atomic::{AtomicBool, Ordering};

/// Hai cờ quyết định `/readyz`.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_only_between_assignment_and_shutdown() {
        let ready = Readiness::default();
        assert!(!ready.is_ready());
        ready.set_assigned(true);
        assert!(ready.is_ready());
        ready.begin_shutdown();
        assert!(!ready.is_ready());
    }
}
