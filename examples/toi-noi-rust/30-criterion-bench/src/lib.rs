//! Lab bài 30: đo thay vì đoán. Workload sổ cái dùng chung cho benchmark, dhat và PostgreSQL.

pub mod index;
pub mod pg;
pub mod timing;
pub mod workload;

pub use index::IndexedLedger;
pub use workload::{make_entry, make_ledger};
