//! Lab bài 25: schema sổ cái trên PostgreSQL.
//!
//! Mỗi ca chạy dựng schema riêng `lab_<uuid>`, chạy các file `sql/*.sql` vào đó rồi xoá.

pub mod bench;
pub mod db;
pub mod ledger;
pub mod partition;
