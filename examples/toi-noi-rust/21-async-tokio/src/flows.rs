//! Các luồng gọi VíXanh: tuần tự và `join!`, `timeout`, `select!`, `spawn`, thử lại có
//! idempotency key, và bắt lô capture dừng bằng `CancellationToken`.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ledger_core::{
    AccountId, EntryId, Ledger, LedgerError, Money, TripCharge, capture_entry, default_policy,
    fare_for_km,
};
use tokio::task::JoinHandle;
use tokio::time::{Instant, sleep, timeout};
use tokio_util::sync::CancellationToken;

use crate::vixanh::{AuthId, PspError, Receipt, Vixanh};

/// Một chuyến cần thanh toán.
#[derive(Debug, Clone)]
pub struct Trip {
    /// Mã chuyến, ví dụ `TN-0001`.
    pub id: String,
    /// Cước.
    pub fare: Money,
}

impl Trip {
    /// Chuyến `id` đi `km` km theo bảng cước học tập.
    pub fn new(id: &str, km: u32) -> Trip {
        Trip {
            id: id.to_string(),
            fare: fare_for_km(km),
        }
    }

    /// Idempotency key của lần thu tiền: một chuyến, một key, mọi lần thử.
    pub fn capture_key(&self) -> String {
        format!("trip:{}:capture", self.id)
    }
}

/// Lỗi của một lần gọi có giới hạn thời gian.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CallError {
    /// Hết hạn trước khi VíXanh trả lời. Chưa biết VíXanh đã xử lý hay chưa.
    #[error("hết {0:?} mà VíXanh chưa trả lời")]
    Timeout(Duration),
    /// VíXanh trả lỗi.
    #[error(transparent)]
    Psp(#[from] PspError),
}

/// Ba chuyến mẫu: authorize mất 120, 300 và 200 ms.
pub fn demo_trips() -> (Vixanh, [Trip; 3]) {
    let psp = Vixanh::builder()
        .authorize_latency("TN-0001", Duration::from_millis(120))
        .authorize_latency("TN-0002", Duration::from_millis(300))
        .authorize_latency("TN-0003", Duration::from_millis(200))
        .build();
    let trips = [
        Trip::new("TN-0001", 5),
        Trip::new("TN-0002", 3),
        Trip::new("TN-0003", 8),
    ];
    (psp, trips)
}

type AuthResult = Result<AuthId, PspError>;

/// Authorize lần lượt từng chuyến: tổng thời gian là TỔNG các độ trễ.
pub async fn authorize_sequential(psp: &Vixanh, trips: &[Trip; 3]) -> ([AuthResult; 3], Duration) {
    let start = Instant::now();
    let a = psp.authorize(&trips[0].id, trips[0].fare).await;
    let b = psp.authorize(&trips[1].id, trips[1].fare).await;
    let c = psp.authorize(&trips[2].id, trips[2].fare).await;
    ([a, b, c], start.elapsed())
}

/// Authorize đồng thời bằng `join!`: tổng thời gian là độ trễ LỚN NHẤT.
pub async fn authorize_join(psp: &Vixanh, trips: &[Trip; 3]) -> ([AuthResult; 3], Duration) {
    let start = Instant::now();
    let (a, b, c) = tokio::join!(
        psp.authorize(&trips[0].id, trips[0].fare),
        psp.authorize(&trips[1].id, trips[1].fare),
        psp.authorize(&trips[2].id, trips[2].fare),
    );
    ([a, b, c], start.elapsed())
}

/// `try_join!`: một future lỗi thì trả lỗi ngay và HỦY các future còn lại (drop).
pub async fn authorize_try_join(
    psp: &Vixanh,
    trips: &[Trip; 3],
) -> (Result<[AuthId; 3], PspError>, Duration) {
    let start = Instant::now();
    let result = tokio::try_join!(
        psp.authorize(&trips[0].id, trips[0].fare),
        psp.authorize(&trips[1].id, trips[1].fare),
        psp.authorize(&trips[2].id, trips[2].fare),
    )
    .map(|(a, b, c)| [a, b, c]);
    (result, start.elapsed())
}

/// Authorize có hạn `limit` bằng `tokio::time::timeout`.
pub async fn authorize_with_timeout(
    psp: &Vixanh,
    trip: &Trip,
    limit: Duration,
) -> Result<AuthId, CallError> {
    match timeout(limit, psp.authorize(&trip.id, trip.fare)).await {
        Ok(result) => Ok(result?),
        Err(_elapsed) => Err(CallError::Timeout(limit)),
    }
}

/// Cùng việc đó bằng `select!`: nhánh nào xong trước thì thắng, nhánh còn lại bị drop.
pub async fn authorize_with_select(
    psp: &Vixanh,
    trip: &Trip,
    limit: Duration,
) -> Result<AuthId, CallError> {
    tokio::select! {
        result = psp.authorize(&trip.id, trip.fare) => Ok(result?),
        () = sleep(limit) => Err(CallError::Timeout(limit)),
    }
}

/// Authorize từng chuyến trong một task riêng (`spawn`) rồi chờ các `JoinHandle`.
/// Runtime xếp lịch task ngay khi `spawn` (không poll đồng bộ trong lời gọi đó); không cần
/// `.await` handle thì task mới bắt đầu.
pub async fn authorize_spawned(psp: &Vixanh, trips: &[Trip]) -> (Vec<AuthResult>, Duration) {
    let start = Instant::now();
    let handles: Vec<JoinHandle<AuthResult>> = trips
        .iter()
        .map(|trip| {
            // `spawn` đòi future `'static`: đưa bản sở hữu (clone) vào, không mượn biến cục bộ.
            let psp = psp.clone();
            let trip = trip.clone();
            tokio::spawn(async move { psp.authorize(&trip.id, trip.fare).await })
        })
        .collect();
    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        results.push(handle.await.expect("task authorize không panic"));
    }
    (results, start.elapsed())
}

/// Cách đặt idempotency key khi thử lại.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyMode {
    /// Một key cho cả chuyến (`trip:TN-0001:capture`): đúng.
    Stable,
    /// Mỗi lần thử một key mới: sai, VíXanh coi là hai giao dịch.
    PerAttempt,
}

/// Kết quả thanh toán một chuyến.
#[derive(Debug)]
pub struct Payment {
    /// Biên lai cuối cùng nhận được.
    pub receipt: Receipt,
    /// Số lần gọi `capture`.
    pub attempts: u32,
    /// Thời gian từ đầu tới lúc có biên lai.
    pub elapsed: Duration,
}

/// Authorize rồi capture; mỗi lần capture có hạn `limit`, hết hạn thì thử lại tối đa `max_attempts` lần.
///
/// # Errors
/// [`CallError::Timeout`] nếu mọi lần thử đều hết hạn; lỗi VíXanh thì trả ngay, không thử lại.
pub async fn pay_trip(
    psp: &Vixanh,
    trip: &Trip,
    limit: Duration,
    max_attempts: u32,
    mode: KeyMode,
) -> Result<Payment, CallError> {
    let start = Instant::now();
    let auth = authorize_with_timeout(psp, trip, limit).await?;
    for attempt in 1..=max_attempts {
        let key = match mode {
            KeyMode::Stable => trip.capture_key(),
            KeyMode::PerAttempt => format!("{}:attempt-{attempt}", trip.capture_key()),
        };
        // Hết hạn thì future `capture` bị drop; phía VíXanh có thể đã trừ tiền rồi.
        if let Ok(result) = timeout(limit, psp.capture(&auth, &key, trip.fare)).await {
            return Ok(Payment {
                receipt: result?,
                attempts: attempt,
                elapsed: start.elapsed(),
            });
        }
    }
    Err(CallError::Timeout(limit))
}

/// Tập chuyến đang thu tiền, để không thu hai lần đồng thời trong một process.
#[derive(Clone, Default)]
pub struct InFlight {
    keys: Arc<Mutex<HashSet<String>>>,
}

impl InFlight {
    /// Đánh dấu `key` đang chạy; `false` nếu đã có.
    pub fn enter(&self, key: &str) -> bool {
        self.keys
            .lock()
            .expect("khóa không bị poison")
            .insert(key.to_string())
    }

    /// Bỏ đánh dấu.
    pub fn leave(&self, key: &str) {
        self.keys.lock().expect("khóa không bị poison").remove(key);
    }

    /// Còn key nào kẹt không.
    pub fn is_empty(&self) -> bool {
        self.keys.lock().expect("khóa không bị poison").is_empty()
    }

    /// Đánh dấu và trả một guard tự bỏ đánh dấu khi bị drop.
    pub fn guard(&self, key: &str) -> Option<InFlightGuard> {
        self.enter(key).then(|| InFlightGuard {
            set: self.clone(),
            key: key.to_string(),
        })
    }
}

/// Bỏ đánh dấu trong `Drop`: chạy cả khi future bị hủy giữa chừng.
pub struct InFlightGuard {
    set: InFlight,
    key: String,
}

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        self.set.leave(&self.key);
    }
}

/// Bản KHÔNG an toàn khi hủy: nếu future bị drop ở `.await`, `leave` không bao giờ chạy.
pub async fn capture_naive(
    psp: &Vixanh,
    in_flight: &InFlight,
    auth: &AuthId,
    trip: &Trip,
) -> Option<Result<Receipt, PspError>> {
    let key = trip.capture_key();
    if !in_flight.enter(&key) {
        return None;
    }
    let result = psp.capture(auth, &key, trip.fare).await;
    in_flight.leave(&key);
    Some(result)
}

/// Bản an toàn khi hủy: guard dọn dẹp trong `Drop`.
pub async fn capture_guarded(
    psp: &Vixanh,
    in_flight: &InFlight,
    auth: &AuthId,
    trip: &Trip,
) -> Option<Result<Receipt, PspError>> {
    let key = trip.capture_key();
    let _guard = in_flight.guard(&key)?;
    Some(psp.capture(auth, &key, trip.fare).await)
}

/// Kết quả một lô capture bị dừng giữa chừng.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct BatchReport {
    /// Chuyến đã có biên lai.
    pub completed: Vec<String>,
    /// Chuyến đang gửi/chờ thì bị hủy: KHÔNG biết VíXanh đã trừ tiền chưa.
    pub interrupted: Option<String>,
    /// Chuyến chưa được gửi đi.
    pub not_started: Vec<String>,
}

/// Capture lần lượt từng chuyến cho tới khi hết chuyến hoặc `token` bị hủy.
pub async fn capture_batch(psp: &Vixanh, trips: &[Trip], token: &CancellationToken) -> BatchReport {
    let mut report = BatchReport::default();
    for (index, trip) in trips.iter().enumerate() {
        // Dừng êm: kiểm tra giữa hai chuyến, chuyến nào chưa gửi thì không gửi.
        if token.is_cancelled() {
            report.not_started = trips[index..].iter().map(|t| t.id.clone()).collect();
            break;
        }
        let auth = AuthId::for_trip(&trip.id);
        let key = trip.capture_key();
        // Dừng cứng: đang chờ VíXanh mà bị hủy thì drop future ngay.
        tokio::select! {
            result = psp.capture(&auth, &key, trip.fare) => {
                if result.is_ok() {
                    report.completed.push(trip.id.clone());
                }
            }
            () = token.cancelled() => {
                report.interrupted = Some(trip.id.clone());
                report.not_started = trips[index + 1..].iter().map(|t| t.id.clone()).collect();
                break;
            }
        }
    }
    report
}

/// Ghi bút toán thu cước vào sổ cái sau khi ĐÃ có biên lai. Key bút toán trùng key gửi VíXanh,
/// nên ghi lại biên lai cũ chỉ là replay.
///
/// # Errors
/// Lỗi của `ledger-core` (tài khoản không hợp lệ, xung đột idempotency...).
pub fn post_capture(
    ledger: &mut Ledger,
    trip: &Trip,
    customer: &str,
    driver: &str,
) -> Result<(EntryId, bool), LedgerError> {
    let charge = TripCharge {
        trip_id: trip.id.clone(),
        customer: AccountId::new(customer)?,
        driver: AccountId::new(driver)?,
        fare: trip.fare,
    };
    let posted = ledger.post(capture_entry(&charge, &default_policy())?)?;
    Ok((posted.id, posted.replayed))
}
