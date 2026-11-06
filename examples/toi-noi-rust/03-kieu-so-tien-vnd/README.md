# Lab 03: kiểu số và tiền VND

Đi kèm bài `rust-kieu-so-tien-vnd-i64-decimal`. Lab chứng minh vì sao sổ cái lưu `i64` đơn vị đồng và chỉ dùng `rust_decimal` cho tỷ lệ phí.

```bash
cargo run --manifest-path examples/toi-noi-rust/03-kieu-so-tien-vnd/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/03-kieu-so-tien-vnd/Cargo.toml -- --verify
cargo run --manifest-path examples/toi-noi-rust/03-kieu-so-tien-vnd/Cargo.toml -- --overflow            # debug: panic
cargo run --release --manifest-path examples/toi-noi-rust/03-kieu-so-tien-vnd/Cargo.toml -- --overflow  # release: wrap
```

`--verify` kiểm: `0.1 + 0.2 != 0.3`, `checked_add(i64::MAX, 1) == None`, phí 20% của 25.500 là 5.100, của 25.503 là 5.101, tài xế nhận 20.400. Crate ngoài duy nhất: `rust_decimal`.
