# Lab 16: Box, Rc, Arc, RefCell và Cell

Thư viện `ledger_lab` (copy 5 module của lab 10, thêm `boxed`, `policy`, `tree`, `shared`, `threads`) và binary `ledger-cli`. Chỉ dùng thư viện chuẩn, không có dependency.

```bash
cargo run --manifest-path examples/toi-noi-rust/16-box-rc-arc-refcell/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/16-box-rc-arc-refcell/Cargo.toml -- --verify
cargo run --manifest-path examples/toi-noi-rust/16-box-rc-arc-refcell/Cargo.toml -- --panic-demo   # panic thật, exit 101
cargo test --manifest-path examples/toi-noi-rust/16-box-rc-arc-refcell/Cargo.toml
```

Chứng minh: `Box` cho kiểu đệ quy (`FareExpr`) và `Box<dyn FeePolicy>` cho chính sách phí khác kiểu; cây tài khoản `Rc` + `Weak` không rò, chu trình `Rc` mạnh thì rò; `RefCell` panic `RefCell already borrowed` khi giữ guard đọc rồi mượn ghi, cách sửa bằng thu hẹp guard và `try_borrow_mut`; `Cell` đếm tra cứu qua `&self`; `Arc<Mutex<Ledger>>` với 4 luồng. Sạch với `cargo clippy --all-targets -- -D warnings` và `cargo fmt --check`. Dữ liệu hư cấu của Tới Nơi.
