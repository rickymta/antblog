//! Chuyển context W3C qua hai loại «hộp thư»: metadata của gRPC và header của Kafka.
//! Propagator (`TraceContextPropagator`) chỉ biết `Injector`/`Extractor`; file này là phần dán.

use opentelemetry::propagation::{Extractor, Injector};
use opentelemetry::{Context, global};
use rdkafka::message::{Header, Headers, OwnedHeaders};
use tonic::metadata::{KeyRef, MetadataKey, MetadataMap, MetadataValue};
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Ghi header propagation vào `MetadataMap` của tonic.
pub struct MetadataInjector<'a>(pub &'a mut MetadataMap);

impl Injector for MetadataInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        // Key hoặc value không hợp lệ thì bỏ qua: mất liên kết trace tốt hơn làm hỏng request.
        if let (Ok(key), Ok(value)) = (
            MetadataKey::from_bytes(key.as_bytes()),
            MetadataValue::try_from(value.as_str()),
        ) {
            self.0.insert(key, value);
        }
    }
}

/// Đọc header propagation từ `MetadataMap` của tonic.
pub struct MetadataExtractor<'a>(pub &'a MetadataMap);

impl Extractor for MetadataExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }
    fn keys(&self) -> Vec<&str> {
        self.0
            .keys()
            .map(|k| match k {
                KeyRef::Ascii(k) => k.as_str(),
                KeyRef::Binary(k) => k.as_str(),
            })
            .collect()
    }
}

/// Gom cặp key/value mà propagator ghi; `OwnedHeaders` của rdkafka bất biến nên phải gom rồi dựng.
#[derive(Default)]
pub struct KafkaInjector(Vec<(String, String)>);

impl Injector for KafkaInjector {
    fn set(&mut self, key: &str, value: String) {
        self.0.push((key.to_owned(), value));
    }
}

impl KafkaInjector {
    /// Dựng `OwnedHeaders` để gắn vào `FutureRecord::headers`.
    pub fn into_headers(self) -> OwnedHeaders {
        self.0
            .iter()
            .fold(OwnedHeaders::new(), |headers, (key, value)| {
                headers.insert(Header {
                    key,
                    value: Some(value.as_str()),
                })
            })
    }
}

/// Đọc header propagation từ header của một thông điệp Kafka.
pub struct KafkaExtractor<'a, H: Headers>(pub &'a H);

impl<H: Headers> Extractor for KafkaExtractor<'_, H> {
    fn get(&self, key: &str) -> Option<&str> {
        (0..self.0.count())
            .filter_map(|i| self.0.try_get(i))
            .find(|h| h.key.eq_ignore_ascii_case(key))
            .and_then(|h| h.value)
            .and_then(|v| std::str::from_utf8(v).ok())
    }
    fn keys(&self) -> Vec<&str> {
        (0..self.0.count())
            .filter_map(|i| self.0.try_get(i))
            .map(|h| h.key)
            .collect()
    }
}

/// Context của upstream trong metadata gRPC; không có header hợp lệ thì trả context rỗng (span mới thành gốc).
pub fn extract_from_metadata(metadata: &MetadataMap) -> Context {
    global::get_text_map_propagator(|p| p.extract(&MetadataExtractor(metadata)))
}

/// Context của producer trong header Kafka; thông điệp không có header thì trả context rỗng.
pub fn extract_from_kafka<H: Headers>(headers: Option<&H>) -> Context {
    match headers {
        Some(h) => global::get_text_map_propagator(|p| p.extract(&KafkaExtractor(h))),
        None => Context::new(),
    }
}

/// Ghi context của span `tracing` hiện tại vào metadata gRPC (phía client).
pub fn inject_current_into_metadata(metadata: &mut MetadataMap) {
    let cx = tracing::Span::current().context();
    global::get_text_map_propagator(|p| p.inject_context(&cx, &mut MetadataInjector(metadata)));
}

/// Header Kafka mang context của span `tracing` hiện tại (phía producer).
pub fn current_kafka_headers() -> OwnedHeaders {
    let cx = tracing::Span::current().context();
    let mut injector = KafkaInjector::default();
    global::get_text_map_propagator(|p| p.inject_context(&cx, &mut injector));
    injector.into_headers()
}
