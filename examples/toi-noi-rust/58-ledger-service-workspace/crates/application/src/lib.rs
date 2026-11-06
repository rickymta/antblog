//! Tầng use case của `ledger-service`.
//!
//! Chứa: lỗi của ứng dụng, các kiểu kết quả, port [`ports::LedgerStore`] và use case
//! [`service::Ledger`]. Adapter (PostgreSQL, Kafka, HTTP, gRPC) nằm ở crate khác và chỉ
//! phụ thuộc vào đây, không bao giờ ngược lại.

pub mod error;
pub mod model;
pub mod ports;
pub mod readiness;
pub mod service;

pub use error::Error;
pub use ledger_domain as domain;
pub use ports::LedgerStore;
pub use service::Ledger;
