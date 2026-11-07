//! Adapter hướng vào của `ledger-service`: HTTP theo hợp đồng lỗi bài 32, JWT bài 33,
//! gRPC theo `proto/ledger/v1/ledger.proto` của bài 35. Mọi handler chỉ gọi use case của
//! `ledger-application`; crate này không biết PostgreSQL hay Kafka.

pub mod auth;
pub mod dto;
pub mod error_map;
pub mod grpc;
pub mod http;
pub mod problem;
pub mod validate;
