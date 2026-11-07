# Lab 58: ledger-service hoàn chỉnh trong Cargo workspace

Năm crate theo kiến trúc bài 51: `domain` → `application` ← `infrastructure` (sqlx, rdkafka), `api` (axum, tonic), và binary `ledger-service` là composition root duy nhất biết cả bốn. `TripCompleted` → hai bút toán (capture, split) → outbox → `LedgerEntryPosted` → số dư qua HTTP và gRPC; có `POST /v1/entries`, JWT, Problem Details, hold, hoàn tiền một phần, graceful shutdown.

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
export TOI_NOI_KAFKA='127.0.0.1:9092'
cargo run  --manifest-path examples/toi-noi-rust/58-ledger-service-workspace/Cargo.toml              # kể lại từng kịch bản
cargo run  --manifest-path examples/toi-noi-rust/58-ledger-service-workspace/Cargo.toml -- --verify  # kiểm, in VERIFY OK
cargo test --manifest-path examples/toi-noi-rust/58-ledger-service-workspace/Cargo.toml
cargo run  --manifest-path examples/toi-noi-rust/58-ledger-service-workspace/Cargo.toml -- serve     # service thật, cần LEDGER_JWT_SECRET
```

Chứng minh: `cargo tree` kiểm chiều phụ thuộc từng crate (và bộ kiểm không mù); luồng đủ vòng Kafka → PostgreSQL → Kafka; JWT 401/403, Problem Details, idempotency chung HTTP và gRPC; hold hết hạn bằng `ManualClock` (không sleep; `clippy.toml` cấm `Timestamp::now` ngoài `SystemClock`); trigger cân bút toán và append-only chặn cả khi bỏ qua application; số dư khóa theo thứ tự mã tài khoản; SIGTERM thoát mã 0.
Dùng lại `ledger-core` của lab 20 (không sửa); `01_schema.sql`, `02_balance_check.sql` là bản của lab 25, giữ nguyên từng byte; `ledger.proto` là bản của lab 35. Mỗi ca dùng schema `lab_<uuid>`, topic `lab.58.<uuid>.*` và group riêng, dọn cuối ca. Thiếu biến thì FAIL rõ.
