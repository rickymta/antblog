//! Composition root: nơi duy nhất ghép adapter cụ thể vào port.
//!
//! `Services` generic trên ba port; chọn `MemoryStore` hay `PgStore`, `MemoryPublisher` hay
//! `KafkaPublisher` là quyết định của người gọi (xem `main.rs`), không phải của use case.

use std::sync::Arc;

use application::{Clock, EventPublisher, GetBalance, LedgerStore, SettleTrip};

/// Các use case đã được cắm adapter. Một `Arc<S>` được chia cho mọi use case cần kho.
pub struct Services<S, P, C> {
    /// Chốt tiền một chuyến.
    pub settle: SettleTrip<S, P, C>,
    /// Tra số dư.
    pub balance: GetBalance<S>,
}

impl<S: LedgerStore, P: EventPublisher, C: Clock> Services<S, P, C> {
    /// Dựng use case từ ba adapter. Compiler kiểm `S`, `P`, `C` hiện thực đúng port.
    pub fn new(store: Arc<S>, publisher: Arc<P>, clock: Arc<C>) -> Services<S, P, C> {
        Services {
            balance: GetBalance::new(Arc::clone(&store)),
            settle: SettleTrip::new(store, publisher, clock),
        }
    }
}
