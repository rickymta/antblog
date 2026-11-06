# Lab 54: use case thanh toán chuyến end-to-end (PostgreSQL + Kafka)

`TripCompleted` và `PaymentCaptured` (đến thứ tự nào cũng được) → aggregate `TripPayment` có `version` → hai bút toán `trip:TN-0001:capture` và `trip:TN-0001:split` qua `psp:vixanh:clearing` → outbox → `LedgerEntryPosted` → số dư; có nhánh khuyến mãi, hoàn một phần, sự kiện trùng, tiến trình con tự `abort` ở hai điểm chọn trước quanh `COMMIT` và đối soát theo dòng clearing. Bốn crate theo bài 51: `domain` ← `application` ← `infrastructure` ← `api`.

Cần hạ tầng `../infra` và `TOI_NOI_PG_URL`, `TOI_NOI_KAFKA` (xem `../infra/README.md`); thiếu thì lab FAIL rõ.

```bash
cargo run  --manifest-path examples/toi-noi-rust/54-trip-payment-use-case/Cargo.toml              # kể lại từng kịch bản
cargo run  --manifest-path examples/toi-noi-rust/54-trip-payment-use-case/Cargo.toml -- --verify  # kiểm, in VERIFY OK
cargo test --manifest-path examples/toi-noi-rust/54-trip-payment-use-case/Cargo.toml
```

Chứng minh: hai thứ tự đến cho cùng sổ; thua tranh chấp `version` thì đọc lại và vẫn chốt đúng một lần (cả đường tạo lẫn đường cập nhật); chết trước và sau `COMMIT` rồi chạy lại vẫn đúng hai bút toán; không test nào assert thời gian. Mỗi ca dùng schema `lab_<uuid>`, topic `lab.54.<uuid>.*` và group riêng, dọn cuối ca. Dùng lại `ledger-core` của lab 20; trigger cân bút toán là bản của lab 25 (test kiểm giống từng byte).
