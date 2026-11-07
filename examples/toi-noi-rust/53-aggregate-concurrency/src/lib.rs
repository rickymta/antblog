//! Lab bài 53: aggregate `Account`, optimistic locking bằng `version`, `SELECT ... FOR UPDATE`,
//! thứ tự khóa và deadlock trên PostgreSQL thật.

pub mod account;
pub mod lab;
pub mod retry;
pub mod scenarios;
pub mod store;
