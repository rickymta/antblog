//! Service sổ cái tối giản: nhận yêu cầu ghi bút toán thu cước và log có cấu trúc.

use std::sync::{Arc, Mutex};

use ledger_core::{
    AccountId, Ledger, LedgerError, TripCharge, capture_entry, default_policy, fare_for_km,
};
use tracing::{Instrument, Span, debug, info, info_span, instrument};

/// Yêu cầu từ Payment. Có số điện thoại và số thẻ (dữ liệu giả) để thấy PII có lọt vào log hay không.
#[derive(Debug, Clone)]
pub struct CaptureRequest {
    pub request_id: String,
    pub trip_id: String,
    pub km: u32,
    pub customer_phone: String,
    pub card_number: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Outcome {
    pub entry_id: String,
    pub replayed: bool,
}

/// Chọn cách chạy tác vụ con để thấy cả lỗi lẫn cách sửa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanPropagation {
    /// Quên: `tokio::spawn(fut)` trần, task con mất span cha.
    Forgotten,
    /// Đúng: `fut.instrument(Span::current())` gắn span hiện tại vào task con.
    Attached,
}

#[derive(Clone)]
pub struct LedgerService {
    ledger: Arc<Mutex<Ledger>>,
    propagation: SpanPropagation,
}

impl LedgerService {
    pub fn new(propagation: SpanPropagation) -> LedgerService {
        LedgerService {
            ledger: Arc::new(Mutex::new(Ledger::new())),
            propagation,
        }
    }

    /// Điểm vào của một request: mở span `request` mang `request_id`, mọi thứ bên trong thừa hưởng.
    pub async fn handle(&self, req: CaptureRequest) -> Result<Outcome, LedgerError> {
        let span = info_span!("request", request_id = %req.request_id);
        self.process(req).instrument(span).await
    }

    async fn process(&self, req: CaptureRequest) -> Result<Outcome, LedgerError> {
        info!("nhận PaymentCaptured");
        let key = format!("trip:{}:capture", req.trip_id);
        let outcome = self.post_entry(&key, &req).await?;
        // Task con là future độc lập: phải mang span theo, nếu không event của nó mất request_id.
        let driver = "DR-001".to_string();
        let task = match self.propagation {
            SpanPropagation::Forgotten => tokio::spawn(notify_driver(driver)),
            SpanPropagation::Attached => {
                tokio::spawn(notify_driver(driver).instrument(Span::current()))
            }
        };
        if task.await.is_err() {
            tracing::warn!("task thông báo bị panic");
        }
        Ok(outcome)
    }

    // `skip(self, req, key)`: không ghi Debug của request (có số điện thoại, số thẻ);
    // `key` đã có trong `fields(idempotency_key = ...)` nên bỏ để khỏi ghi hai lần.
    // `err(Display)`: nếu trả Err thì tự phát event ERROR kèm thông điệp lỗi.
    #[instrument(skip(self, req, key), fields(idempotency_key = %key, trip_id = %req.trip_id), err(Display))]
    async fn post_entry(&self, key: &str, req: &CaptureRequest) -> Result<Outcome, LedgerError> {
        let charge = TripCharge {
            trip_id: req.trip_id.clone(),
            customer: AccountId::new("customer:CU-001")?,
            driver: AccountId::new("driver:DR-001")?,
            fare: fare_for_km(req.km),
        };
        let entry = capture_entry(&charge, &default_policy())?;
        debug!(postings = entry.postings().len(), "đã dựng bút toán");
        // Giả lập chờ I/O (ghi database) trước khi khoá sổ.
        tokio::task::yield_now().await;
        // Khoá std Mutex rồi thả ngay trong khối này, không giữ qua `.await`.
        let outcome = {
            let mut ledger = self.ledger.lock().expect("sổ cái bị poison");
            let posted = ledger.post(entry)?;
            Outcome {
                entry_id: posted.id.to_string(),
                replayed: posted.replayed,
            }
        };
        info!(entry_id = %outcome.entry_id, replayed = outcome.replayed, "đã ghi bút toán");
        Ok(outcome)
    }
}

async fn notify_driver(driver: String) {
    info!(driver = %driver, "đã đẩy thông báo cho tài xế");
}

/// Bản SAI để minh họa: giữ guard của `span.enter()` qua `.await`.
pub async fn hold_enter_guard_across_await(request_id: &str) {
    let span = info_span!("request", request_id = %request_id);
    let _guard = span.enter();
    tokio::task::yield_now().await;
    info!("tiếp tục sau await");
}

/// Bản ĐÚNG: `.instrument(span)` vào/ra span mỗi lần future được poll.
pub async fn instrument_the_future(request_id: &str) {
    let span = info_span!("request", request_id = %request_id);
    async {
        tokio::task::yield_now().await;
        info!("tiếp tục sau await");
    }
    .instrument(span)
    .await;
}

/// Bản SAI: `#[instrument]` trần ghi `Debug` của mọi tham số, gồm cả PII.
#[instrument]
pub fn audit_leaky(req: &CaptureRequest) {
    info!("kiểm toán yêu cầu");
}

/// Bản ĐÚNG: bỏ tham số nhạy cảm, chỉ ghi trường chọn lọc.
#[instrument(skip(req), fields(trip_id = %req.trip_id))]
pub fn audit_safe(req: &CaptureRequest) {
    info!("kiểm toán yêu cầu");
}
