//! Lab bài 48: số dư theo thời điểm, snapshot hằng ngày, đóng kỳ kế toán và cuộc đua đóng kỳ.
//!
//! Mọi thí nghiệm chạy trên PostgreSQL thật, trong schema `lab_<uuid>` riêng và bị xoá cuối ca.
//! Thời gian nghiệp vụ (posted_at, mốc đóng kỳ) luôn đi qua tham số, không đọc đồng hồ máy.

pub mod db;
pub mod period;
pub mod post;
pub mod race;
pub mod seed;
pub mod snapshot;
pub mod tz;
