# Lab 11: trait, generics và chính sách phí thay thế được

Mở rộng thư viện `ledger_lab` của bài 10 bằng hai trait: `RoundingPolicy` (làm tròn về đồng, dùng `rust_decimal`) và `FeePolicy` (phí nền tảng), cộng `capture_entry` dựng bút toán thu cước chỉ qua trait.

```bash
cargo run --manifest-path examples/toi-noi-rust/11-trait-generics-phi/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/11-trait-generics-phi/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/11-trait-generics-phi/Cargo.toml
```

Chứng minh: default method (`round_ratio`, `split`); `PercentageFee<R: RoundingPolicy>` (generic, static dispatch); `impl FeePolicy` ở tham số và kiểu trả về; `Vec<Box<dyn FeePolicy>>` (dynamic dispatch, con trỏ 16 byte); `F: FeePolicy + ?Sized`; chính sách `TieredFee` thêm ở `main.rs` mà `capture_entry` không đổi.

Phụ thuộc: `rust_decimal`. Dữ liệu hư cấu của Tới Nơi.
