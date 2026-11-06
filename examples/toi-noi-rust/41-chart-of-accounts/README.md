# Lab 41: chart of accounts và dòng tiền một chuyến

Bài: `rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen`. Chứng minh bằng PostgreSQL thật: tám mẫu tài khoản
của sơ đồ chuẩn (`chart.rs`), dòng tiền hai bước qua clearing của chuyến `TN-0001` và đơn giao `DL-0001`
(`flows.rs`), dạng ba dòng cùng key bị `IdempotencyConflict`, và bốn metric đúng/sai bằng SQL
(`metrics.rs`): bút toán không cân, tổng toàn sổ khác 0, số dư âm bất thường theo loại, clearing tồn đọng.
Mục 10 chạy sổ riêng với nạp ví: clearing thành tài khoản biên mang số âm, đúng cách lab 42 dùng.

Cần PostgreSQL lab đang chạy (`infra/README.md`). Mọi DDL nằm trong schema `lab_<uuid>` và bị xoá cuối ca.
Thời gian nghiệp vụ là chuỗi ISO truyền vào, không đọc đồng hồ máy, không `sleep`.

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/41-chart-of-accounts/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/41-chart-of-accounts/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/41-chart-of-accounts/Cargo.toml
```

Dùng lại `ledger-core` của lab 20 qua path dependency. Ca "sổ đã hỏng" tắt trigger kiểm cân để dữ liệu lệch lọt vào: đó là cài lỗi có chủ đích, không phải đường ghi bình thường.
