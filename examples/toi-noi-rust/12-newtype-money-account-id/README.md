# Lab 12: newtype và value object (Money, AccountId, EntryId)

Tiếp lab 10: `Money<C>` mang loại tiền ở kiểu (phantom type `Vnd`/`Usd`) với `Add`/`Sub` trả `Result`, `TryFrom<i128>`; `AccountId` parse bằng `FromStr`/`TryFrom<&str>` có kiểm tra; `EntryId` dạng `JE-000001`; `Km` là newtype thứ tư để thấy `Money + Km` không biên dịch.

```bash
cargo run --manifest-path examples/toi-noi-rust/12-newtype-money-account-id/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/12-newtype-money-account-id/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/12-newtype-money-account-id/Cargo.toml
```

Chứng minh: cộng khác loại tiền, cộng Money với số trần hoặc `Km`, gán `EntryId` vào chỗ cần `AccountId` đều không biên dịch (bốn doc test `compile_fail,E0308`; trên stable rustdoc không đối chiếu mã lỗi, mã E0308 được kiểm bằng rustc trong crate nháp); tràn `i64` thành `Err(Overflow)` thay vì panic hay quay vòng; `?` đổi `MoneyError`/`IdError` thành `LedgerError` nhờ `impl From`; `derive` đúng chỗ (`Copy` cho `Money`/`EntryId`, không cho `AccountId`).
Sạch với `cargo clippy --all-targets -- -D warnings` và `cargo fmt --check`. Chỉ dùng thư viện chuẩn. Dữ liệu hư cấu của Tới Nơi.
