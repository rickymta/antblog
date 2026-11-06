//! Lab bài 45: khuyến mãi, trợ giá và ngân sách campaign là tài khoản có giới hạn,
//! chạy trên PostgreSQL thật. Dùng `ledger-core` của bài 20 cho bất biến "bút toán cân".

pub mod db;
pub mod promo;
pub mod race;
pub mod scenarios;
pub mod store;

pub use promo::{Funding, PromoGrant, Shares};
pub use store::{BudgetGuard, BudgetSpend, PgLedger, PostError, Posted};
