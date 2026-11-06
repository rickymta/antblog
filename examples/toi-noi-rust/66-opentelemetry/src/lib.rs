//! Lab bài 66: một `trace_id` đi từ client (C#) qua gRPC (tonic) và Kafka (rdkafka) tới consumer,
//! bằng `tracing-opentelemetry` và W3C Trace Context.

pub mod checks;
pub mod collector;
pub mod consumer;
pub mod events;
pub mod kafka;
pub mod metrics;
pub mod propagation;
pub mod service;
pub mod stack;
pub mod telemetry;
pub mod upstream;

/// Code sinh từ `proto/ledger/v1/ledger.proto` (cùng file với bài 35) bởi `build.rs`.
pub mod pb {
    tonic::include_proto!("ledger.v1");
}
