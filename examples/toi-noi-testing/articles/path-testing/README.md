# TEST-048 · Path Testing

Bài: `/articles/path-testing`. Tính cyclomatic complexity của hàm chuyển trạng thái (`nextStatus` trong
`src/domain/trip-state.ts`) và của `acceptTrip` (`src/app/trips.ts`, chuyển `searching → accepted`), chọn tập
basis path theo phương pháp đường cơ sở của McCabe, mỗi path một test.

| Tệp | Nội dung |
|---|---|
| `eslint.config.js` | ESLint 9 + parser typescript-eslint, chỉ luật `complexity` |
| `cfg.ts` | CFG dựng tay của `acceptTrip` (23 nút, 31 cạnh), V(G), đi đường theo lựa chọn, vectơ cạnh, hạng, đếm đường |
| `basis-paths.test.ts` | P1–P2 (`nextStatus`), kiểm CFG (V(G) = 10, hạng 10, 16 đường), B0–B9 chạy trên `acceptTrip` thật |

Chạy từ gốc lab:

```bash
npx eslint -c articles/path-testing/eslint.config.js --rule 'complexity: ["warn", 1]' src/domain/trip-state.ts src/app/trips.ts
#   acceptTrip 10 · createTrip 10 · expireStaleSearches 3 · nextStatus 2 … (14 warnings — mong đợi)
npx vitest run articles/path-testing --reporter=verbose        # 17 passed
npx vitest run articles/path-testing --coverage.enabled --coverage.include=src/app/trips.ts \
  --coverage.reporter=json --coverage.reportsDirectory=reports/coverage-path-testing
#   mọi khối nhánh dòng 154–184 (acceptTrip) có lượt chạy > 0
```

B3 dựng tay dữ liệu vi phạm bất biến; B7, B9 dùng stub (`vi.fn`) vì đường đó không xảy ra với kho thật
trong một tiến trình; B6, B8 là «đồng thời» ở mức xen kẽ `await` của kho trong bộ nhớ, không phải đồng thời thật.
