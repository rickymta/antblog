# Lab 23: sqlx với PostgreSQL
Pool (`PgPoolOptions`) đọc `TOI_NOI_PG_URL`; migration `migrations/` chạy bằng `sqlx::migrate!`; ghi và đọc bút toán `TN-0001` (25.500 / 5.100 / 20.400) bằng `query!`/`query_as!`. Mỗi lần chạy dùng schema `lab_<uuid>`, xoá cuối ca, không đụng `public`.
```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab' SQLX_OFFLINE=true
cargo run --manifest-path examples/toi-noi-rust/23-sqlx-postgres/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/23-sqlx-postgres/Cargo.toml
```
Làm mới `.sqlx` khi sửa câu `query!` (cần `cargo install sqlx-cli --no-default-features --features postgres,rustls`); macro cần bảng có sẵn lúc biên dịch, trong schema cố định `lab_23_prepare`:
```bash
docker exec toi-noi-rust-pg psql -U toi_noi -d toi_noi_rust_lab -c "create schema lab_23_prepare"
export DATABASE_URL="$TOI_NOI_PG_URL?options=-c%20search_path%3Dlab_23_prepare"; cd examples/toi-noi-rust/23-sqlx-postgres
sqlx migrate run --source migrations && cargo sqlx prepare   # ghi .sqlx/, commit
docker exec toi-noi-rust-pg psql -U toi_noi -d toi_noi_rust_lab -c "drop schema lab_23_prepare cascade"
env -u DATABASE_URL SQLX_OFFLINE=true cargo build            # xanh dù không có DB
```
