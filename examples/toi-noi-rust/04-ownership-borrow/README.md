# Lab 04: ownership, move và borrow

Bài `rust-ownership-move-borrow-but-toan`. Chỉ dùng thư viện chuẩn, không có dependency.

```bash
cargo run --manifest-path examples/toi-noi-rust/04-ownership-borrow/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/04-ownership-borrow/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/04-ownership-borrow/Cargo.toml
```

Lab chứng minh: `i64` và `Money` là `Copy`, `String` bị move; ra khỏi scope thì `drop`; `post(&mut Ledger, JournalEntry)` nhận ownership của bút toán; `balance(&Ledger, &str)` chỉ mượn; số dư `psp:vixanh:clearing` sau khi post `JE-000001` là 25.500.
Các lỗi E0382, E0502, E0499, E0106 được giữ dạng chú thích `// Lỗi ... nếu bỏ comment` trong `src/main.rs`.
