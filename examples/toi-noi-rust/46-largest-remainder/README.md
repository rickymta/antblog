# Lab 46: làm tròn và chia tiền không mất đồng (largest remainder)

Chạy từ gốc repository (không cần PostgreSQL, Redis hay Kafka):

```bash
cargo run --manifest-path examples/toi-noi-rust/46-largest-remainder/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/46-largest-remainder/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/46-largest-remainder/Cargo.toml
```

Chứng minh: làm tròn từng phần làm lệch đồng (kể cả với `rust_decimal`); bảy `RoundingStrategy` cho kết quả khác nhau; `allocate` (largest remainder, phá hòa theo `key`, số âm, trọng số 0, tính bằng `i128`) giữ tổng đúng từng đồng; kết quả nối được vào `ledger-core` (`JournalEntry`, `FeePolicy`). Proptest kiểm tổng, độ lệch dưới 1 đồng, tất định theo thứ tự đầu vào, hoàn tiền đối xứng, thứ tự trọng số; nó cũng tìm phản ví dụ của cách làm tròn ngây thơ.

Dependency: `ledger-core` (path, lab 20), `rust_decimal`, `proptest` (ở `[dependencies]` vì binary chạy phần tìm phản ví dụ). Dữ liệu hư cấu của Tới Nơi. `src/search.rs` tắt `failure_persistence`; `tests/properties.rs` chỉ ghi `tests/properties.proptest-regressions` khi một property fail (lúc này chưa có).
