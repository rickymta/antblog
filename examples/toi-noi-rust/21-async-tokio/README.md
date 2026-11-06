# Lab 21: async/await và tokio khi gọi VíXanh giả

VíXanh giả (`src/vixanh.rs`) là hàm async dùng `tokio::time::sleep`, có lúc chậm, có lúc lỗi. `src/flows.rs` gọi nó bằng tuần tự, `join!`/`try_join!`, `timeout`, `select!`, `spawn` + `JoinHandle`, `CancellationToken`; `src/blocking.rs` đo việc chặn runtime. Dùng lại `ledger-core` của lab 20 để ghi bút toán `JE-000001`.

```bash
cargo run --manifest-path examples/toi-noi-rust/21-async-tokio/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/21-async-tokio/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/21-async-tokio/Cargo.toml
```

Chứng minh: future lười; `join!` = độ trễ lớn nhất, tuần tự = tổng (đồng hồ ảo, tất định); hết hạn không có nghĩa là VíXanh chưa trừ tiền, nên thử lại phải dùng một idempotency key; hủy future chỉ chạy `Drop`; `std::thread::sleep` làm tick chậm cỡ 300 ms, `spawn_blocking` thì không (đo bằng đồng hồ thật, số đo trên máy viết bài, không phải SLA).

Không cần PostgreSQL hay Redis. Dependency: `tokio`, `tokio-util`, `thiserror`, `ledger-core` (path). `test-util` của tokio nằm ở `[dependencies]` vì `--verify` cũng dùng đồng hồ ảo; dự án thật để ở `[dev-dependencies]`. Dữ liệu hư cấu của Tới Nơi.
