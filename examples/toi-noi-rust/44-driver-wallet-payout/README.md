# Lab 44: ví tài xế, thu nhập theo ngày và payout theo lô (PostgreSQL)

Chứng minh bằng chạy thật: thu nhập theo ngày `Asia/Ho_Chi_Minh` với chuyến qua nửa đêm UTC; lô payout gom ví đủ ngưỡng (`driver:<id>` âm, `bank:payout:clearing` dương) với trạng thái `CREATED`, `SENT`, `SETTLED`, `FAILED` và bút toán đảo khi thất bại; hạn mức rút theo `Clock` truyền vào; hai lô song song không chọn trùng nhờ `FOR UPDATE SKIP LOCKED` và unique theo kỳ; ngân hàng giả qua trait `Bank`, timeout sau khi đã nhận lệnh: tra cứu theo mã trước, không tự chi lại; đối chiếu sao kê; idempotency theo mã lô và tài xế. Không dùng largest remainder (bài 46).

Cần PostgreSQL của `examples/toi-noi-rust/infra`. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/44-driver-wallet-payout/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/44-driver-wallet-payout/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/44-driver-wallet-payout/Cargo.toml
```

Mỗi ca dùng schema `lab_<uuid>` riêng, xoá cuối ca. Thiếu `TOI_NOI_PG_URL` thì lab thoát mã 1 và nói cách bật compose. Thời gian nghiệp vụ đi qua `FixedClock`, không `sleep`, không assert thời gian. Dùng lại `ledger-core` của lab 20.
