# Lab 52: command handler, pipeline và domain event không framework

Chứng minh bằng chạy thật: `Command` có kiểu `Output` liên kết và `CommandHandler<C>`; thiếu handler là lỗi biên dịch E0277 (doc test `compile_fail` ở `src/lib.rs`), khác `DynBus` tra theo `TypeId` chỉ lỗi lúc chạy; pipeline `Validated<Traced<Transactional<H>>>` với thứ tự chứng minh bằng log; rollback khi lỗi ở handler và khi lỗi lúc COMMIT (trigger cân bút toán của lab 25); domain event gom trong aggregate, thu về sau handler, ghi outbox cùng transaction, không phát trước commit; đo kích thước future và số cấp phát của đường generic so với `Box`.

Cần PostgreSQL của `examples/toi-noi-rust/infra`. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/52-command-pipeline/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/52-command-pipeline/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/52-command-pipeline/Cargo.toml
```

Mỗi ca dùng schema `lab_<uuid>` riêng, xóa cuối ca. Thiếu `TOI_NOI_PG_URL` thì lab thoát mã 1. Dùng lại `ledger-core` của lab 20 và nạp `01_schema.sql`, `02_balance_check.sql` của lab 25. Trên Rust stable, doc test `compile_fail` chỉ kiểm code không biên dịch, không đối chiếu mã lỗi. Kích thước future của `CapturePayment` phụ thuộc bản build (debug khác release) và phiên bản rustc; số cấp phát thì ổn định. Số đo trên máy viết bài, không phải SLA.
