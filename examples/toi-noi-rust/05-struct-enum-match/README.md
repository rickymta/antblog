# Lab 05 · Struct, enum và pattern matching

Bài: `rust-struct-enum-match-tai-khoan-but-toan`. Chỉ dùng std, không có dependency.

```bash
cargo run --manifest-path examples/toi-noi-rust/05-struct-enum-match/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/05-struct-enum-match/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/05-struct-enum-match/Cargo.toml
```

Lab chứng minh: `struct` (có tên trường, tuple struct, unit struct), `impl` với `&self`/`&mut self`/`self`, `enum` có dữ liệu (`AccountKind`, `PaymentState`, `EntryKind`), `match` vét cạn với guard và `@`, `if let`, `let else`, `impl Display`, bút toán 3 posting cân (-25.500, +20.400, +5.100) và máy trạng thái thanh toán trả `None` cho chuyển không hợp lệ.
