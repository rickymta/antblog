//! `matcher` (P05): ghép chuyến và giảm km rỗng. Rust **đề xuất**, Dispatch (C#) **quyết định**.
//!
//! Mọi con số của crate này là **trên dữ liệu mô phỏng** (simulator), không dự báo thị trường thật.

pub mod assign;
pub mod batch;
pub mod cost;
pub mod deviation; // LOC-48
pub mod eval;
pub mod multistop; // LOC-50
pub mod policy;
pub mod prune;
pub mod reposition; // LOC-49
pub mod suggest; // LOC-46
