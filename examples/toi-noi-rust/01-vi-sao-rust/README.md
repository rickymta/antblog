# Lab 01 — Vì sao sổ cái Tới Nơi viết bằng Rust

Bài: `rust-vi-sao-so-cai-toi-noi-chon-rust`. Một binary chỉ dùng thư viện chuẩn, không crate ngoài.

```bash
cargo run --manifest-path examples/toi-noi-rust/01-vi-sao-rust/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/01-vi-sao-rust/Cargo.toml -- --verify
```

Lab chứng minh ba điều: `checked_add` trả `None` khi tràn `i64`; `match` vét cạn trên 6 trạng thái thanh toán
(thêm biến thể mà quên xử lý là lỗi E0004); `Drop` chạy tất định theo thứ tự ngược khai báo.
Hai lỗi biên dịch E0004 và E0502 được giữ dưới dạng chú thích trong `src/main.rs`.
