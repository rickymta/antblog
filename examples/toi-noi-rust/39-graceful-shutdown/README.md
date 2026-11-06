# Lab bài 39: graceful shutdown, health/readiness, backpressure

Service sổ cái nhận `TripCompleted` từ Kafka, ghi bút toán vào PostgreSQL, phát `LedgerEntryPosted` qua outbox. Lab chứng minh bằng chạy thật: SIGTERM và ctrl-c, thứ tự tắt, drain có hạn, SIGKILL không mất bút toán, `/healthz` khác `/readyz`, hàng đợi bounded và `pause`/`resume`.

Cần hạ tầng (`../infra/README.md`) và ba biến `TOI_NOI_PG_URL`, `TOI_NOI_KAFKA`, `TOI_NOI_REDIS`; thiếu thì lab báo lỗi, không bỏ qua.

```bash
cargo run --manifest-path examples/toi-noi-rust/39-graceful-shutdown/Cargo.toml                 # mọi kịch bản
cargo run --manifest-path examples/toi-noi-rust/39-graceful-shutdown/Cargo.toml -- --verify     # kiểm kỳ vọng, in VERIFY OK
cargo run --manifest-path examples/toi-noi-rust/39-graceful-shutdown/Cargo.toml -- graceful     # một kịch bản: probes graceful http timeout backpressure poll kill
cargo run --manifest-path examples/toi-noi-rust/39-graceful-shutdown/Cargo.toml -- serve        # service thật, tắt bằng SIGTERM hoặc ctrl-c
cargo run --manifest-path examples/toi-noi-rust/39-graceful-shutdown/Cargo.toml -- resume       # chỉ đo độ trễ sau resume, không nằm trong --verify
cargo test --manifest-path examples/toi-noi-rust/39-graceful-shutdown/Cargo.toml
```

Mỗi ca có schema `lab_<uuid>`, topic `lab.39.<uuid>.*`, group `lab-39-<uuid>`, dọn cuối ca. Test `tests/signals.rs` spawn binary `serve` rồi gửi tín hiệu thật bằng `kill`. Mã thoát của `serve`: 0 sạch, 3 drain quá hạn, 4 nhận tín hiệu thứ hai.
