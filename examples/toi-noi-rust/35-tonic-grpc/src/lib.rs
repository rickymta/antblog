//! Lab bài 35: gRPC với tonic cho service sổ cái (GetBalance, PostEntry).

pub mod interceptor;
pub mod lab;
pub mod lab_db;
pub mod server;
pub mod service;
pub mod status;
pub mod store;

/// Code sinh từ `proto/ledger/v1/ledger.proto` bởi `build.rs`.
pub mod pb {
    tonic::include_proto!("ledger.v1");
}
