//! Phần nghiệp vụ của consumer: TripCompleted -> bút toán -> LedgerEntryPosted.
//!
//! Sổ cái ở đây là `Ledger` trong bộ nhớ của `ledger-core` (lab 20), đứng thay cho PostgreSQL
//! để bài tập trung vào Kafka. Nó sống lâu hơn consumer, giống cơ sở dữ liệu sống lâu hơn process.

use std::sync::{Arc, Mutex};

use anyhow::Result;
use jiff::Timestamp;
use ledger_core::{
    AccountId, Ledger, LedgerError, Money, TripCharge, capture_entry, default_policy,
};
use rdkafka::producer::FutureProducer;

use crate::events::{
    DecodeError, LedgerEntryPosted, PostingDto, SCHEMA_VERSION, TripCompleted,
    decode_trip_completed,
};
use crate::kafka::send;

/// Vì sao một thông điệp bị bỏ qua (log rồi đi tiếp, không chặn partition).
#[derive(Debug)]
pub enum SkipReason {
    /// Thông điệp hỏng hoặc version lạ.
    Decode(DecodeError),
    /// Giải mã được nhưng sổ cái từ chối (ví dụ cùng key khác nội dung).
    Rejected(LedgerError),
}

/// Kết quả xử lý một thông điệp.
#[derive(Debug)]
pub enum Outcome {
    /// Bút toán mới được ghi.
    Posted(LedgerEntryPosted),
    /// Key đã có (nhận lại thông điệp): sổ cái trả bút toán cũ, không ghi thêm.
    Replayed(LedgerEntryPosted),
    /// Không xử lý được.
    Skipped(SkipReason),
}

/// Bản tóm tắt dễ so sánh và in ra.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Summary {
    Posted(String),
    Replayed(String),
    Skipped(String),
}

impl std::fmt::Display for Summary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Summary::Posted(id) => write!(f, "ghi bút toán {id}"),
            Summary::Replayed(id) => write!(f, "nhận lại, bút toán {id} đã có"),
            Summary::Skipped(why) => write!(f, "bỏ qua: {why}"),
        }
    }
}

impl Outcome {
    /// Tóm tắt kết quả.
    pub fn summary(&self) -> Summary {
        match self {
            Outcome::Posted(e) => Summary::Posted(e.entry_id.clone()),
            Outcome::Replayed(e) => Summary::Replayed(e.entry_id.clone()),
            Outcome::Skipped(SkipReason::Decode(e)) => Summary::Skipped(e.to_string()),
            Outcome::Skipped(SkipReason::Rejected(e)) => Summary::Skipped(e.to_string()),
        }
    }
}

/// Giải mã và ghi bút toán. Không chạm Kafka, nên test được không cần broker.
#[derive(Default)]
pub struct Processor {
    ledger: Mutex<Ledger>,
}

impl Processor {
    pub fn new() -> Processor {
        Processor::default()
    }

    /// Xử lý một payload. Thông điệp xấu trả `Skipped`, không panic, không `Err`.
    pub fn handle(&self, payload: &[u8]) -> Outcome {
        let trip = match decode_trip_completed(payload) {
            Ok(trip) => trip,
            Err(err) => return Outcome::Skipped(SkipReason::Decode(err)),
        };
        match self.post(&trip) {
            Ok((event, false)) => Outcome::Posted(event),
            Ok((event, true)) => Outcome::Replayed(event),
            Err(err) => Outcome::Skipped(SkipReason::Rejected(err)),
        }
    }

    fn post(&self, trip: &TripCompleted) -> Result<(LedgerEntryPosted, bool), LedgerError> {
        let charge = TripCharge {
            trip_id: trip.trip_id.clone(),
            customer: AccountId::new(&format!("customer:{}", trip.customer_id))?,
            driver: AccountId::new(&format!("driver:{}", trip.driver_id))?,
            fare: Money::from_dong(trip.fare_dong),
        };
        let entry = capture_entry(&charge, &default_policy())?;
        let mut ledger = self.ledger.lock().unwrap();
        let posted = ledger.post(entry)?;
        let event = LedgerEntryPosted {
            schema_version: SCHEMA_VERSION,
            entry_id: posted.id.to_string(),
            idempotency_key: posted.entry.idempotency_key().to_string(),
            postings: posted
                .entry
                .postings()
                .iter()
                .map(|p| PostingDto {
                    account: p.account.as_str().to_string(),
                    amount_dong: p.amount.dong(),
                })
                .collect(),
            // Sổ trong bộ nhớ không lưu giờ ghi; lab lấy giờ phát sự kiện.
            posted_at: Timestamp::now(),
        };
        Ok((event, posted.replayed))
    }

    /// Số bút toán trong sổ.
    pub fn entry_count(&self) -> usize {
        self.ledger.lock().unwrap().len()
    }

    /// Số dư (đồng) của một tài khoản.
    pub fn balance(&self, account: &str) -> Result<i64, LedgerError> {
        let id = AccountId::new(account)?;
        Ok(self.ledger.lock().unwrap().balance(&id)?.dong())
    }
}

/// Processor cộng producer: ghi bút toán xong thì phát `LedgerEntryPosted`.
#[derive(Clone)]
pub struct Worker {
    pub processor: Arc<Processor>,
    producer: FutureProducer,
    out_topic: String,
}

impl Worker {
    pub fn new(processor: Arc<Processor>, producer: FutureProducer, out_topic: String) -> Worker {
        Worker {
            processor,
            producer,
            out_topic,
        }
    }

    /// Bước 1 của vòng «ghi rồi mới commit»: ghi bút toán và phát sự kiện. Commit offset do người gọi làm SAU hàm này.
    /// Nhận lại thông điệp cũng phát lại sự kiện (lần phát trước có thể chưa tới nơi); bên nhận
    /// khử trùng bằng `idempotencyKey`. Phát bằng outbox là chuyện của bài 37.
    pub async fn handle(&self, payload: Option<&[u8]>) -> Result<Outcome> {
        let outcome = self.processor.handle(payload.unwrap_or_default());
        if let Outcome::Posted(event) | Outcome::Replayed(event) = &outcome {
            let json = serde_json::to_vec(event)?;
            send(
                &self.producer,
                &self.out_topic,
                &event.idempotency_key,
                &json,
            )
            .await?;
        }
        Ok(outcome)
    }
}
