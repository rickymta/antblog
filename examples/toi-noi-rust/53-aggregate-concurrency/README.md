# Lab 53: aggregate Account, optimistic locking và SELECT FOR UPDATE

Chứng minh bằng chạy thật trên PostgreSQL: aggregate `Account` giữ invariant hạn mức rút trong domain; read-modify-write không khóa bị lost update và vượt hạn mức (`Barrier`); `account_balances.version` với `UPDATE ... WHERE version = $n` (0 dòng là xung đột), retry có giới hạn và jitter; `SELECT ... FOR UPDATE` cho cùng kết quả mà không retry; bẫy `JOIN ... FOR UPDATE OF b`; chuyển tiền hai ví ngược chiều dựng deadlock `40P01` thật, sửa bằng khóa theo id; khóa của nạn nhân nhả trước khi nó gửi `ROLLBACK`, còn kết nối thì báo `25P02` tới khi `ROLLBACK`; `NOWAIT` và `SKIP LOCKED`; `journal_entries` không cần version vì append-only.

Cần PostgreSQL của `examples/toi-noi-rust/infra`. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/53-aggregate-concurrency/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/53-aggregate-concurrency/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/53-aggregate-concurrency/Cargo.toml -- --measure
```

Mỗi thí nghiệm chạy trong schema `lab_<uuid>` riêng, dọn khi xong; thiếu `TOI_NOI_PG_URL` thì thoát mã 1. `--verify` và test không assert thời gian. `--measure` in số lần thử theo độ tranh chấp (có đo giờ, thay đổi mỗi lần chạy, không phải SLA). Dùng lại `ledger-core` của lab 20; trigger cân bút toán là bản của bài 25. Bảng `account_limits` là của riêng bài này, ngoài schema chuẩn.
