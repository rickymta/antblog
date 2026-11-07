//! Handler: đọc aggregate, để domain quyết định, commit; tranh chấp version thì đọc lại và quyết
//! định lại. Ba bước nằm trong một hàm, không phải các lớp generic của bài 52.

use domain::{DomainError, DomainEvent, Money, TripPayment};
use thiserror::Error;

use crate::command::{Command, validate};
use crate::port::{PaymentStore, PostedEntry, StoreError, Work};

/// Số lần tối đa đọc-quyết-commit cho một lệnh. Mỗi lần thua tranh chấp nghĩa là có một lệnh
/// khác vừa thắng, nên với hai lệnh cùng chuyến số lần thật không bao giờ tới mức này.
pub const MAX_ATTEMPTS: u32 = 8;

/// Kết quả xử lý một lệnh.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// Số lần đọc-quyết-commit đã chạy (1 nếu không tranh chấp).
    pub attempts: u32,
    /// Số lần thua tranh chấp version.
    pub conflicts: u32,
    /// Tên các sự kiện domain đã xảy ra; rỗng nghĩa là lệnh trùng, không đổi gì.
    pub events: Vec<&'static str>,
    /// Bút toán mới ghi.
    pub posted: Vec<PostedEntry>,
}

impl Outcome {
    /// Lệnh trùng: aggregate không đổi, không có gì để commit.
    pub fn is_noop(&self) -> bool {
        self.events.is_empty()
    }
}

/// Lỗi của use case. `is_permanent` quyết định consumer bỏ qua hay thử lại.
#[derive(Debug, Error)]
pub enum AppError {
    /// Sai hình thức.
    #[error("lệnh không hợp lệ: {0}")]
    Invalid(String),
    /// Domain từ chối.
    #[error(transparent)]
    Domain(#[from] DomainError),
    /// Kho từ chối hoặc hỏng.
    #[error(transparent)]
    Store(StoreError),
    /// Thua tranh chấp quá `MAX_ATTEMPTS` lần liên tiếp.
    #[error("thua tranh chấp version {attempts} lần liên tiếp")]
    TooManyConflicts {
        /// Số lần đã thử.
        attempts: u32,
    },
}

impl AppError {
    /// Lỗi vĩnh viễn: thử lại bao nhiêu lần cũng vẫn sai.
    pub fn is_permanent(&self) -> bool {
        match self {
            AppError::Invalid(_) | AppError::Domain(_) => true,
            AppError::Store(err) => matches!(
                err,
                StoreError::BudgetExhausted { .. } | StoreError::IdempotencyConflict { .. }
            ),
            AppError::TooManyConflicts { .. } => false,
        }
    }
}

impl From<StoreError> for AppError {
    fn from(err: StoreError) -> AppError {
        AppError::Store(err)
    }
}

// Giao lệnh cho aggregate. `match` vét cạn: thêm lệnh mới mà quên ở đây là lỗi biên dịch.
fn decide(payment: &mut TripPayment, command: &Command) -> Result<Vec<DomainEvent>, DomainError> {
    match command {
        Command::TripCompleted(ev) => payment.record_trip(ev),
        Command::PaymentCaptured(ev) => payment.record_capture(ev),
        Command::GrantPromo { grant, .. } => payment.grant_promo(grant),
        Command::Refund { n, amount_dong, .. } => {
            payment.refund(*n, Money::from_dong(*amount_dong))
        }
    }
}

/// Xử lý một lệnh. Generic trên `S`: dispatch tĩnh, không `dyn`, không framework.
///
/// # Errors
/// [`AppError`]; dùng [`AppError::is_permanent`] để biết có nên thử lại.
pub async fn handle<S: PaymentStore>(store: &S, command: &Command) -> Result<Outcome, AppError> {
    validate(command)?;
    let trip_id = command.trip_id();
    let mut conflicts = 0;
    for attempt in 1..=MAX_ATTEMPTS {
        let (mut payment, expected_version) = match store.load(trip_id).await? {
            Some(loaded) => (loaded.payment, loaded.version),
            None => (TripPayment::new(trip_id), 0),
        };
        let events = decide(&mut payment, command)?;
        if events.is_empty() {
            return Ok(Outcome {
                attempts: attempt,
                conflicts,
                events: Vec::new(),
                posted: Vec::new(),
            });
        }
        let names = events.iter().map(DomainEvent::name).collect();
        let entries = events.iter().flat_map(DomainEvent::entries).collect();
        let work = Work {
            payment,
            expected_version,
            entries,
        };
        match store.commit(work).await {
            Ok(committed) => {
                return Ok(Outcome {
                    attempts: attempt,
                    conflicts,
                    events: names,
                    posted: committed.posted,
                });
            }
            Err(StoreError::VersionConflict) => conflicts += 1,
            Err(other) => return Err(other.into()),
        }
    }
    Err(AppError::TooManyConflicts {
        attempts: MAX_ATTEMPTS,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::future::Future;
    use std::pin::pin;
    use std::sync::Mutex;
    use std::task::{Context, Poll, Waker};

    use domain::{PaymentCaptured, TripCompleted};

    use super::*;
    use crate::port::{Committed, Loaded};

    // Kho trong bộ nhớ: chứng minh use case chạy được mà không cần PostgreSQL (port là trait).
    #[derive(Default)]
    struct MemStore {
        state: Mutex<HashMap<String, (TripPayment, i64)>>,
        // Dữ kiện của «bên thắng»: lần commit đầu, nó chen vào trước rồi trả VersionConflict.
        rival: Mutex<Option<TripCompleted>>,
        always_conflict: bool,
    }

    impl PaymentStore for MemStore {
        async fn load(&self, trip_id: &str) -> Result<Option<Loaded>, StoreError> {
            Ok(self
                .state
                .lock()
                .unwrap()
                .get(trip_id)
                .map(|(p, v)| Loaded {
                    payment: p.clone(),
                    version: *v,
                }))
        }

        async fn commit(&self, work: Work) -> Result<Committed, StoreError> {
            let mut state = self.state.lock().unwrap();
            if let Some(rival) = self.rival.lock().unwrap().take() {
                let slot = state
                    .entry(work.payment.trip_id.clone())
                    .or_insert_with(|| (TripPayment::new(&work.payment.trip_id), 0));
                slot.0.record_trip(&rival).unwrap();
                slot.1 += 1;
                return Err(StoreError::VersionConflict);
            }
            let current = state.get(&work.payment.trip_id).map_or(0, |(_, v)| *v);
            if self.always_conflict || current != work.expected_version {
                return Err(StoreError::VersionConflict);
            }
            let posted = work
                .entries
                .iter()
                .enumerate()
                .map(|(i, e)| PostedEntry {
                    entry_id: format!("JE-{:06}", i + 1),
                    key: e.entry.idempotency_key().to_string(),
                })
                .collect();
            state.insert(work.payment.trip_id.clone(), (work.payment, current + 1));
            Ok(Committed {
                version: current + 1,
                posted,
            })
        }
    }

    // Mọi future ở đây hoàn thành ngay (khóa đồng bộ), nên vòng poll với waker rỗng là đủ.
    fn block_on<F: Future>(future: F) -> F::Output {
        let mut future = pin!(future);
        let mut cx = Context::from_waker(Waker::noop());
        loop {
            if let Poll::Ready(value) = future.as_mut().poll(&mut cx) {
                return value;
            }
        }
    }

    fn trip() -> Command {
        Command::TripCompleted(TripCompleted::sample("TN-0001", "CU-001", "DR-001", 25_500))
    }

    fn paid() -> Command {
        Command::PaymentCaptured(PaymentCaptured::sample(
            "TN-0001",
            "PAY-0001",
            "vx_ch_0001",
            25_500,
        ))
    }

    #[test]
    fn two_commands_settle_through_the_port() {
        let store = MemStore::default();
        let first = block_on(handle(&store, &trip())).unwrap();
        assert_eq!(
            (first.attempts, first.conflicts, first.posted.len()),
            (1, 0, 0)
        );
        let second = block_on(handle(&store, &paid())).unwrap();
        assert_eq!(second.events, ["CaptureRecorded", "Settled"]);
        let keys: Vec<_> = second.posted.iter().map(|p| p.key.as_str()).collect();
        assert_eq!(keys, ["trip:TN-0001:capture", "trip:TN-0001:split"]);
        assert!(block_on(handle(&store, &paid())).unwrap().is_noop());
    }

    #[test]
    fn losing_the_version_race_reads_again_and_still_settles() {
        let store = MemStore::default();
        *store.rival.lock().unwrap() =
            Some(TripCompleted::sample("TN-0001", "CU-001", "DR-001", 25_500));
        let outcome = block_on(handle(&store, &paid())).unwrap();
        assert_eq!((outcome.attempts, outcome.conflicts), (2, 1));
        assert_eq!(outcome.events, ["CaptureRecorded", "Settled"]);
        assert_eq!(outcome.posted.len(), 2);
    }

    #[test]
    fn errors_are_classified_for_the_consumer() {
        let store = MemStore::default();
        let bad = Command::Refund {
            trip_id: "TN-0001".into(),
            n: 0,
            amount_dong: 10,
        };
        let invalid = block_on(handle(&store, &bad)).unwrap_err();
        assert!(matches!(invalid, AppError::Invalid(_)) && invalid.is_permanent());
        let early = Command::Refund {
            trip_id: "TN-0001".into(),
            n: 1,
            amount_dong: 10,
        };
        assert!(block_on(handle(&store, &early)).unwrap_err().is_permanent());
        let stubborn = MemStore {
            always_conflict: true,
            ..MemStore::default()
        };
        let err = block_on(handle(&stubborn, &trip())).unwrap_err();
        assert!(matches!(
            err,
            AppError::TooManyConflicts {
                attempts: MAX_ATTEMPTS
            }
        ));
        assert!(!err.is_permanent());
    }
}
