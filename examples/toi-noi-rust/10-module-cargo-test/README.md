# Lab 10: module, crate, cargo test và clippy

Thư viện `ledger_lab` (src/lib.rs + 5 module) và binary `ledger-cli` (src/main.rs) gom mô hình sổ cái của bài 03–09: `Money(i64)`, `AccountId`, `Posting`, `JournalEntry::try_new`, `LedgerError`, `Ledger::post` idempotent, `balance`.

```bash
cargo run --manifest-path examples/toi-noi-rust/10-module-cargo-test/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/10-module-cargo-test/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/10-module-cargo-test/Cargo.toml
```

Chứng minh: cây module trỏ tới file, `pub`/`pub(crate)`, binary dùng library qua tên crate; 16 unit test (trong `#[cfg(test)] mod tests`), 2 doc test, 4 integration test ở `tests/ledger.rs`; sạch với `cargo clippy --all-targets -- -D warnings` và `cargo fmt --check`.

Chỉ dùng thư viện chuẩn (không có dependency). Dữ liệu hư cấu của Tới Nơi.
