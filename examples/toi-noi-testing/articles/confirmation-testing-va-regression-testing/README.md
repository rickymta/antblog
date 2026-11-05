# TEST-019 — Confirmation Testing và Regression Testing

Bài: `/articles/confirmation-testing-va-regression-testing`.

Defect giả định: hàm làm tròn 0,1 half-up dùng chung cho quãng đường (SPEC R-FARE-02) và khối lượng (R-PARCEL-01) được viết bằng `toFixed(1)` ⇒ 7,35 km thành 7,3 km. Ba bản build:

| Tệp | Bản build | `roundToTenth` |
|---|---|---|
| `build-rc1.test.ts` | rc1 — có defect | `Number(v.toFixed(1))` (`rounding-variants.ts`) |
| `build-rc2.test.ts` | rc2 — sửa lần 1, sai | `Math.ceil(v * 10) / 10` (`rounding-variants.ts`) |
| `build-rc3.test.ts` | rc3 — sửa lần 2 | bản thật `src/domain/money.ts` (không mock) |

`define-suite.ts` chứa cùng một bộ test cho cả ba bản: 6 confirmation test (CF-01…04 chạy lại, CF-05…06 thêm mới) và 9 regression test (RG-01…09) chọn theo impact analysis — mọi đường đi qua `roundToTenth`, ở cấp component và cấp hệ thống (API qua supertest). Bản build được thay bằng `vi.mock('../../src/domain/money')` — không sửa `src/`.

```bash
npx vitest run articles/confirmation-testing-va-regression-testing               # 45 passed (12 ca đánh dấu it.fails)
SHOW_RED=1 npx vitest run articles/confirmation-testing-va-regression-testing    # thấy failure thật: 12 failed, 33 passed
npx vitest related src/domain/money.ts --run                                      # chọn test theo đồ thị import (công cụ impact analysis)
```

PowerShell: `$env:SHOW_RED='1'; npx vitest run articles/confirmation-testing-va-regression-testing` (xong thì `Remove-Item Env:SHOW_RED`).

Kết quả mong đợi khi `SHOW_RED=1`:

- rc1: CF-01…06 đỏ (7,35 ⇒ 7,3; 46.000 ⇒ 45.000; 100,05 được chấp nhận; 5,05 kg ⇒ 5,0 kg phụ phí 0; quét 996 giá trị lệch 398), RG-01…09 xanh.
- rc2: CF-01…06 xanh; RG-01, 03, 05, 06, 08, 09 đỏ (0,44 km được chấp nhận; 100,04 km bị từ chối; 0,04 kg được chấp nhận; 30,04 kg bị từ chối; phí giao 2,01 km 22.000 thay vì 21.000).
- rc3: 15/15 xanh.
