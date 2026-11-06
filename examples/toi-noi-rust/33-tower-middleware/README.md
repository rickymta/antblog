# Lab 33: middleware với tower cho API sổ cái

Không cần hạ tầng ngoài: sổ cái là `ledger-core` (lab 20) trong bộ nhớ, khoá JWT HS256 sinh tại chỗ mỗi lần chạy (giá trị lab, không phải secret).

```bash
cargo run --manifest-path examples/toi-noi-rust/33-tower-middleware/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/33-tower-middleware/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/33-tower-middleware/Cargo.toml
```

Chứng minh bằng chạy thật: thứ tự layer của `ServiceBuilder` và `Router::layer` ngược nhau; `x-request-id` vào response và span; `TimeoutLayer` trả 504 và huỷ handler (mặc định cũ là 408); CORS chỉ khi cấu hình; JWT hết hạn, sai chữ ký, sai `aud`, sai thuật toán đều 401; `finance` ghi, `cs` chỉ đọc, thiếu quyền 403 đến trước lỗi body; `Authorization` bị che trong log (có đối chứng); concurrency limit xếp hàng hay từ chối (`LoadShed`), cái bẫy `with_state` và semaphore theo từng route; `RateLimit` cần `Buffer`, và qua `Router::layer` thì mỗi route một bucket; bọc cả router thành một service mới là hạn mức toàn cục.

Phụ thuộc chính: `axum` 0.8, `tower` 0.5, `tower-http` 0.7, `jsonwebtoken` 11 (`rust_crypto`), `tracing-subscriber`, `jiff`, `ledger-core`.
