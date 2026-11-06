# Lab 43: refund, reversal, chargeback và maker-checker

Chuyến `TN-0001` (25.500 / phí 5.100 / tài xế 20.400, dòng tiền hai bước capture rồi split) được hoàn, đảo và bị chargeback trên PostgreSQL thật. Mỗi thí nghiệm chạy trong schema `lab_<uuid>` riêng và dọn khi xong.

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run  --manifest-path examples/toi-noi-rust/43-refund-reversal-chargeback/Cargo.toml
cargo run  --manifest-path examples/toi-noi-rust/43-refund-reversal-chargeback/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/43-refund-reversal-chargeback/Cargo.toml
```

- Đảo toàn phần có `reversal_of`, không đảo hai lần (replay theo key, chỉ mục duy nhất, không đảo bút toán đảo).
- Hoàn một phần tách theo tỷ lệ (nền tảng hoàn phần phí, tài xế hoàn phần còn lại); tổng hoàn không vượt số đã thu dưới `Barrier` + `FOR UPDATE`, và `CHECK` là chốt chặn khi thiếu khoá.
- Chargeback từ VíXanh: `psp:vixanh:clearing` đi âm, ba cách kết thúc (thu hồi tài xế, nền tảng chịu, thắng và đảo).
- Maker-checker: bảng `refund_requests`, `CHECK` người duyệt khác người tạo, duyệt lặp lại idempotent, audit ghi cả lần bị từ chối.
- Phụ thuộc `ledger-core` của lab 20 (không sửa). Cần PostgreSQL của `infra/`; thiếu `TOI_NOI_PG_URL` thì lab fail rõ.
