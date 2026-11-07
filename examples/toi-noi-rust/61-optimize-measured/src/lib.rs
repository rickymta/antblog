//! Lab bài 61: tối ưu có số đo. Ghi bút toán vào PostgreSQL bằng nhiều cách, đo bằng chỉ số
//! không phụ thuộc tải (số cấp phát, số message ở tầng dây) trước, thời gian sau.

pub mod account;
pub mod alloc_count;
pub mod columns;
pub mod db;
pub mod experiments;
pub mod hist;
pub mod loadgen;
pub mod omission;
pub mod prepared;
pub mod proxy;
pub mod strategies;
pub mod workload;

// Allocator đếm áp dụng cho mọi binary và test dùng crate này (chỉ lab, không phải thư viện thật).
#[global_allocator]
static ALLOC: alloc_count::CountingAlloc = alloc_count::CountingAlloc;
