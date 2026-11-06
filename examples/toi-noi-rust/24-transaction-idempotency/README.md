# Lab 24: transaction, isolation và idempotency key trên PostgreSQL

Chứng minh bằng chạy thật: ghi `journal_entries` + `postings` trong một transaction (lỗi giữa chừng không để lại gì, drop là rollback), `INSERT ... ON CONFLICT (idempotency_key) DO NOTHING RETURNING id` với nhiều task cùng key, `ON CONFLICT` với snapshot cũ (0 dòng ở READ COMMITTED, 40001 ở hai mức cao hơn), write skew ở READ COMMITTED và REPEATABLE READ so với SQLSTATE 40001 và retry có giới hạn ở SERIALIZABLE, trigger append-only cấm UPDATE/DELETE/TRUNCATE.

Cần PostgreSQL của `examples/toi-noi-rust/infra` (bài này không chạm Redis/Kafka). Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/24-transaction-idempotency/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/24-transaction-idempotency/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/24-transaction-idempotency/Cargo.toml
```

Mỗi thí nghiệm chạy trong schema `lab_<uuid>` riêng và `DROP SCHEMA ... CASCADE` khi xong, nên chạy lặp và song song được. Thiếu `TOI_NOI_PG_URL` hoặc không kết nối được thì lab thoát mã 1 kèm cách bật compose. Dùng lại `ledger-core` của lab 20 (đường dẫn tương đối, không sao chép).
