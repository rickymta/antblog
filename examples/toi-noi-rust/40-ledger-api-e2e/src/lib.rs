//! `ledger-api`: service sổ cái chạy thật với PostgreSQL và Kafka (lab bài 40).
//!
//! `TripCompleted` → `ledger-core` tính phí → bút toán + inbox + outbox cùng transaction →
//! relay phát `LedgerEntryPosted`; HTTP và gRPC trả số dư.

pub mod app;
pub mod config;
pub mod consumer;
pub mod events;
pub mod grpc;
pub mod harness;
pub mod http;
pub mod kafka;
pub mod relay;
pub mod scenario;
pub mod state;
pub mod store;
pub mod telemetry;
