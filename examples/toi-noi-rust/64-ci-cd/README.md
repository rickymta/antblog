# Lab 64: CI/CD cho crate Rust có PostgreSQL

Crate sổ cái nhỏ (`ledger_ci`), ba workflow GitHub Actions mẫu (`workflows/`) và `deny.toml`. **Workflow chưa chạy trên GitHub**: chỉ kiểm cú pháp (actionlint, schema SchemaStore) và chạy từng lệnh của chúng ở local. Cần `TOI_NOI_PG_URL`; thiếu thì FAIL, không skip.

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab' SQLX_OFFLINE=true
cargo run  --manifest-path examples/toi-noi-rust/64-ci-cd/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/64-ci-cd/Cargo.toml
```

Trong thư mục lab (`cargo install --locked cargo-audit cargo-deny sqlx-cli`): `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo audit`, `cargo deny check`, `cargo build --release --locked`, và với database TRỐNG: `sqlx migrate run --source migrations && cargo sqlx prepare --check`.

`--verify` kiểm workflow (YAML parse được, đủ job, `contents: read`, pin SHA, `--locked`, MSRV khớp `rust-version`) và `deny.toml`, rồi ghi `TN-0001` vào PostgreSQL trong schema `lab_<uuid>`. Không gọi mạng. `.sqlx/` là dữ liệu offline của `query!`, commit cùng mã nguồn.
