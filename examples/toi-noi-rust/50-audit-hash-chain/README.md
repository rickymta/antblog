# Lab 50: audit, append-only, hash chain và phân quyền sổ cái

Chạy trên PostgreSQL thật (compose trong `../infra`). Mỗi ca tạo schema `lab_<uuid>` và năm role `lab_<uuid>_owner|writer|cs|finance|admin`, dọn cả hai cuối ca kể cả khi lỗi.

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run  --manifest-path examples/toi-noi-rust/50-audit-hash-chain/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/50-audit-hash-chain/Cargo.toml
cargo run  --release --manifest-path examples/toi-noi-rust/50-audit-hash-chain/Cargo.toml -- --measure
```

Lab chứng minh: `REVOKE` cho `permission denied` thật, trigger chặn cả superuser nhưng tắt được; chủ bảng vượt `REVOKE`; hash chain `SHA-256(prev_hash || canonical_bytes)` rẽ nhánh khi nối không khoá và tuần tự khi có advisory lock; `verify_chain` chỉ đúng dòng đầu tiên sai; chuỗi tính lại toàn bộ chỉ lộ nhờ điểm neo ngoài database; ma trận quyền Finance/CS/Admin đo bằng role thật; tên và số điện thoại không vào bảng, chỉ vào log ở dạng che.

`src/canonical.rs` là hàm thuần (đối chiếu độc lập bằng Python và `shasum -a 256`). `sql/` là ba lớp: schema, trigger, grant. `--measure` in số đo trên máy viết bài, không phải SLA; test không assert thời gian.
