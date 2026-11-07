//! Use case của sổ cái. Mọi thứ ở đây chạy được với một adapter giả: không cần PostgreSQL.

use std::sync::Arc;

use ledger_domain::chart::kind_of;
use ledger_domain::clock::Clock;
use ledger_domain::hold::NewHold;
use ledger_domain::trip::{TripFare, plan_capture};
use ledger_domain::{AccountId, FeePolicy, JournalEntry, Money, Posting};

use crate::error::Error;
use crate::model::{Applied, Funds, HoldOutcome, PostOutcome};
use crate::ports::LedgerStore;

/// `TripCompleted` sau khi adapter đã giải mã (JSON, schema version) xong. Use case không thấy JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TripCompletedCmd {
    /// Định danh để khử trùng ở inbox.
    pub event_id: String,
    /// Mã chuyến.
    pub trip_id: String,
    /// Mã khách.
    pub customer_id: String,
    /// Mã tài xế.
    pub driver_id: String,
    /// Cước, đồng.
    pub fare_dong: i64,
}

/// Các use case của sổ cái. Chung một `S: LedgerStore`, một đồng hồ và một chính sách phí.
pub struct Ledger<S> {
    store: S,
    clock: Arc<dyn Clock>,
    fee: Arc<dyn FeePolicy + Send + Sync>,
}

fn account(raw: &str) -> Result<AccountId, Error> {
    Ok(AccountId::new(raw)?)
}

impl<S: LedgerStore> Ledger<S> {
    /// Lắp use case từ port và hai phụ thuộc thuần (đồng hồ, chính sách phí).
    pub fn new(store: S, clock: Arc<dyn Clock>, fee: Arc<dyn FeePolicy + Send + Sync>) -> Self {
        Ledger { store, clock, fee }
    }

    /// Thời điểm hiện tại theo đồng hồ của use case (HTTP và gRPC dùng cho `asOf`).
    pub fn now(&self) -> jiff::Timestamp {
        self.clock.now()
    }

    /// Chuyến hoàn thành → hai bút toán (capture, split) → payments, hold, số dư, outbox.
    ///
    /// # Errors
    /// [`Error::Trip`]/[`Error::Ledger`] khi sự kiện sai nội dung; lỗi của kho khi hạ tầng hỏng.
    pub async fn on_trip_completed(&self, cmd: &TripCompletedCmd) -> Result<Applied, Error> {
        let trip = TripFare::new(
            &cmd.trip_id,
            &cmd.customer_id,
            &cmd.driver_id,
            cmd.fare_dong,
        )?;
        let plan = plan_capture(&trip, self.fee.as_ref())?;
        self.store
            .apply_trip(&cmd.event_id, &trip, &plan, self.clock.now())
            .await
    }

    /// Ghi một bút toán thủ công (`POST /v1/entries`, gRPC `PostEntry`).
    ///
    /// # Errors
    /// [`Error::Invalid`] với posting 0 đồng hoặc tài khoản ngoài sơ đồ; [`Error::Ledger`] khi lệch.
    pub async fn post_entry(
        &self,
        key: &str,
        lines: &[(String, i64)],
    ) -> Result<PostOutcome, Error> {
        let mut postings = Vec::with_capacity(lines.len());
        for (raw, dong) in lines {
            // `ledger-core` nhận posting 0 đồng; CHECK của bảng thì không. Chặn ở biên use case.
            if *dong == 0 {
                return Err(Error::Invalid(format!("posting 0 đồng ở {raw}")));
            }
            let id = account(raw)?;
            if kind_of(&id).is_none() {
                return Err(Error::Invalid(format!(
                    "tài khoản ngoài sơ đồ chuẩn: {raw}"
                )));
            }
            postings.push(Posting::new(id, Money::from_dong(*dong)));
        }
        let entry = JournalEntry::try_new(key, postings)?;
        self.store.post_entry(&entry).await
    }

    /// Số dư sổ cái.
    ///
    /// # Errors
    /// [`Error::UnknownAccount`] nếu tài khoản chưa có bút toán nào.
    pub async fn balance(&self, raw: &str) -> Result<Money, Error> {
        let id = account(raw)?;
        self.store
            .balance(&id)
            .await?
            .ok_or_else(|| Error::UnknownAccount(raw.to_string()))
    }

    /// Số dư và hold còn hiệu lực theo đồng hồ của use case.
    ///
    /// # Errors
    /// [`Error::UnknownAccount`] nếu tài khoản chưa có bút toán nào.
    pub async fn funds(&self, raw: &str) -> Result<Funds, Error> {
        let id = account(raw)?;
        self.store
            .funds(&id, self.clock.now())
            .await?
            .ok_or_else(|| Error::UnknownAccount(raw.to_string()))
    }

    /// Giữ `amount_dong` trong ví, hạn `ttl_secs` giây tính từ đồng hồ của use case.
    ///
    /// # Errors
    /// [`Error::Hold`] khi không đủ khả dụng hoặc yêu cầu sai; [`Error::UnknownAccount`].
    pub async fn place_hold(
        &self,
        raw: &str,
        amount_dong: i64,
        key: &str,
        ttl_secs: i64,
    ) -> Result<HoldOutcome, Error> {
        let now = self.clock.now();
        let hold = NewHold::new(
            account(raw)?,
            amount_dong,
            key,
            now,
            jiff::SignedDuration::from_secs(ttl_secs),
        )?;
        self.store.place_hold(&hold, now).await
    }

    /// Hoàn lần `n` một phần tiền của chuyến.
    ///
    /// # Errors
    /// [`Error::Refund`] khi vượt phần còn hoàn được; [`Error::UnknownTrip`].
    pub async fn refund(
        &self,
        trip_id: &str,
        n: u32,
        amount_dong: i64,
    ) -> Result<PostOutcome, Error> {
        self.store
            .refund(trip_id, n, Money::from_dong(amount_dong))
            .await
    }

    /// Kho còn trả lời không.
    ///
    /// # Errors
    /// Lỗi của kho.
    pub async fn ping(&self) -> Result<(), Error> {
        self.store.ping().await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use jiff::Timestamp;
    use ledger_domain::LedgerError;
    use ledger_domain::clock::ManualClock;
    use ledger_domain::default_policy;
    use ledger_domain::trip::CapturePlan;

    use super::*;

    /// Adapter giả: ghi lại lời gọi, không có I/O. Chứng minh use case chạy không cần PostgreSQL.
    #[derive(Default)]
    struct FakeStore {
        trips: Mutex<Vec<(String, CapturePlan, Timestamp)>>,
        entries: Mutex<Vec<JournalEntry>>,
    }

    impl LedgerStore for Arc<FakeStore> {
        async fn apply_trip(
            &self,
            event_id: &str,
            _trip: &TripFare,
            plan: &CapturePlan,
            now: Timestamp,
        ) -> Result<Applied, Error> {
            self.trips
                .lock()
                .unwrap()
                .push((event_id.to_string(), plan.clone(), now));
            Ok(Applied::Duplicate)
        }
        async fn post_entry(&self, entry: &JournalEntry) -> Result<PostOutcome, Error> {
            self.entries.lock().unwrap().push(entry.clone());
            Ok(PostOutcome::Replayed {
                entry_id: "JE-000001".to_string(),
            })
        }
        async fn balance(&self, _: &AccountId) -> Result<Option<Money>, Error> {
            Ok(None)
        }
        async fn funds(&self, _: &AccountId, _: Timestamp) -> Result<Option<Funds>, Error> {
            Ok(None)
        }
        async fn place_hold(&self, _: &NewHold, _: Timestamp) -> Result<HoldOutcome, Error> {
            Ok(HoldOutcome::Created { hold_id: 1 })
        }
        async fn refund(&self, _: &str, _: u32, _: Money) -> Result<PostOutcome, Error> {
            Err(Error::UnknownTrip("TN-0001".to_string()))
        }
        async fn ping(&self) -> Result<(), Error> {
            Ok(())
        }
    }

    fn ledger() -> (Ledger<Arc<FakeStore>>, Arc<FakeStore>) {
        let store = Arc::new(FakeStore::default());
        let ledger = Ledger::new(
            store.clone(),
            Arc::new(ManualClock::lab_start()),
            Arc::new(default_policy()),
        );
        (ledger, store)
    }

    fn cmd(fare_dong: i64) -> TripCompletedCmd {
        TripCompletedCmd {
            event_id: "TripCompleted:TN-0001".to_string(),
            trip_id: "TN-0001".to_string(),
            customer_id: "CU-001".to_string(),
            driver_id: "DR-001".to_string(),
            fare_dong,
        }
    }

    #[tokio::test]
    async fn trip_reaches_the_store_as_two_entries_stamped_by_the_clock() {
        let (ledger, store) = ledger();
        ledger.on_trip_completed(&cmd(25_500)).await.unwrap();
        let trips = store.trips.lock().unwrap();
        let (event_id, plan, now) = &trips[0];
        assert_eq!(event_id, "TripCompleted:TN-0001");
        assert_eq!(plan.capture.idempotency_key(), "trip:TN-0001:capture");
        assert_eq!(plan.split.idempotency_key(), "trip:TN-0001:split");
        assert_eq!(now.to_string(), "2026-11-06T08:00:00Z");
    }

    #[tokio::test]
    async fn bad_input_never_reaches_the_store() {
        let (ledger, store) = ledger();
        assert!(
            ledger
                .on_trip_completed(&cmd(0))
                .await
                .unwrap_err()
                .is_permanent()
        );
        let zero = [
            ("customer:CU-001".to_string(), 0),
            ("platform:revenue".to_string(), 0),
        ];
        assert!(matches!(
            ledger.post_entry("k", &zero).await,
            Err(Error::Invalid(_))
        ));
        let unknown = [
            ("lạ:1".to_string(), -5),
            ("platform:revenue".to_string(), 5),
        ];
        assert!(matches!(
            ledger.post_entry("k", &unknown).await,
            Err(Error::Invalid(_))
        ));
        let skewed = [
            ("customer:CU-001".to_string(), -5),
            ("platform:revenue".to_string(), 4),
        ];
        assert!(matches!(
            ledger.post_entry("k", &skewed).await,
            Err(Error::Ledger(LedgerError::Unbalanced { .. }))
        ));
        assert!(store.trips.lock().unwrap().is_empty());
        assert!(store.entries.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn missing_account_is_a_distinct_error() {
        let (ledger, _) = ledger();
        assert_eq!(
            ledger.balance("driver:DR-404").await.unwrap_err(),
            Error::UnknownAccount("driver:DR-404".to_string())
        );
    }
}
