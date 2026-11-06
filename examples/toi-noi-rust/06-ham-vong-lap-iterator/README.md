# Lab 06: hàm, điều kiện, vòng lặp và iterator

Bài: `rust-ham-dieu-kien-vong-lap-iterator-so-du`. Chỉ dùng thư viện chuẩn, không có dependency.

```bash
cargo run --manifest-path examples/toi-noi-rust/06-ham-vong-lap-iterator/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/06-ham-vong-lap-iterator/Cargo.toml -- --verify
```

Lab chứng minh:
- `fare(5)` = 25.500, `fare(2)` = 12.000, `fare(7)` = 34.500, viết bằng vòng lặp và bằng biểu thức cho cùng kết quả.
- `loop` với `break giá_trị`, `while`, `for` trên `0..=n`.
- Số dư `driver:DR-001` = 20.400 từ `Vec<Posting>`, vòng `for` và iterator cho cùng kết quả; tổng mọi posting = 0.
- Adapter của iterator lazy: dựng `filter` chưa chạy closure.
