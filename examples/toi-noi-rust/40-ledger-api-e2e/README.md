# Lab 40: ledger-api end-to-end (PostgreSQL + Kafka)

Service `ledger-api` trong một crate: consumer `TripCompleted` → `ledger-core` tính phí (25.500 → 5.100 và 20.400) → bút toán + inbox + outbox cùng transaction → relay phát `LedgerEntryPosted` → HTTP `GET /v1/accounts/{id}/balance` và gRPC `GetBalance`. Có `/healthz`, `/readyz`, graceful shutdown, log JSON có `request_id`.

Cần hạ tầng `infra/` và hai biến `TOI_NOI_PG_URL`, `TOI_NOI_KAFKA` (xem `../infra/README.md`); lab không dùng Redis. Thiếu thì lab FAIL rõ.

```bash
cargo run --manifest-path examples/toi-noi-rust/40-ledger-api-e2e/Cargo.toml              # kể lại từng kịch bản
cargo run --manifest-path examples/toi-noi-rust/40-ledger-api-e2e/Cargo.toml -- --verify  # kiểm, in VERIFY OK
cargo test --manifest-path examples/toi-noi-rust/40-ledger-api-e2e/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/40-ledger-api-e2e/Cargo.toml -- serve     # chạy service thật (cần biến LEDGER_*)
```

Chứng minh: luồng đủ vòng qua cả HTTP và gRPC; phát lại sự kiện không tạo bút toán thứ hai; chết sau commit DB, trước commit offset rồi chạy lại vẫn đúng một bút toán; SIGTERM thoát mã 0. Mỗi ca dùng schema `lab_<uuid>`, topic `lab.40.<uuid>.*` và group riêng, dọn cuối ca. `build.rs` dùng `protoc-bin-vendored` nên máy không cần `protoc`.
