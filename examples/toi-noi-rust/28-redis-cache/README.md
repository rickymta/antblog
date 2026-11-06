# Lab 28: Redis cache số dư, rate limit và lock có TTL

PostgreSQL (schema riêng `lab_<uuid>`) là sổ cái; Redis (key `lab:28:<uuid>:...` có TTL) chỉ là cache và công cụ điều phối. Cần Docker Compose ở `examples/toi-noi-rust/infra` đã bật.

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
export TOI_NOI_REDIS='redis://127.0.0.1:6379'
cargo run --manifest-path examples/toi-noi-rust/28-redis-cache/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/28-redis-cache/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/28-redis-cache/Cargo.toml
```

Chứng minh: cache-aside có TTL và xoá key sau commit; hai cửa sổ lệch (trước DEL, và nạp số cũ sau DEL) cùng cách xử lý «lệch thì tin sổ cái»; token bucket Lua cho qua 10/100 yêu cầu đồng thời còn GET/SET từ client cho qua 100 (barrier ép thứ tự xấu nhất); lock `SET NX PX` hết hạn khi worker cũ còn chạy nên có hai worker trong vùng găng, còn `UNIQUE (idempotency_key)` của PostgreSQL vẫn giữ sổ cái đúng; pipeline so với gọi lần lượt (số đo chỉ in, không kiểm). Thiếu biến môi trường hoặc không kết nối được thì lab FAIL, không skip. Dọn: SCAN theo tiền tố rồi `DEL` (không `FLUSHALL`), `DROP SCHEMA ... CASCADE`.

Phụ thuộc chính: `redis` 1.8 (`tokio-comp`, `connection-manager`), `sqlx` 0.9 (`runtime-tokio`, `postgres`), `tokio`, `uuid`, `anyhow`, `ledger-core` (lab 20).
