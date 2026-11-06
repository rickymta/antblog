//! `location-store`: kho vị trí **mới nhất** của tài xế (LOC-18).
//!
//! Luồng dữ liệu: topic `location.updates.v1` (key `driver_id`, giá trị `DriverLocationUpdated`) →
//! [`kafka::KafkaConsumer`] → [`Update`] → [`LatestStore`] (bộ nhớ, theo ô H3, nhiều luồng đọc) →
//! tùy chọn [`mirror::MirrorWriter`] (Redis GEO là bản phụ) và [`history::Sampler`] (giảm mẫu sang lịch sử).
//!
//! API các phần sau dựa vào:
//! - [`LatestStore`]: `apply`, `get`, `nearest`, `sweep`, `snapshot`, `stats`
//! - [`Update`] / [`Position`]: một bản ghi vị trí, `accuracy/speed/heading` là `Option` (vắng khác 0)
//! - [`pb`]: các message protobuf sinh từ `proto/location/v1/location.proto` (không tự định nghĩa lại)
//!
//! Mã protobuf sinh bằng `protox` + `prost-build::compile_fds` (không cần `protoc`); xem `build.rs`.

pub mod history;
pub mod ingest;
pub mod kafka;
pub mod mirror;
mod model;
mod store;

pub mod pb {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/toinoi.location.v1.rs"));
}

pub use model::{Position, Update, now_ms};
pub use store::{Applied, LatestStore, Nearby, StoreConfig, StoreStats};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("Kafka: {0}")]
    Kafka(#[from] rdkafka::error::KafkaError),
    #[error("Redis: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("protobuf: {0}")]
    Decode(#[from] prost::DecodeError),
    #[error("{0}")]
    Other(String),
}
