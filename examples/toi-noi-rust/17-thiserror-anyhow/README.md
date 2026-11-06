# Lab 17: thiserror, anyhow và ngữ cảnh lỗi

Thay `impl Display/Error` viết tay của `LedgerError` (lab 10) bằng `thiserror`, tách lỗi domain (`LedgerError`) khỏi lỗi hạ tầng (`StorageError`: `io::Error`, `ParseIntError`), rồi dùng `anyhow` ở biên (`src/app.rs`, `main`) để thêm ngữ cảnh và phân loại lại bằng `downcast_ref`.

```bash
cargo run --manifest-path examples/toi-noi-rust/17-thiserror-anyhow/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/17-thiserror-anyhow/Cargo.toml -- --verify
cargo run --manifest-path examples/toi-noi-rust/17-thiserror-anyhow/Cargo.toml -- --scenario disk-full
cargo run --manifest-path examples/toi-noi-rust/17-thiserror-anyhow/Cargo.toml -- --scenario conflict --exit-code
cargo test --manifest-path examples/toi-noi-rust/17-thiserror-anyhow/Cargo.toml
```

Kịch bản: `ok`, `unbalanced`, `conflict`, `too-many`, `disk-full`, `bad-config`, `missing-config`. Không có `--exit-code` thì `main` trả `Err` (mã 1); có thì thoát theo bảng 2/3/4/1. Kho lưu trữ là giả lập trong bộ nhớ. Dữ liệu hư cấu của Tới Nơi.
