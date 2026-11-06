# Lab 20: workspace `ledger-core` + `ledger-cli`

Bài tổng hợp cấp cơ bản. Cargo workspace gồm thư viện `ledger-core` (money, account, entry, policy, state, error, ledger; serde cho bút toán) và binary `ledger-cli` chạy kịch bản chuyến `TN-0001`.

```bash
cargo run --manifest-path examples/toi-noi-rust/20-ledger-core-workspace/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/20-ledger-core-workspace/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/20-ledger-core-workspace/Cargo.toml
cargo doc --no-deps --manifest-path examples/toi-noi-rust/20-ledger-core-workspace/Cargo.toml
```

Chứng minh: workspace với `default-members` chạy được từ gốc repo; `#![deny(missing_docs)]`; clippy pedantic có chọn lọc; unit, integration, property (proptest) và doc test; `--verify` in `VERIFY OK`.
Sạch với `cargo clippy --all-targets -- -D warnings` và `cargo fmt --all --check`.

Dependency: `rust_decimal`, `serde`, `serde_json`, `thiserror`, `proptest` (dev). Dữ liệu hư cấu của Tới Nơi.
