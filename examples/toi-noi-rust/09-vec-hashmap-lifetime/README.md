# Lab 09: Vec, HashMap, BTreeMap và lifetime cơ bản

Sổ cái trong bộ nhớ (chỉ dùng thư viện chuẩn, không có dependency): `Vec<Posting>`, `BTreeMap` cho bút toán theo id,
`HashMap` cho số dư theo tài khoản, `HashSet` cho idempotency key, `VecDeque` làm hàng đợi, cùng hai hàm minh họa lifetime.

```bash
cargo run --manifest-path examples/toi-noi-rust/09-vec-hashmap-lifetime/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/09-vec-hashmap-lifetime/Cargo.toml -- --verify
```

`--verify` kiểm: số dư `driver:DR-001` = 57.600 và `platform:revenue` = 14.400 sau 3 chuyến; tổng mọi số dư = 0;
ghi trùng `trip:TN-0001:capture` bị từ chối; `range` từ `JE-000002` trả 2 bút toán; `v[7]` panic còn `get(7)` trả `None`.
Tiền ở đây là `i64` đơn vị đồng để tập trung vào collection (newtype `Money` ở bài 04 và 05).
