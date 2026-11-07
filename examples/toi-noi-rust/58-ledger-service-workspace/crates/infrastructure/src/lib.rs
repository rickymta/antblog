//! Adapter hướng ra ngoài của `ledger-service`: cài port [`ledger_application::LedgerStore`] bằng
//! PostgreSQL, đọc `TripCompleted` từ Kafka và phát `LedgerEntryPosted` qua outbox.
//!
//! Crate này phụ thuộc `application` và `domain`, không bao giờ phụ thuộc `api`.

pub mod consumer;
pub mod kafka;
pub mod pg;
pub mod relay;
pub mod stats;
pub mod wire;
