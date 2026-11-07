//! Consumer của `LedgerEntryPosted`: đọc `traceparent` từ header Kafka và nối span.
//!
//! Hai cách nối, chọn theo hình dạng việc xử lý:
//! - một thông điệp một span, CON của span producer (`process_next`): một trace trọn vẹn;
//! - cả lô một span, có LINK tới từng producer (`process_batch`): một span chỉ có một cha,
//!   nên N thông điệp từ N trace không thể cùng là con.
//!
//! Semantic conventions mặc định khuyên link; cha-con chỉ là lựa chọn (MAY) cho xử lý một thông điệp
//! và phải được ghi rõ. Lab ghi rõ ở đây: `process_next` chọn cha-con để đọc được một waterfall.

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context as _, Result};
use opentelemetry::KeyValue;
use opentelemetry::trace::{Status as SpanStatus, TraceContextExt};
use rdkafka::Message;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::OwnedMessage;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::events::decode;
use crate::kafka::recv_within;
use crate::metrics::Instruments;
use crate::propagation::extract_from_kafka;
use crate::telemetry::current_trace_id;

/// Consumer cộng read model đơn giản (danh sách `entry_id` đã áp dụng).
pub struct EntryConsumer {
    consumer: StreamConsumer,
    topic: String,
    group: String,
    metrics: Arc<Instruments>,
    applied: Mutex<Vec<String>>,
}

impl EntryConsumer {
    /// Subscribe `topic` bằng consumer đã dựng (commit thủ công).
    ///
    /// # Errors
    /// Subscribe thất bại.
    pub fn new(
        consumer: StreamConsumer,
        topic: &str,
        group: &str,
        metrics: Arc<Instruments>,
    ) -> Result<EntryConsumer> {
        consumer.subscribe(&[topic])?;
        Ok(EntryConsumer {
            consumer,
            topic: topic.to_owned(),
            group: group.to_owned(),
            metrics,
            applied: Mutex::new(Vec::new()),
        })
    }

    /// Các `entry_id` đã áp dụng vào read model, theo thứ tự.
    pub fn applied(&self) -> Vec<String> {
        self.applied.lock().expect("mutex không bị poison").clone()
    }

    /// Nhận một thông điệp, xử lý trong span CON của producer, rồi commit offset.
    ///
    /// # Errors
    /// Hết giờ chờ, payload hỏng hoặc commit thất bại.
    pub async fn process_next(&self, wait: Duration) -> Result<String> {
        let msg = recv_within(&self.consumer, wait, "thông điệp LedgerEntryPosted").await?;
        let entry_id = self.process_one(&msg);
        // Commit SAU khi xử lý xong (at-least-once, xem bài 36), kể cả khi xử lý lỗi payload.
        self.consumer.commit_message(&msg, CommitMode::Sync)?;
        let (partition, offset) = (msg.partition(), msg.offset());
        self.update_lag(partition, offset)?;
        entry_id
    }

    // Xử lý một thông điệp trong span CONSUMER có cha lấy từ header.
    fn process_one(&self, msg: &impl Message) -> Result<String> {
        let parent = extract_from_kafka(msg.headers());
        let span = tracing::info_span!(
            "process ledger.entry-posted",
            otel.kind = "consumer",
            messaging.system.name = "kafka",
            messaging.operation.name = "process",
            messaging.operation.type = "process",
            messaging.destination.name = %msg.topic(),
            messaging.destination.partition.id = %msg.partition(),
            messaging.consumer.group.name = %self.group,
            kafka.offset = msg.offset(),
        );
        // Không có header (producer chưa gắn trace) thì `parent` rỗng và span này thành gốc mới.
        if let Err(e) = span.set_parent(parent) {
            tracing::warn!(error = %e, "không gắn được span cha, trace bị đứt");
        }
        let _entered = span.enter();
        let started = Instant::now();
        let result = self.apply(msg.payload());
        let mut attrs = vec![
            KeyValue::new("messaging.system.name", "kafka"),
            KeyValue::new("messaging.operation.name", "process"),
            KeyValue::new("messaging.destination.name", self.topic.clone()),
            KeyValue::new("messaging.consumer.group.name", self.group.clone()),
        ];
        if result.is_err() {
            attrs.push(KeyValue::new("error.type", "decode"));
            span.set_status(SpanStatus::error("decode"));
        }
        self.metrics
            .process_duration
            .record(started.elapsed().as_secs_f64(), &attrs);
        result
    }

    // Giải mã và áp dụng vào read model; có span con riêng.
    fn apply(&self, payload: Option<&[u8]>) -> Result<String> {
        let span = tracing::info_span!("projection.apply");
        let _entered = span.enter();
        let event = decode(payload.context("thông điệp rỗng")?)?;
        self.applied
            .lock()
            .expect("mutex không bị poison")
            .push(event.entry_id.clone());
        tracing::info!(trace_id = %current_trace_id(), entry_id = %event.entry_id, "đã áp dụng vào read model");
        Ok(event.entry_id)
    }

    /// Nhận `count` thông điệp rồi xử lý cả lô trong MỘT span gốc có link tới từng producer.
    ///
    /// # Errors
    /// Hết giờ chờ, payload hỏng hoặc commit thất bại.
    pub async fn process_batch(&self, count: usize, wait: Duration) -> Result<Vec<String>> {
        let mut batch: Vec<OwnedMessage> = Vec::with_capacity(count);
        while batch.len() < count {
            let msg = recv_within(&self.consumer, wait, "thông điệp trong lô").await?;
            batch.push(msg.detach());
        }
        let ids = self.process_owned_batch(&batch)?;
        if let Some(last) = batch.last() {
            // Commit một lần cho cả lô: offset của thông điệp cuối.
            let mut tpl = rdkafka::TopicPartitionList::new();
            tpl.add_partition_offset(
                last.topic(),
                last.partition(),
                rdkafka::Offset::Offset(last.offset() + 1),
            )?;
            self.consumer.commit(&tpl, CommitMode::Sync)?;
            self.update_lag(last.partition(), last.offset())?;
        }
        Ok(ids)
    }

    fn process_owned_batch(&self, batch: &[OwnedMessage]) -> Result<Vec<String>> {
        // `parent: None` ép span này thành gốc của một trace mới, không kế thừa span đang chạy.
        let span = tracing::info_span!(
            parent: None,
            "process ledger.entry-posted",
            otel.kind = "consumer",
            messaging.system.name = "kafka",
            messaging.operation.name = "process",
            messaging.operation.type = "process",
            messaging.destination.name = %self.topic,
            messaging.consumer.group.name = %self.group,
            messaging.batch.message_count = batch.len() as i64,
        );
        // Mỗi thông điệp một link (kèm sampled flag của chính nó). Link thêm TRƯỚC khi span chạy.
        for msg in batch {
            let linked = extract_from_kafka(msg.headers());
            let ctx = linked.span().span_context().clone();
            if ctx.is_valid() {
                span.add_link(ctx);
            }
        }
        let _entered = span.enter();
        let started = Instant::now();
        let result = batch
            .iter()
            .map(|m| self.apply(m.payload()))
            .collect::<Result<Vec<_>>>();
        // Cả lô là một thao tác process: một mẫu cho cả lô.
        let mut attrs = vec![
            KeyValue::new("messaging.system.name", "kafka"),
            KeyValue::new("messaging.operation.name", "process"),
            KeyValue::new("messaging.destination.name", self.topic.clone()),
            KeyValue::new("messaging.consumer.group.name", self.group.clone()),
        ];
        if result.is_err() {
            attrs.push(KeyValue::new("error.type", "decode"));
            span.set_status(SpanStatus::error("decode"));
        }
        self.metrics
            .process_duration
            .record(started.elapsed().as_secs_f64(), &attrs);
        result
    }

    // lag = offset cuối của partition (high watermark) trừ offset kế tiếp sẽ đọc.
    fn update_lag(&self, partition: i32, offset: i64) -> Result<()> {
        let (_low, high) = tokio::task::block_in_place(|| {
            self.consumer
                .fetch_watermarks(&self.topic, partition, Duration::from_secs(5))
        })?;
        self.metrics
            .lag
            .store((high - (offset + 1)).max(0), Ordering::Relaxed);
        Ok(())
    }
}
