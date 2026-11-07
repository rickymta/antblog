# Lab 49: event sourcing có chừng mực, projection và replay trên PostgreSQL

Dùng chính `journal_entries` + `postings` làm event log (append-only, bài 24). Hai projection (số dư, thu nhập theo ngày) có checkpoint `projection_checkpoints`, xử lý theo lô trong cùng transaction với checkpoint. Lab dựng lại bằng hai transaction có kiểm soát lỗi «commit muộn» của `id > checkpoint`, sửa bằng khoảng an toàn `pg_snapshot_xmin` và con trỏ `(tx_id, id)`, rồi dựng lại projection từ đầu vào bảng mới, so sánh, đổi tên nguyên tử, đổi phiên bản và đo lag.

Cần PostgreSQL của `examples/toi-noi-rust/infra`. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/49-projection-replay/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/49-projection-replay/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/49-projection-replay/Cargo.toml
```

Mỗi thí nghiệm chạy trong schema `lab_<uuid>` riêng và dọn khi xong. Test và `--verify` không assert thời gian; thời gian dựng lại chỉ được in. Dùng lại `ledger-core` của lab 20 (đường dẫn tương đối).
