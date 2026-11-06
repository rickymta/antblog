# Lab 19: property-based test với proptest

Từ lab 10 (`Money`, `AccountId`, `Posting`, `JournalEntry`, `Ledger`) thêm `split` (chia tiền largest remainder, không mất đồng), `strategies` (bộ sinh bút toán luôn cân) và `shrink_demo` (bản `split_floor_only` cố ý sai để xem shrinking).

```bash
cargo run --manifest-path examples/toi-noi-rust/19-proptest-bat-bien/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/19-proptest-bat-bien/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/19-proptest-bat-bien/Cargo.toml
cargo test --manifest-path examples/toi-noi-rust/19-proptest-bat-bien/Cargo.toml -- --ignored   # cố ý FAIL, xem shrinking
```

Chứng minh: tổng posting bằng 0, tổng toàn sổ bằng 0 sau mọi dãy post, số dư khớp mô hình tham chiếu, ghi hai lần cùng key chỉ có một bút toán, `split` không mất đồng; proptest thu nhỏ phản ví dụ về `total = 1, weights = [1, 1]`.

Lệnh `--ignored` ghi file `tests/shrinking_demo.proptest-regressions`; file này do máy sinh, xoá đi sau khi xem (dự án thật nên commit để thành test hồi quy). Dependency duy nhất: `proptest` (nằm ở `[dependencies]` chỉ vì binary chạy demo; ở dự án thật để `[dev-dependencies]`). Dữ liệu hư cấu của Tới Nơi.
