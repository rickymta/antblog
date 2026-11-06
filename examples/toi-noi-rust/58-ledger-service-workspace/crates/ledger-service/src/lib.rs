//! `ledger-service`: composition root. Crate duy nhất biết cả bốn crate còn lại; nó chọn adapter
//! nào nối vào port nào, rồi chạy bốn task (consumer, relay, HTTP, gRPC) và tắt có thứ tự.

pub mod app;
pub mod arch;
pub mod config;
pub mod harness;
pub mod scenario;
pub mod telemetry;
