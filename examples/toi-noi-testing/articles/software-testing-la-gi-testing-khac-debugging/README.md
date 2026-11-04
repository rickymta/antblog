# TEST-001 — Software Testing là gì? Testing khác Debugging

Bài: `/articles/software-testing-la-gi-testing-khac-debugging`.

| Tệp | Vai trò |
|---|---|
| `buggy-fare.ts` | Bản chép `calculateFare` có một defect: bước 6 làm tròn XUỐNG thay vì LÊN (SPEC R-FARE-08). |
| `test-cases.ts` | 4 test case, kết quả mong đợi tính tay từ SPEC (R-FARE-01, R-FARE-07, R-FARE-08, R-FARE-09). |
| `testing.test.ts` | Việc của tester: chạy test case trên bản có lỗi, gây ra failure. |
| `debugging.test.ts` | Việc của developer: tái hiện, chẩn đoán; rồi confirmation/regression test trên bản đúng (`src/domain/fare.ts`). |

```bash
npx vitest run articles/software-testing-la-gi-testing-khac-debugging             # 11 passed (2 ca đánh dấu it.fails)
SHOW_RED=1 npx vitest run articles/software-testing-la-gi-testing-khac-debugging  # thấy failure thật: 2 failed, 9 passed
```

PowerShell: `$env:SHOW_RED='1'; npx vitest run articles/software-testing-la-gi-testing-khac-debugging` (chạy xong nhớ `Remove-Item Env:SHOW_RED`).

Kết quả mong đợi khi `SHOW_RED=1`: `TC-FARE-01` expected 52000, received 51000; `TC-FARE-02` expected 26000, received 25000; `TC-FARE-03`, `TC-FARE-04` đạt (defect không lộ ra ở đầu vào đã tròn nghìn).
