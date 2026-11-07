# Lab 48: số dư theo thời điểm, snapshot và đóng kỳ

Bài: `rust-so-du-theo-thoi-diem-snapshot-dong-ky`. Chứng minh bằng PostgreSQL thật: số dư «lúc 23:59» theo
`Asia/Ho_Chi_Minh` với ranh giới nửa mở `[from, to)` và lỗi lệch ngày UTC; `posted_at` (xảy ra) khác `recorded_at` (ghi);
`balance_snapshots` hằng ngày, «snapshot + phần chênh» khớp tính lại toàn bộ (proptest + 400 mẫu), `EXPLAIN` trên 150.400 posting;
`accounting_periods` với trigger cấm ghi vào kỳ đã đóng, điều chỉnh ở kỳ sau bằng bút toán đảo có `reversal_of`;
cuộc đua «ghi đang chạy lúc đóng kỳ» dựng bằng `Barrier` (bản không khoá để lộ lỗi, bản `FOR SHARE`/`FOR UPDATE` đúng).

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/48-snapshot-period-close/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/48-snapshot-period-close/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/48-snapshot-period-close/Cargo.toml
```

SQL nằm trong `sql/` (01 schema, 02 trigger chặn kỳ đóng, 03 bản sai không khoá, 04 trigger vô hiệu snapshot). Mọi DDL trong schema `lab_<uuid>`, xoá cuối ca. Thời gian in ra là số đo trên một máy, không phải SLA; `--verify` không kiểm thời gian.
