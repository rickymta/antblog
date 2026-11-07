# Lab 42: hold, capture, void và số dư khả dụng

Chứng minh bằng chạy thật trên PostgreSQL: hold không ghi bút toán, chỉ giảm `available = balance - SUM(phần còn lại của hold ACTIVE)`; capture toàn phần và một phần (phần dư được nhả); void khi hủy chuyến; hai hold đồng thời vượt khả dụng (`Barrier`): bản không khóa bị vượt, `SELECT ... FOR UPDATE` trên `account_balances` thì đúng một bên thắng; gộp khóa và `SUM` vào một câu thì `SUM` đọc snapshot cũ; ví chưa có hàng số dư bị từ chối rõ; job hết hạn dùng clock truyền vào với `FOR UPDATE SKIP LOCKED`; capture đến sau hạn; capture lặp lại idempotent. `ActiveHold` là typestate cục bộ trong một transaction; `HoldStatus` là enum ở biên DB.

Cần PostgreSQL của `examples/toi-noi-rust/infra` (không dùng Redis/Kafka). Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/42-hold-capture-void/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/42-hold-capture-void/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/42-hold-capture-void/Cargo.toml
```

Mỗi thí nghiệm chạy trong schema `lab_<uuid>` riêng và `DROP SCHEMA ... CASCADE` khi xong. Thiếu `TOI_NOI_PG_URL` thì lab thoát mã 1. Lab không `sleep` để chờ hết hạn (dùng `ManualClock`); chỉ một chỗ poll `pg_stat_activity` chờ phiên bị chặn, theo điều kiện chứ không theo thời gian. Dùng lại `ledger-core` của lab 20.
