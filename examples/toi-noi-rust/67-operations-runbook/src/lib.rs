//! Lab bài 67: vận hành sổ cái. SLI bằng SQL và Kafka, burn rate đa cửa sổ bằng hàm thuần,
//! alert lệch đối soát, diễn tập runbook trên PostgreSQL và Kafka thật, replay an toàn, `pg_dump` thật.

pub mod alert;
pub mod backup;
pub mod chain;
pub mod clock;
pub mod consumer;
pub mod drills;
pub mod events;
pub mod kafka;
pub mod lab;
pub mod recon;
pub mod relay;
pub mod replay;
pub mod sli;
pub mod slo;
pub mod store;
