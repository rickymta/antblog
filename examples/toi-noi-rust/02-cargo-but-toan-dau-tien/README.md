# Lab 02: cargo và bút toán đầu tiên

Chương trình ghi một bút toán `JE-000001` cho chuyến `TN-0001` (khách `CU-001` trả 25.500 VND, hai posting, tổng bằng 0) rồi in số dư từng tài khoản. Chỉ dùng thư viện chuẩn, không có crate ngoài.

```bash
cargo run --manifest-path examples/toi-noi-rust/02-cargo-but-toan-dau-tien/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/02-cargo-but-toan-dau-tien/Cargo.toml -- --verify
```

Lab chứng minh: cấu trúc một crate binary (`Cargo.toml`, `Cargo.lock`, `src/main.rs`), `println!` với `{}` và `{:?}`, bất biến "tổng posting bằng 0", và chế độ `--verify` đọc từ `std::env::args` (thoát mã 1 nếu sai).
Đây là bản học tập: bút toán chưa được lưu, chưa kiểm tra lúc tạo, chưa có checked arithmetic (bài 03) và `Money` (bài 03, 05).
