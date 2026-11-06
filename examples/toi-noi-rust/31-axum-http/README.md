# Lab 31: HTTP với axum, endpoint ghi bút toán và tra số dư

`POST /v1/entries` (201 tạo mới, 200 gửi lại, 422 lệch, 409 xung đột key, 400 JSON hỏng, 422 sai hình dạng, 415 sai `Content-Type`, 413 quá 2 MB) và `GET /v1/accounts/{id}/balance` (200, 404). `Router` + handler async, extractor `State`/`Json`/`Path`, `AppError: IntoResponse`, một `tower::Service` viết tay (`/healthz`), PostgreSQL thật qua sqlx.

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/31-axum-http/Cargo.toml            # chuỗi request mẫu
cargo run --manifest-path examples/toi-noi-rust/31-axum-http/Cargo.toml -- --verify
cargo run --manifest-path examples/toi-noi-rust/31-axum-http/Cargo.toml -- --serve  # để tự gọi bằng curl, Ctrl-C để dừng
cargo test --manifest-path examples/toi-noi-rust/31-axum-http/Cargo.toml
```

Chứng minh: server bind `127.0.0.1:0` rồi đọc cổng thật; `--verify` tự khởi động server và gọi HTTP thật bằng `reqwest` (bảng mã trạng thái, số dư, 8 request đồng thời cùng key chỉ ra một 201); `cargo test` gọi `Router` như một `tower::Service` bằng `oneshot`, không mở cổng. Mỗi lần chạy dùng schema `lab_<uuid>` riêng và dọn khi xong. Thiếu `TOI_NOI_PG_URL` hoặc không kết nối được thì lab FAIL.
Sạch với `cargo clippy --all-targets -- -D warnings` và `cargo fmt --check`. Dependency chính: `axum`, `tower`, `sqlx`, `tokio`, `reqwest` (chỉ là client của `cargo run` và `--verify`, server không dùng).
