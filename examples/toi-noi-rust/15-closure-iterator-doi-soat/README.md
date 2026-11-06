# Lab 15: closure, iterator adapter và lazy evaluation

Đối soát trong bộ nhớ giữa sổ cái (`Ledger`, mô hình của bài 10) và sao kê VíXanh giả dạng văn bản, khớp theo `external_ref`: tìm thiếu bên PSP, thiếu bên ta, lệch số tiền.

```bash
cargo run --manifest-path examples/toi-noi-rust/15-closure-iterator-doi-soat/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/15-closure-iterator-doi-soat/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/15-closure-iterator-doi-soat/Cargo.toml
```

Chứng minh: `Fn`/`FnMut`/`FnOnce` (`visit_common`, `group_by`), `move` khi trả closure (`at_least`), `impl Iterator` (`Ledger::entries_with_id`), `map`/`filter`/`filter_map`/`zip`/`take_while`/`fold`/`try_fold`, group-by thủ công bằng `entry().or_default()`, `collect` vào `HashMap`/`BTreeMap`/`Result<Vec<_>, _>`, và lazy bằng `inspect` (`lazy_probe`).

Mã nguồn mới: `src/reconcile.rs`, `src/statement.rs`. Các module còn lại copy từ lab 10 (thêm `Money::checked_sub`, `Ledger::entries_with_id`). Chỉ dùng thư viện chuẩn. Dữ liệu hư cấu của Tới Nơi.

Quy ước riêng của lab: bút toán thu qua VíXanh có `idempotency_key` = `psp:vixanh:<external_ref>` để có sẵn khóa khớp; không thay `trip:TN-0001:capture` của các bài khác. Dấu như bước capture của bài 13: ví khách âm, `psp:vixanh:clearing` dương.
