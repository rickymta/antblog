# TEST-047 · MC/DC Testing — bài thực hành nâng cao Series 05

Bài: `/articles/mc-dc-testing`. Đề: thiết kế bộ test cho thuật toán áp voucher (SPEC R-VOUCHER-02…08) đạt
**100% statement, 100% branch** (`src/domain/voucher.ts`, `src/app/quote.ts`) và **MC/DC** cho quyết định
«voucher được áp» `V = C1 ∧ C2 ∧ C3 ∧ (C4 ∨ C5) ∧ (C6 ∨ C7) ∧ C8 ∧ C9`, chạy và nộp báo cáo coverage.

| Tệp | Nội dung |
|---|---|
| `mcdc.ts` | Bộ kiểm MC/DC tối giản: tìm cặp độc lập (unique-cause, `null` = điều kiện không được đánh giá) |
| `voucher-cases.ts` | 10 ca M1–M10 (N + 1, N = 9), hàm tính vectơ điều kiện từ dữ liệu, bộ chạy `priceQuote` thật |
| `mcdc-voucher.test.ts` | A: kiểm thiết kế (cặp độc lập, 100%) · B: chạy 10 ca · C: MC/DC trên từng quyết định của code |
| `bo-sung-statement-branch.test.ts` | S1–S6 để đạt 100% statement/branch · Đ1–Đ2 theo đặc tả (trần giảm, thứ tự lỗi) |
| `buggy-eligible.ts` | Bản lỗi `!kindOk && !cityOk` (chép, không sửa `src/`) |
| `bat-loi-and-or.test.ts` | Bộ decision coverage 3 ca không phân biệt bản đúng/bản lỗi; ca MC/DC M5, M7 bắt được |
| `bao-cao-coverage.md` | Báo cáo coverage nộp bài (kết quả thật) |

Chạy từ gốc lab:

```bash
npx vitest run articles/mc-dc-testing --reporter=verbose                       # 33 passed

# Chỉ 10 ca MC/DC (+ kiểm thiết kế): 98,05% stmt / 82,75% branch
npx vitest run articles/mc-dc-testing/mcdc-voucher.test.ts --coverage.enabled \
  --coverage.include=src/domain/voucher.ts --coverage.include=src/app/quote.ts \
  --coverage.reporter=text --coverage.reportsDirectory=reports/coverage-mc-dc-testing

# Cả bộ: 100% stmt / 100% branch
npx vitest run articles/mc-dc-testing --coverage.enabled \
  --coverage.include=src/domain/voucher.ts --coverage.include=src/app/quote.ts \
  --coverage.reporter=text --coverage.reportsDirectory=reports/coverage-mc-dc-testing

# Bản lỗi với bộ decision coverage: 100% stmt / 100% branch mà lỗi vẫn lọt
npx vitest run articles/mc-dc-testing/bat-loi-and-or.test.ts -t "decision coverage" --coverage.enabled \
  --coverage.include=articles/mc-dc-testing/buggy-eligible.ts --coverage.reporter=text \
  --coverage.reportsDirectory=reports/coverage-mc-dc-testing
```

`reportsDirectory` riêng để không ghi đè báo cáo coverage chung của lab.
