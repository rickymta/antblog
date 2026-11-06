# Lab 07: Result, Option và toán tử `?`

Chỉ dùng std, không có dependency. Chạy từ gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/07-result-option/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/07-result-option/Cargo.toml -- --verify
cargo run --manifest-path examples/toi-noi-rust/07-result-option/Cargo.toml -- --main-error   # thoát mã 1
cargo run --manifest-path examples/toi-noi-rust/07-result-option/Cargo.toml -- --panic-demo   # thoát mã 101
```

Lab chứng minh: `Result`/`Option` là enum; `LedgerError` có `Display` + `Error` viết tay; `JournalEntry::try_new` từ chối bút toán lệch (`Unbalanced { diff: -500 }`) và tràn (`Overflow`); `?` cùng `From<ParseIntError>`; ghi trùng `trip:TN-0001:capture` trả `DuplicateIdempotencyKey` và `post_idempotent` trả lại bút toán cũ; `expect` chỉ cho invariant của chính code.
