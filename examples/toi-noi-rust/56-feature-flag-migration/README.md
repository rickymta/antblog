# Lab 56: feature flag, cấu hình kiểu an toàn và migration không dừng

Đổi sổ cái đang chạy: thêm `postings.currency` và `fee_events.policy_version` bằng expand/contract với hai phiên bản code cùng chạy (`AppV1`, `AppV2`), backfill theo lô có checkpoint, quan sát khóa thật qua `pg_locks`, `lock_timeout` và thử lại, bật quy tắc phí mới bằng cờ theo `tripId`, ghi `policy_version` vào sự kiện để replay ra đúng số cũ, rollback trước và sau contract, cấu hình enum kiểm lúc khởi động.

Cần PostgreSQL của `examples/toi-noi-rust/infra`. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/56-feature-flag-migration/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/56-feature-flag-migration/Cargo.toml
```

Mỗi thí nghiệm chạy trong schema `lab_<uuid>` riêng và dọn khi xong; chờ khóa được quan sát qua `pg_stat_activity`, không dùng `sleep` để đoán. `--verify` và test không assert thời gian. Dùng lại `ledger-core` của lab 20 và trigger cân bút toán của bài 25 (`sql/02_balance_check.sql`, bản sao nguyên văn).
