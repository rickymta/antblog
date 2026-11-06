# Lab 27: integration test với PostgreSQL thật

Mỗi ca test tự dựng schema `lab_<uuid>`, chạy 3 migration trong đó, dùng pool có `search_path` riêng rồi `DROP SCHEMA ... CASCADE` bằng `close()`. Chạy song song không đụng nhau.

```bash
docker compose -f examples/toi-noi-rust/infra/docker-compose.yml up -d --wait
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run  --manifest-path examples/toi-noi-rust/27-integration-test-pg/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/27-integration-test-pg/Cargo.toml
```

- `tests/common/mod.rs`: fixture `TestDb` (binary dùng lại qua `#[path]`). Thiếu `TOI_NOI_PG_URL` thì test panic với cách bật compose, không skip.
- `tests/ledger_pg.rs`: cô lập song song (cùng `idempotency_key`), trigger append-only, constraint trigger hoãn, SQLSTATE 40001, schema sót và dọn theo tuổi.
- `tests/sqlx_test_compare.rs`: so sánh với `#[sqlx::test]`, chỉ bật bằng feature (cần quyền `CREATEDB`): `DATABASE_URL="$TOI_NOI_PG_URL" cargo test --features sqlx-test --test sqlx_test_compare`. Macro để lại schema `_sqlx_test` trong database lab (ca đỏ để lại cả database `_sqlx_test_*`); dọn bằng `drop schema _sqlx_test cascade` và `drop database` khi không còn ca nào chạy.
- `cargo run -- --cleanup-stale [giây]` xoá schema do fixture tạo mà quá tuổi (mặc định 3600); `--measure` đo thời gian dựng `TestDb`.
- `scripts/sqlite_contrast.sh`: thử các câu lệnh PostgreSQL trên `sqlite3`. `ci-example.yml`: workflow mẫu, chưa chạy trên GitHub.
