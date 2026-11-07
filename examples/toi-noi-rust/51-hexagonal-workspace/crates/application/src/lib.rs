//! `application`: use case và các port. Crate này không biết sqlx, Kafka hay axum, cũng không có tokio.
//!
//! Port là trait do application sở hữu; adapter ở crate `infrastructure` hiện thực chúng.
//! Trait trả `impl Future + Send` thay vì `async fn` trần để use case generic vẫn
//! `tokio::spawn` được ở crate `api`, mà application không cần phụ thuộc tokio.

mod ports;
mod settle;

pub use ports::{Clock, EventPublisher, LedgerStore, PostOutcome, PublishError, StoreError};
pub use settle::{GetBalance, SettleError, SettleReceipt, SettleTrip, StepReceipt};
