# Lab 68 — Bài tổng hợp toàn loạt

Bài: `rust-tong-hop-so-cai-tu-quyet-dinh-chon-rust-toi-van-hanh`. Một binary chỉ đọc file, dependency duy nhất là `serde_json`; không cần PostgreSQL, Kafka hay Docker.

```bash
cargo run --manifest-path examples/toi-noi-rust/68-series-wrapup/Cargo.toml                    # bản đồ 68 bài
cargo run --manifest-path examples/toi-noi-rust/68-series-wrapup/Cargo.toml -- --routes        # ba đường đọc
cargo run --manifest-path examples/toi-noi-rust/68-series-wrapup/Cargo.toml -- --evidence      # sổ bằng chứng cho ADR bài 01
cargo run --manifest-path examples/toi-noi-rust/68-series-wrapup/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/68-series-wrapup/Cargo.toml
```

`--verify` quét từ gốc repo: 68 bài trong `docs/rust-master-curriculum.md`, ba lộ trình trong `api/seed/learning-paths.json` đủ và đúng thứ tự, mỗi bài có file (`slug`, «Bài NN:») và thư mục lab có `Cargo.toml`, đường đọc và sổ bằng chứng hợp lệ. Thiếu gì thì liệt kê đúng chỗ thiếu và thoát mã 1.

Lab chỉ kiểm hình thức, không biết bài viết đúng hay sai và không chạy lại lab khác.
