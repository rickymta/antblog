# Lab 67: vận hành sổ cái, SLO, alert, runbook, replay và backup

Chạy thật trên PostgreSQL và Kafka của `examples/toi-noi-rust/infra`. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
export TOI_NOI_KAFKA='127.0.0.1:9092'
cargo run  --manifest-path examples/toi-noi-rust/67-operations-runbook/Cargo.toml              # kể lại từng ca
cargo run  --manifest-path examples/toi-noi-rust/67-operations-runbook/Cargo.toml -- --verify  # in VERIFY OK
cargo test --manifest-path examples/toi-noi-rust/67-operations-runbook/Cargo.toml
```

Chứng minh: SLI bằng SQL (tuổi outbox, độ tươi theo bucket, tổng sổ, lệch projection) và Kafka (consumer lag); burn rate đa cửa sổ bằng hàm thuần toàn số nguyên (`slo.rs`, không đọc đồng hồ); alert `reconciliation-mismatch` và các alert khác (`alert.rs`) chỉ báo người, không sửa sổ; sáu ca diễn tập (relay chết, message hỏng vào DLQ, replay dry-run/giới hạn tốc độ/idempotent, lệch đối soát, kỳ đã đóng, tổng sổ khác 0); `pg_dump` thật trong container PostgreSQL của lab rồi `pg_restore` vào database tạm, so số dư và hash chain.
Lab không dựng WAL archiving nên không diễn tập PITR (chỉ đọc cấu hình và nói rõ). `docker exec` chỉ chạy `pg_dump`, `pg_restore`, `psql`, `stat`, `rm` trên file và database tạm của lab; cần Docker và container `toi-noi-rust-pg` (đổi bằng `TOI_NOI_PG_CONTAINER`).
Dùng lại `ledger-core` lab 20; `01_schema.sql`, `02_balance_check.sql` là bản của lab 25, `03_append_only.sql` của lab 58. Mỗi ca dùng schema `lab_<uuid>`, topic `lab.67.<uuid>.*`, group riêng, dọn cuối ca.
