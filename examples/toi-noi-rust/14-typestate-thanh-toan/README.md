# Lab 14: typestate và máy trạng thái thanh toán

Mô hình hóa sáu trạng thái thanh toán (`INITIATED` … `REFUNDED`) theo hai cách, cùng chạy chuyến `TN-0001` (25.500 VND, phí 5.100, tài xế 20.400): `enum PaymentState` + `transition` trả `Result` (kiểm tra lúc chạy, `src/state.rs`, `src/record.rs`) và typestate `Payment<S>` với `PhantomData` (kiểm tra lúc biên dịch, `src/typestate.rs`). Biên enum ↔ typestate qua `to_record`, `TryFrom<PaymentRecord>`, `AnyPayment`.

```bash
cargo run --manifest-path examples/toi-noi-rust/14-typestate-thanh-toan/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/14-typestate-thanh-toan/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/14-typestate-thanh-toan/Cargo.toml
```

Chứng minh: 6/30 cặp (trạng thái, sự kiện) hợp lệ; `capture` trên `Payment<Initiated>` và thu hai lần không biên dịch được (doc test `compile_fail`; rustc báo E0599 và E0382, nhưng rustdoc trên stable không đối chiếu mã lỗi); hai cách cho cùng một sổ cái. Chỉ dùng thư viện chuẩn. Dữ liệu hư cấu của Tới Nơi.
