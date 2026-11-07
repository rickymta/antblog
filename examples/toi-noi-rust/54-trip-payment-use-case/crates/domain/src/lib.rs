//! Domain của use case thanh toán chuyến. Thuần: không async, không I/O, không biết PostgreSQL hay Kafka.
//!
//! Bốn phần: sự kiện trên dây (`events`), bút toán của từng bước (`flows`),
//! aggregate `TripPayment` gom các sự kiện đến lệch thứ tự (`payment`) và đối soát thuần (`reconcile`).

pub mod chart;
pub mod events;
pub mod flows;
pub mod payment;
pub mod reconcile;

// Kiểu lõi của `ledger-core` đi cùng domain để các crate trên không phải khai báo thêm `ledger-core`.
pub use events::{LedgerEntryPosted, PaymentCaptured, PostingDto, SCHEMA_VERSION, TripCompleted};
pub use ledger_core::{AccountId, JournalEntryBuilder, Money};
pub use payment::{
    CaptureFacts, DomainError, DomainEvent, Funds, PlannedEntry, PromoGrant, TripFacts,
    TripPayment, Waiting,
};
