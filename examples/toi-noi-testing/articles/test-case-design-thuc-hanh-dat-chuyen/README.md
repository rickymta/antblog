# TCD-10 — Capstone: thiết kế trọn bộ test case «đặt chuyến từ báo giá»

Bài đọc: `api/seed/content/test-case-design-thuc-hanh-dat-chuyen.md` (/articles/test-case-design-thuc-hanh-dat-chuyen).

```sh
npx vitest run articles/test-case-design-thuc-hanh-dat-chuyen --reporter=verbose
```

| Tệp | Nội dung |
|---|---|
| `thiet-ke.ts` | 11 yêu cầu trong phạm vi, 26 test condition, 9 rủi ro, 31 test case logic (kỹ thuật, truy vết, số ca cụ thể, phụ thuộc), test data TD |
| `dat-chuyen.test.ts` | 47 test case cụ thể chạy qua Express thật bằng supertest, dùng `/api/test/reset`, `/api/test/clock`, `/api/test/weather`; thứ tự = lịch risk-based của TCD-09; ca cuối sinh RTM |
| `rtm.ts` | Sinh RTM xuôi/ngược và số coverage từ thiết kế + kết quả chạy |

Kết quả khi viết bài (Node 22, Vitest 3.2.7): **1 file, 48 test passed** = 47 test case cụ thể + 1 test sinh báo cáo.

- So với SPEC: 44 đạt, 3 không đạt — chạy bằng `it.fails`: `TC-BOOK-038` (DR-050-01), `TC-BOOK-061` (DR-050-02), `TC-BOOK-062` (DR-TCD10-01, phát hiện mới cùng nguyên nhân gốc với DR-050-02). `SHOW_RED=1` ⇒ 3 failed, 45 passed.
- Coverage: yêu cầu trong phạm vi 11/11, test condition 26/26, không có test case mồ côi.
- Báo cáo sinh ra: `reports/tcd-10/rtm.md`, `reports/tcd-10/rtm.json` (thư mục `reports/` không commit).
- Không sửa `src/`.
