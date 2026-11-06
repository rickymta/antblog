# Lab 13: double-entry, journal entry, posting và bất biến tổng bằng 0

Thư viện `ledger_lab` mở rộng lab 10: `JournalEntry` chỉ tạo được khi cân (`try_new`, `JournalEntryBuilder`), chiều tiền (`Flow`, dấu của posting không phải nợ/có kế toán), chart of accounts rút gọn (`AccountKind`), bút toán đảo (`Ledger::reverse`) và số dư là projection (`Ledger::balances`). Binary `ledger-cli` chạy dòng tiền hai bước của chuyến `TN-0001`: capture `JE-000001`, chia `JE-000002`, đảo `JE-000003`.

```bash
cargo run --manifest-path examples/toi-noi-rust/13-double-entry-journal/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/13-double-entry-journal/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/13-double-entry-journal/Cargo.toml
```

Chứng minh: bút toán lệch không tạo được; builder và `try_new` dùng chung một hàm kiểm tra; bút toán đảo không xoá bản gốc và idempotent; số dư tính lại từ posting, còn ô balance lưu sẵn thì mất cập nhật khi hai bên ghi xen kẽ (`naive::lost_update`).

Chỉ dùng thư viện chuẩn (không có dependency). Dữ liệu hư cấu của Tới Nơi.
