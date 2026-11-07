//! Decorator của `PaymentStore` để DỰNG cuộc đua version một cách tất định: mỗi bên phải đọc xong
//! (cùng version cũ) rồi mới được đi tiếp tới commit. Không ngủ, không đoán thời gian.

use std::sync::atomic::{AtomicUsize, Ordering};

use application::{Committed, Loaded, PaymentStore, StoreError, Work};
use tokio::sync::Barrier;

/// Bọc một kho: `parties` lần `load` đầu tiên chờ nhau ở `Barrier`; các lần sau (đọc lại sau khi thua) đi thẳng.
pub struct Gated<S> {
    inner: S,
    barrier: Barrier,
    loads: AtomicUsize,
    parties: usize,
}

impl<S> Gated<S> {
    /// Bọc `inner`; `parties` là số lệnh cùng chạy.
    pub fn new(inner: S, parties: usize) -> Gated<S> {
        Gated {
            inner,
            barrier: Barrier::new(parties),
            loads: AtomicUsize::new(0),
            parties,
        }
    }
}

impl<S: PaymentStore + Sync> PaymentStore for Gated<S> {
    async fn load(&self, trip_id: &str) -> Result<Option<Loaded>, StoreError> {
        let loaded = self.inner.load(trip_id).await;
        if self.loads.fetch_add(1, Ordering::SeqCst) < self.parties {
            self.barrier.wait().await; // mọi bên đã đọc version cũ rồi mới cùng đi tiếp
        }
        loaded
    }

    async fn commit(&self, work: Work) -> Result<Committed, StoreError> {
        self.inner.commit(work).await
    }
}
