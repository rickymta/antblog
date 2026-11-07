# Lab 45: khuyến mãi, trợ giá và ngân sách campaign

Chuyến `TN-0002` cước 25.500, mã giảm 5.000 do nền tảng chịu: khách trả 20.500, tài xế vẫn nhận 20.400, doanh thu 5.100 (phí 20% trên cước gốc), chi phí khuyến mãi 5.000. Ngân sách cấp từ `platform:treasury`, phần trả thay khách đi từ `platform:promo-wallet`. Ngân sách campaign là tài khoản `promo:budget:<campaign>` có giới hạn; lab chứng minh bằng `Barrier` rằng bản có `for update` không bị vượt còn bản không khóa thì bị vượt.

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run  --manifest-path examples/toi-noi-rust/45-promo-budget/Cargo.toml
cargo run  --manifest-path examples/toi-noi-rust/45-promo-budget/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/45-promo-budget/Cargo.toml
```

- `src/promo.rs`: dựng bút toán `promo`, `capture`, `split`, `subsidy`; hoàn chuyến bằng bút toán đảo.
- `src/store.rs`: ghi idempotent + kiểm ngân sách trong cùng transaction (`BudgetGuard::Locked` / `Unlocked`), `PostError` kèm mã gRPC cho Promotion.
- `src/race.rs`: cuộc đua `Barrier`. `src/scenarios.rs`: kịch bản và `--verify`. `sql/budget_floor.sql`: trigger sàn số dư, cài riêng.
- Mỗi ca chạy trong schema `lab_<uuid>` và luôn dọn (`with_db`). Thiếu `TOI_NOI_PG_URL` thì fail rõ, không skip.
