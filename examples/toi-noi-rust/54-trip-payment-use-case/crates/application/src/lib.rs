//! Use case thanh toán chuyến. Biết domain và biết *hình dạng* của kho lưu trữ (trait `PaymentStore`),
//! không biết PostgreSQL hay Kafka: adapter nằm ở crate `payment-infra` (bài 51).

pub mod command;
pub mod handler;
pub mod port;

pub use command::{Command, validate};
pub use handler::{AppError, MAX_ATTEMPTS, Outcome, handle};
pub use port::{Committed, Loaded, PaymentStore, PostedEntry, StoreError, Work};
