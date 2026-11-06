# Lab 47: đối soát sao kê VíXanh với sổ cái PostgreSQL

Chứng minh bằng chạy thật: `csv` đọc streaming (dòng hỏng vào danh sách lỗi có số dòng, file vẫn chạy tiếp), SHA-256 của file cùng với kỳ làm khóa idempotent, bảng tạm + `COPY` (`copy_in_raw` của sqlx) rồi khớp bằng SQL theo `external_ref` (1-1, 1-n), phân loại lệch (thiếu bên ta, thiếu bên PSP, lệch tiền, trùng, khác ngày do cắt kỳ), ghi `reconciliation_runs`/`reconciliation_items`, job định kỳ với đồng hồ truyền vào, báo cáo CSV. Lệch không tự sửa: chỉ mở việc `OPEN` cho Finance.

Cần PostgreSQL của `examples/toi-noi-rust/infra`. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/47-psp-reconciliation/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/47-psp-reconciliation/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/47-psp-reconciliation/Cargo.toml
```

Đo file lớn (sinh tại chỗ trong `target/`, không commit): `cargo build --release`, rồi `target/release/ledger_47_psp_reconciliation gen target/vixanh_1m.csv 1000000` và `/usr/bin/time -l target/release/ledger_47_psp_reconciliation bench sql target/vixanh_1m.csv 1000000` (hoặc `bench rust`). Mỗi ca chạy trong schema `lab_<uuid>` riêng, dọn cuối ca. Cần tzdb của hệ điều hành cho `Asia/Ho_Chi_Minh` (jiff).
