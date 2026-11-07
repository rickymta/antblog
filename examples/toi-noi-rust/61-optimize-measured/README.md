# Lab 61: tối ưu có số đo

Dùng lại `ledger-core` (lab 20) và đúng schema, trigger cân bút toán của lab 25. Chứng minh bằng chỉ số không phụ thuộc tải (số cấp phát, số message ở tầng dây) trước, thời gian sau: `Cow` khi chuẩn hóa mã tài khoản, mượn `&str` thay `clone`, ghi từng dòng so với `unnest`, `COPY` (`copy_in_raw`) và một câu CTE (giữ idempotency và `account_balances`), prepared statement của sqlx, trigger chạy theo dòng, p50/p99 với coordinated omission.

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'   # compose: infra/README.md
cargo run --manifest-path examples/toi-noi-rust/61-optimize-measured/Cargo.toml                 # phần không cần database
cargo run --manifest-path examples/toi-noi-rust/61-optimize-measured/Cargo.toml -- --verify     # in VERIFY OK
cargo test --manifest-path examples/toi-noi-rust/61-optimize-measured/Cargo.toml
cargo run --release --manifest-path examples/toi-noi-rust/61-optimize-measured/Cargo.toml -- --wire      # round trip, byte
cargo run --release --manifest-path examples/toi-noi-rust/61-optimize-measured/Cargo.toml -- --prepared  # Parse, cache
cargo run --release --manifest-path examples/toi-noi-rust/61-optimize-measured/Cargo.toml -- --pg-bench --n 10000
cargo run --release --manifest-path examples/toi-noi-rust/61-optimize-measured/Cargo.toml -- --latency --rate 20
cargo run --release --manifest-path examples/toi-noi-rust/61-optimize-measured/Cargo.toml -- --breakdown --n 200  # vì sao 9 round trip không chậm gấp 9 lần
```
`--verify` và `cargo test` không assert thời gian; thiếu `TOI_NOI_PG_URL` thì lỗi rõ, không bỏ qua. Mỗi ca chạy trong schema `lab_<uuid>` riêng và xóa nó khi xong. `--wire` và `--prepared` đếm qua một proxy TCP của lab (không đổi nội dung message): round trip = số `ReadyForQuery` + số `CopyInResponse`. Số đo thời gian chỉ in khi chạy `--pg-bench`, `--latency`, `--breakdown`, là số trên máy đã đo, không phải SLA. `--breakdown` đặt `track_wal_io_timing` theo phiên, cần role superuser như compose của lab.
Dependency: `ledger-core` (path), `sqlx` 0.9, `tokio`, `uuid`. Allocator đếm và histogram tự viết. Dữ liệu hư cấu của Tới Nơi.
