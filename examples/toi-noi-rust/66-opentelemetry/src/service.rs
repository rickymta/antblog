//! gRPC server `ledger.v1.Ledger/PostEntry` có trace: trích `traceparent` từ metadata, ghi bút toán,
//! phát `LedgerEntryPosted` lên Kafka kèm `traceparent` mới.
//!
//! Sổ cái là `Ledger` trong bộ nhớ của `ledger-core` (như lab 36): lab này nói về trace, không về lưu trữ.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::Result;
use jiff::Timestamp;
use ledger_core::{AccountId, JournalEntry, Ledger, LedgerError, Money, Posting};
use opentelemetry::KeyValue;
use opentelemetry::trace::Status as SpanStatus;
use rdkafka::producer::{FutureProducer, FutureRecord};
use tonic::{Code, Request, Response, Status};
use tracing::{Instrument, Span, field};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::events::{LedgerEntryPosted, PostingDto, SCHEMA_VERSION};
use crate::metrics::Instruments;
use crate::pb::ledger_server::Ledger as LedgerRpc;
use crate::pb::{GetBalanceRequest, GetBalanceResponse, PostEntryRequest, PostEntryResponse};
use crate::propagation::{current_kafka_headers, extract_from_metadata};
use crate::telemetry::current_trace_id;

/// Tên RPC theo semantic conventions: `<package>.<Service>/<Method>`.
pub const POST_ENTRY: &str = "ledger.v1.Ledger/PostEntry";

/// Phát `LedgerEntryPosted` lên một topic, mỗi lần gửi là một span PRODUCER.
pub struct EntryPublisher {
    producer: FutureProducer,
    topic: String,
}

impl EntryPublisher {
    pub fn new(producer: FutureProducer, topic: String) -> EntryPublisher {
        EntryPublisher { producer, topic }
    }

    /// Gửi và chờ broker xác nhận; trả `(partition, offset)`.
    ///
    /// # Errors
    /// Không tuần tự hoá được hoặc broker không xác nhận.
    pub async fn publish(&self, event: &LedgerEntryPosted) -> Result<(i32, i64)> {
        // Tên span dùng tên topic logic; tên topic thật (có uuid của lab) nằm trong attribute.
        let span = tracing::info_span!(
            "send ledger.entry-posted",
            otel.kind = "producer",
            messaging.system.name = "kafka",
            messaging.operation.name = "send",
            messaging.operation.type = "send",
            messaging.destination.name = %self.topic,
            messaging.destination.partition.id = field::Empty,
            kafka.offset = field::Empty,
        );
        async {
            let payload = serde_json::to_vec(event)?;
            // Span producer đang là span hiện tại: context của NÓ đi vào header,
            // nên span consumer sẽ là con của span này, không phải của span gRPC.
            let headers = current_kafka_headers();
            let record = FutureRecord::to(&self.topic)
                .key(&event.idempotency_key)
                .payload(&payload)
                .headers(headers);
            let (partition, offset) = self
                .producer
                .send(record, Duration::from_secs(10))
                .await
                .map(|d| (d.partition, d.offset))
                .map_err(|(e, _)| anyhow::anyhow!("gửi Kafka: {e}"))?;
            let span = Span::current();
            span.record("messaging.destination.partition.id", partition.to_string());
            span.record("kafka.offset", offset);
            Ok((partition, offset))
        }
        .instrument(span)
        .await
    }
}

/// Server gRPC.
pub struct LedgerService {
    ledger: Mutex<Ledger>,
    publisher: EntryPublisher,
    metrics: Arc<Instruments>,
}

impl LedgerService {
    pub fn new(publisher: EntryPublisher, metrics: Arc<Instruments>) -> LedgerService {
        LedgerService {
            ledger: Mutex::new(Ledger::default()),
            publisher,
            metrics,
        }
    }

    // Ghi vào sổ trong một span riêng; đo histogram và đếm lỗi ở đây, đúng chỗ biết kết quả.
    fn post_to_ledger(
        &self,
        req: &PostEntryRequest,
    ) -> Result<(String, bool, LedgerEntryPosted), LedgerError> {
        let span = tracing::info_span!(
            "ledger.entry.post",
            ledger.postings.count = req.postings.len() as i64,
            ledger.outcome = field::Empty,
            error.type = field::Empty,
        );
        let _entered = span.enter();
        let started = Instant::now();
        let result = self.try_post(req);
        let outcome = match &result {
            Ok((_, true, _)) => "replayed",
            Ok((_, false, _)) => "posted",
            Err(_) => "rejected",
        };
        span.record("ledger.outcome", outcome);
        // Histogram: attribute chỉ có `ledger.outcome` (3 giá trị). Không có account, không có trip.
        self.metrics.post_duration.record(
            started.elapsed().as_secs_f64(),
            &[KeyValue::new("ledger.outcome", outcome)],
        );
        if let Err(e) = &result {
            let kind = error_type(e);
            span.record("error.type", kind);
            // Mô tả trạng thái là mã cố định: thông điệp lỗi gốc có thể chứa số tiền.
            span.set_status(SpanStatus::error(kind));
            self.metrics
                .post_errors
                .add(1, &[KeyValue::new("error.type", kind)]);
        }
        result
    }

    fn try_post(
        &self,
        req: &PostEntryRequest,
    ) -> Result<(String, bool, LedgerEntryPosted), LedgerError> {
        let postings = req
            .postings
            .iter()
            .map(|p| {
                Ok(Posting::new(
                    AccountId::new(&p.account)?,
                    Money::from_dong(p.amount_dong),
                ))
            })
            .collect::<Result<Vec<_>, LedgerError>>()?;
        let entry = JournalEntry::try_new(&req.idempotency_key, postings)?;
        let mut ledger = self.ledger.lock().expect("mutex không bị poison");
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
            posted_at: Timestamp::now(),
        };
        Ok((posted.id.to_string(), posted.replayed, event))
    }

    async fn handle_post(&self, req: PostEntryRequest) -> Result<PostEntryResponse, Status> {
        let (entry_id, replayed, event) = self
            .post_to_ledger(&req)
            .map_err(|e| ledger_error_status(&e))?;
        // Key đã có thì sổ không ghi thêm, nên cũng không phát thêm sự kiện.
        if !replayed {
            self.publisher.publish(&event).await.map_err(|e| {
                tracing::error!(trace_id = %current_trace_id(), error = %e, "phát sự kiện thất bại");
                Status::unavailable("không phát được sự kiện")
            })?;
        }
        // Log có `trace_id`: từ dòng log này tìm được trace, và ngược lại.
        tracing::info!(trace_id = %current_trace_id(), entry_id = %entry_id, replayed, "bút toán đã ghi");
        Ok(PostEntryResponse { entry_id, replayed })
    }
}

#[tonic::async_trait]
impl LedgerRpc for LedgerService {
    async fn get_balance(
        &self,
        _request: Request<GetBalanceRequest>,
    ) -> Result<Response<GetBalanceResponse>, Status> {
        Err(Status::unimplemented("lab 66 chỉ có PostEntry"))
    }

    async fn post_entry(
        &self,
        request: Request<PostEntryRequest>,
    ) -> Result<Response<PostEntryResponse>, Status> {
        // 1. Trích context của upstream (C#) từ metadata, TRƯỚC khi span được bắt đầu.
        let parent = extract_from_metadata(request.metadata());
        let span = tracing::info_span!(
            "ledger.v1.Ledger/PostEntry",
            otel.kind = "server",
            rpc.system.name = "grpc",
            rpc.method = POST_ENTRY,
            rpc.status_code = field::Empty,
            error.type = field::Empty,
        );
        // 2. Gắn cha. `set_parent` lỗi (AlreadyStarted) nếu span đã chạy: phải gọi trước `enter`.
        if let Err(e) = span.set_parent(parent) {
            tracing::warn!(error = %e, "không gắn được span cha, trace bị đứt");
        }
        // 3. Chạy handler trong span; code bên trong tạo span con tự động theo ngữ cảnh.
        let result = self
            .handle_post(request.into_inner())
            .instrument(span.clone())
            .await;
        let code = result.as_ref().err().map_or(Code::Ok, Status::code);
        span.record("rpc.status_code", code_name(code));
        // Phía server, semantic conventions gRPC chỉ coi một số mã là lỗi; INVALID_ARGUMENT thì không.
        if is_server_error(code) {
            span.record("error.type", code_name(code));
            span.set_status(SpanStatus::error(code_name(code)));
        }
        result.map(Response::new)
    }
}

/// `error.type` có giới hạn: một biến thể một giá trị, không có nhánh `_`.
pub fn error_type(e: &LedgerError) -> &'static str {
    match e {
        LedgerError::EmptyAccountId => "empty_account",
        LedgerError::EmptyIdempotencyKey => "empty_idempotency_key",
        LedgerError::TooFewPostings { .. } => "too_few_postings",
        LedgerError::Unbalanced { .. } => "unbalanced",
        LedgerError::Overflow => "overflow",
        LedgerError::InvalidFee { .. } => "invalid_fee",
        LedgerError::IdempotencyConflict { .. } => "idempotency_conflict",
        LedgerError::UnknownEntry { .. } => "unknown_entry",
        LedgerError::InvalidEntryId { .. } => "invalid_entry_id",
    }
}

fn ledger_error_status(e: &LedgerError) -> Status {
    match e {
        LedgerError::IdempotencyConflict { .. } => Status::already_exists(e.to_string()),
        LedgerError::InvalidFee { .. } | LedgerError::UnknownEntry { .. } => {
            Status::internal("lỗi nội bộ")
        }
        LedgerError::EmptyAccountId
        | LedgerError::EmptyIdempotencyKey
        | LedgerError::TooFewPostings { .. }
        | LedgerError::Unbalanced { .. }
        | LedgerError::Overflow
        | LedgerError::InvalidEntryId { .. } => Status::invalid_argument(e.to_string()),
    }
}

/// Mã gRPC mà semantic conventions coi là lỗi ở span SERVER.
pub fn is_server_error(code: Code) -> bool {
    matches!(
        code,
        Code::Unknown
            | Code::DeadlineExceeded
            | Code::Unimplemented
            | Code::Internal
            | Code::Unavailable
            | Code::DataLoss
    )
}

/// Tên mã gRPC viết hoa theo đặc tả, để làm `rpc.status_code`.
pub fn code_name(code: Code) -> &'static str {
    match code {
        Code::Ok => "OK",
        Code::Cancelled => "CANCELLED",
        Code::Unknown => "UNKNOWN",
        Code::InvalidArgument => "INVALID_ARGUMENT",
        Code::DeadlineExceeded => "DEADLINE_EXCEEDED",
        Code::NotFound => "NOT_FOUND",
        Code::AlreadyExists => "ALREADY_EXISTS",
        Code::PermissionDenied => "PERMISSION_DENIED",
        Code::ResourceExhausted => "RESOURCE_EXHAUSTED",
        Code::FailedPrecondition => "FAILED_PRECONDITION",
        Code::Aborted => "ABORTED",
        Code::OutOfRange => "OUT_OF_RANGE",
        Code::Unimplemented => "UNIMPLEMENTED",
        Code::Internal => "INTERNAL",
        Code::Unavailable => "UNAVAILABLE",
        Code::DataLoss => "DATA_LOSS",
        Code::Unauthenticated => "UNAUTHENTICATED",
    }
}
