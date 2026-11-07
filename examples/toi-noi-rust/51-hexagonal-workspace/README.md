# Lab 51: Hexagonal/Clean bằng workspace Cargo

Bốn crate: `domain` (luật chuyến đi, dùng `ledger-core` của bài 20), `application` (use case và port), `infrastructure` (adapter bộ nhớ, sqlx, Kafka) và `api` (HTTP axum, composition root). Chiều phụ thuộc: `api` → `infrastructure` → `application` → `domain`.

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
export TOI_NOI_KAFKA='127.0.0.1:9092'
cargo run  --manifest-path examples/toi-noi-rust/51-hexagonal-workspace/Cargo.toml                 # chạy thử bộ nhớ, không cần hạ tầng
cargo run  --manifest-path examples/toi-noi-rust/51-hexagonal-workspace/Cargo.toml -- --verify     # bộ nhớ, PostgreSQL, Kafka thật
cargo test --manifest-path examples/toi-noi-rust/51-hexagonal-workspace/Cargo.toml
cargo build --no-default-features --manifest-path examples/toi-noi-rust/51-hexagonal-workspace/Cargo.toml   # không có rdkafka
```

Lab chứng minh: Cargo không cho `domain` dùng crate chưa khai báo và từ chối vòng phụ thuộc; test `crates/api/tests/architecture.rs` đọc đồ thị của `cargo metadata` để chặn `domain` kéo `tokio`/`sqlx`; một bộ test hợp đồng chạy cho `MemoryStore` và `PgStore`; feature `kafka` quyết định `rdkafka` có trong build hay không.
