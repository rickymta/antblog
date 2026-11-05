# TEST-049 — Loop Testing

Bài: `/articles/loop-testing`.

Loop testing (Beizer) áp vào ba vòng lặp của lab, không sửa `src/`:

| Tệp | Vòng được kiểm | Yêu cầu |
|---|---|---|
| `vong-cong-tien-mon.test.ts` | vòng cộng tiền món `itemsSubtotal` (`src/domain/food.ts`) với 0, 1, 2, 5, 19, 20, 21 dòng — ở tầng domain, qua `POST /api/quotes`, và với giá trần trên kho PGlite | R-FOOD-02…04, R-API-02 |
| `vong-thu-lai-va-long-nhau.test.ts` | vòng thử lại `chargeWithRetry` (`src/app/payment.ts`); vòng lồng nhau trong `canonicalJson` (`src/api/idempotency.ts`) | R-PAY-04, R-IDEM-02 |
| `loop-variants.ts` | bốn bản cài đặt sai của vòng cộng tiền (L1…L4) để dựng ma trận phát hiện | — |
| `fixtures.ts` | dữ liệu dòng món + oracle dạng đóng (không dùng vòng lặp) | R-FOOD-02 |

```bash
npx vitest run articles/loop-testing                 # 35 passed (1 ca đánh dấu it.fails — defect tràn integer)
SHOW_RED=1 npx vitest run articles/loop-testing      # thấy failure thật: 1 failed, 34 passed
```

PowerShell: `$env:SHOW_RED='1'; npx vitest run articles/loop-testing` (xong thì `Remove-Item Env:SHOW_RED`).

Defect ghi nhận (ca `DEFECT · PostgreSQL: 9 dòng giá trần`): R-FOOD-02 cho phép tiền món tới 20 × 5.000.000 × 50 = 5.000.000.000 đ, nhưng cột `trips.items_subtotal`/`trips.fare` trong `src/infra/pglite/schema.sql` là `integer` (tối đa 2.147.483.647). Từ 9 dòng ở giá trần, báo giá trả 200 còn `POST /api/trips` trên PGlite trả 500 `INTERNAL_ERROR` (log: `value "2250000000" is out of range for type integer`). Kho trong bộ nhớ không bị.
