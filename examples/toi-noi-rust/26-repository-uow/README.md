# Lab 26: Repository và Unit of Work kiểu Rust

Workspace hai package. `ledger-port`: trait `LedgerRepository` và `LedgerUnitOfWork` (`-> impl Future + Send`), service `post_trip_capture` generic, adapter bộ nhớ, bộ test hợp đồng `contract`; không phụ thuộc sqlx. `ledger-pg`: adapter sqlx (`Transaction<'static, Postgres>`, rollback khi drop) và `LabDb` tạo schema `lab_<uuid>` rồi dọn.

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/26-repository-uow/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/26-repository-uow/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/26-repository-uow/Cargo.toml
cargo tree --manifest-path examples/toi-noi-rust/26-repository-uow/Cargo.toml -p ledger-port | grep -c sqlx   # 0
```

Chứng minh: cùng một bộ hợp đồng (idempotent, cân, số dư, rollback, đồng thời) chạy trên adapter giả và adapter PostgreSQL thật; `ledger-core` và `ledger-port` không có sqlx; sequence PostgreSQL có lỗ hổng mã, bộ nhớ thì không. Cần PostgreSQL của `infra/` đang chạy; thiếu `TOI_NOI_PG_URL` hoặc không kết nối được thì lab FAIL.
Sạch với `cargo clippy --all-targets -- -D warnings` và `cargo fmt --all --check`. Dependency: `sqlx`, `tokio`, `uuid`, `anyhow`, `thiserror`.
