//! `geo-engine`: dịch vụ định tuyến gRPC bọc `route-engine` (LOC-29).
//!
//! - [`pb`]: mã sinh từ `proto/location/v1/routing.proto` (protox + tonic-prost-build, không cần `protoc`)
//! - [`service::RoutingSvc`]: `Route` và `Matrix`, deadline từ header `grpc-timeout`, cache theo ô H3 và khung giờ,
//!   giới hạn số tính toán đồng thời, fallback chim bay × hệ số vòng
//! - [`cache::RouteCache`]: cache có giới hạn, chia shard
//! - binary `geo-engine` chạy máy chủ; lab `loc29_dich_vu_dinh_tuyen` đo p50/p99 dưới tải

pub mod cache;
pub mod service;

#[allow(clippy::all, clippy::pedantic)]
pub mod pb {
    tonic::include_proto!("toinoi.location.v1");
}

pub use service::{GeoEngineConfig, Metrics, MetricsSnapshot, RoutingSvc};
