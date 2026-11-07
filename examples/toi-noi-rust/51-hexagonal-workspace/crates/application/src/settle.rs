//! Use case "chốt tiền một chuyến": ghi capture và split, phát sự kiện cho bút toán mới.

use std::sync::Arc;

use domain::{AccountId, EntryId, EntryPosted, JournalEntry, Money, Trip, TripError};
use thiserror::Error;

use crate::ports::{Clock, EventPublisher, LedgerStore, PublishError, StoreError};

/// Lỗi của use case: gộp lỗi miền và lỗi hai port.
#[derive(Debug, Error)]
pub enum SettleError {
    /// Chuyến không dựng được bút toán.
    #[error(transparent)]
    Trip(#[from] TripError),
    /// Kho lỗi.
    #[error(transparent)]
    Store(#[from] StoreError),
    /// Bút toán đã ghi nhưng sự kiện chưa phát được (cần outbox, xem bài 37).
    #[error(transparent)]
    Publish(#[from] PublishError),
}

/// Kết quả của một bước.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepReceipt {
    /// Khoá idempotency của bút toán.
    pub key: String,
    /// Mã bút toán.
    pub entry_id: EntryId,
    /// `true` nếu bút toán đã có từ trước, nên không phát lại sự kiện.
    pub replayed: bool,
}

/// Kết quả cả chuyến: hai bước và số dư các tài khoản liên quan sau khi ghi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettleReceipt {
    /// Capture rồi split.
    pub steps: Vec<StepReceipt>,
    /// Số dư theo thứ tự `customer`, `clearing`, `revenue`, `driver`.
    pub balances: Vec<(AccountId, Money)>,
}

/// Use case chốt tiền chuyến, generic trên ba port (static dispatch, không `dyn`).
pub struct SettleTrip<S, P, C> {
    store: Arc<S>,
    publisher: Arc<P>,
    clock: Arc<C>,
}

impl<S: LedgerStore, P: EventPublisher, C: Clock> SettleTrip<S, P, C> {
    /// Nhận các adapter đã dựng sẵn từ composition root.
    pub fn new(store: Arc<S>, publisher: Arc<P>, clock: Arc<C>) -> Self {
        SettleTrip {
            store,
            publisher,
            clock,
        }
    }

    /// Ghi capture, rồi split. Gọi lại với cùng chuyến thì trả `replayed = true` và không phát lại.
    ///
    /// Hai bước là hai lần ghi riêng, mỗi lần idempotent: chạy lại sau lỗi giữa chừng sẽ ghi nốt bước thiếu.
    pub async fn execute(&self, trip: &Trip) -> Result<SettleReceipt, SettleError> {
        let settlement = trip.settlement()?;
        let mut steps = Vec::with_capacity(2);
        for entry in [&settlement.capture, &settlement.split] {
            steps.push(self.post_and_publish(entry).await?);
        }
        let mut balances = Vec::with_capacity(4);
        for entry in [&settlement.capture, &settlement.split] {
            for p in entry.postings() {
                if !balances.iter().any(|(a, _)| a == &p.account) {
                    balances.push((p.account.clone(), self.store.balance(&p.account).await?));
                }
            }
        }
        Ok(SettleReceipt { steps, balances })
    }

    async fn post_and_publish(&self, entry: &JournalEntry) -> Result<StepReceipt, SettleError> {
        let outcome = self.store.post(entry).await?;
        if !outcome.replayed {
            let event = EntryPosted {
                entry_id: outcome.id,
                idempotency_key: entry.idempotency_key().to_string(),
                postings: entry.postings().to_vec(),
                posted_at_unix: self.clock.now_unix(),
            };
            self.publisher.publish(&event).await?;
        }
        Ok(StepReceipt {
            key: entry.idempotency_key().to_string(),
            entry_id: outcome.id,
            replayed: outcome.replayed,
        })
    }
}

/// Use case truy vấn số dư.
pub struct GetBalance<S> {
    store: Arc<S>,
}

impl<S: LedgerStore> GetBalance<S> {
    /// Nhận kho từ composition root.
    pub fn new(store: Arc<S>) -> Self {
        GetBalance { store }
    }

    /// Số dư đã ghi của `account`.
    pub async fn execute(&self, account: &AccountId) -> Result<Money, StoreError> {
        self.store.balance(account).await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::pin;
    use std::sync::Mutex;
    use std::task::{Context, Poll, Waker};

    use super::*;
    use crate::ports::PostOutcome;

    // Executor tối thiểu: future của test không bao giờ chờ, nên poll liên tục là đủ.
    // Nhờ vậy application không cần tokio, kể cả khi test.
    fn block_on<F: Future>(f: F) -> F::Output {
        let mut f = pin!(f);
        let mut cx = Context::from_waker(Waker::noop());
        loop {
            if let Poll::Ready(v) = f.as_mut().poll(&mut cx) {
                return v;
            }
        }
    }

    // Kho giả: nhớ key đã ghi, đánh số JE-000001... theo thứ tự ghi.
    #[derive(Default)]
    struct FakeStore(Mutex<Vec<String>>);
    impl LedgerStore for FakeStore {
        async fn post(&self, entry: &JournalEntry) -> Result<PostOutcome, StoreError> {
            let mut keys = self.0.lock().unwrap();
            let key = entry.idempotency_key().to_string();
            let (index, replayed) = match keys.iter().position(|k| *k == key) {
                Some(i) => (i, true),
                None => {
                    keys.push(key);
                    (keys.len() - 1, false)
                }
            };
            let id = EntryId::parse(&format!("JE-{:06}", index + 1)).unwrap();
            Ok(PostOutcome { id, replayed })
        }
        async fn balance(&self, _: &AccountId) -> Result<Money, StoreError> {
            Ok(Money::ZERO)
        }
    }

    #[derive(Default)]
    struct CountingPublisher(Mutex<usize>);
    impl EventPublisher for CountingPublisher {
        async fn publish(&self, _: &EntryPosted) -> Result<(), PublishError> {
            *self.0.lock().unwrap() += 1;
            Ok(())
        }
    }

    struct Fixed;
    impl Clock for Fixed {
        fn now_unix(&self) -> i64 {
            0
        }
    }

    #[test]
    fn second_run_replays_and_publishes_nothing_new() {
        let publisher = Arc::new(CountingPublisher::default());
        let uc = SettleTrip::new(
            Arc::new(FakeStore::default()),
            publisher.clone(),
            Arc::new(Fixed),
        );
        let trip = Trip::new("TN-0001", "CU-001", "DR-001", 5).unwrap();

        let first = block_on(uc.execute(&trip)).unwrap();
        assert!(first.steps.iter().all(|s| !s.replayed));
        assert_eq!(*publisher.0.lock().unwrap(), 2);

        let second = block_on(uc.execute(&trip)).unwrap();
        assert!(second.steps.iter().all(|s| s.replayed));
        assert_eq!(*publisher.0.lock().unwrap(), 2);
    }
}
