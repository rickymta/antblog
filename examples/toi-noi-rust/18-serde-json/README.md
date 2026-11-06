# Lab 18: serde, JSON, Decimal và thời gian

Thêm hợp đồng JSON cho sổ cái: sự kiện `LedgerEntryPosted` / `LedgerEntryReversed` (`src/event.rs`) ghi và đọc bằng `serde` + `serde_json`. Khóa camelCase khớp BFF C#, `Money` là số nguyên đồng, `feeRate` là chuỗi `"0.20"` (`rust_decimal`, feature `serde-with-str`), `postedAt` là `jiff::Timestamp` UTC (feature `serde`), `schemaVersion: 1`, `tag = "type"`. Đọc JSON đi qua `try_from` nên không tạo được bút toán lệch.

```bash
cargo run --manifest-path examples/toi-noi-rust/18-serde-json/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/18-serde-json/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/18-serde-json/Cargo.toml
```

Chứng minh: JSON ghi ra khớp file `tests/golden/ledger_entry_posted.v1.json`, round-trip bằng nhau, lỗi đọc có `line()`/`column()`, 16 unit test (module từ lab 10), 15 test hợp đồng ở `tests/json_contract.rs`, 2 doc test; sạch với `cargo clippy --all-targets -- -D warnings` và `cargo fmt --check`.

Dữ liệu hư cấu của Tới Nơi, không credential.
