# TEST-029 · Static Analysis: phân tích code tĩnh

Bài: `/articles/static-analysis-phan-tich-code-tinh`.

- `voucher-smelly.mts` — bản CỐ TÌNH có code smell: gộp R-VOUCHER-03…07 vào một hàm `checkVoucher`, gài 4 defect thật và vài cảnh báo vô hại. Đuôi `.mts` để `npm run typecheck` của cả lab không quét tệp (nó cố ý lỗi dưới `--strict`). Không sửa `src/`.
- `tsconfig.json` — kế thừa `tsconfig.json` của lab (strict), chỉ gồm tệp smelly + `src/domain/voucher.ts`, `src/app/quote.ts`, `src/domain/cancel.ts`, `src/domain/fare.ts` (hai tệp cuối cho bài tập 2).
- `eslint.config.js` — ESLint 9 flat config + typescript-eslint (có thông tin kiểu), luật `complexity` (ngưỡng 10), `no-unreachable`, `no-useless-assignment`, `no-unused-vars`, `no-unnecessary-condition`, `no-console`, `restrict-template-expressions` (cố tình chặt).
- `xac-nhan-canh-bao.test.ts` — 8 ca vitest xác nhận bằng test động cảnh báo nào là defect thật (ca trên bản smelly dùng `it.fails`).

Chạy từ gốc lab:

```bash
npx tsc -p articles/static-analysis-phan-tich-code-tinh --strict false        # 0 lỗi
npx tsc -p articles/static-analysis-phan-tich-code-tinh --strict              # 2 lỗi: TS7006, TS2454
npx tsc -p articles/static-analysis-phan-tich-code-tinh --strict --noUnusedLocals --noUnusedParameters --allowUnreachableCode false   # 4 lỗi
npx eslint -c articles/static-analysis-phan-tich-code-tinh/eslint.config.js src/domain/voucher.ts articles/static-analysis-phan-tich-code-tinh/voucher-smelly.mts   # 9 problems (7 errors, 2 warnings)
npx eslint -c articles/static-analysis-phan-tich-code-tinh/eslint.config.js --rule 'complexity: ["warn", 1]' src/domain/voucher.ts src/app/quote.ts   # in độ phức tạp từng hàm
npx vitest run articles/static-analysis-phan-tich-code-tinh                   # 8 passed
```

Lệnh ESLint và `tsc --strict` trên tệp smelly kết thúc với mã thoát khác 0 — đó là kết quả mong đợi.
