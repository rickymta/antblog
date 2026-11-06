# Lab 25: schema sổ cái PostgreSQL

Bài: `rust-schema-so-cai-accounts-journal-postings-partition`. Chứng minh bằng PostgreSQL thật:
bốn bảng chuẩn (`accounts`, `journal_entries`, `postings`, `account_balances`), CHECK từng dòng,
kiểm tổng bằng 0 bằng constraint trigger `DEFERRABLE INITIALLY DEFERRED` lúc commit, projection số dư
cập nhật cùng transaction bằng `ON CONFLICT DO UPDATE`, đối chiếu với `SUM(postings)`, index và
`EXPLAIN (ANALYZE, BUFFERS)` trên 120.000 posting, bản `postings_p` partition theo tháng.

Cần PostgreSQL lab đang chạy (`infra/README.md`). Mọi DDL nằm trong schema `lab_<uuid>` và bị xoá cuối ca.

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/25-schema-so-cai/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/25-schema-so-cai/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/25-schema-so-cai/Cargo.toml
```

SQL nằm trong `sql/` (01 schema, 02 trigger kiểm cân, 03 index, 04 partition). Thời gian in ra là số đo trên một máy, không phải SLA.
