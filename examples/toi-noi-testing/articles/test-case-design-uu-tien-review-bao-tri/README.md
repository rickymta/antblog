# TCD-09 — Ưu tiên, review và bảo trì bộ test case

Bài đọc: `api/seed/content/test-case-design-uu-tien-review-bao-tri.md` (/articles/test-case-design-uu-tien-review-bao-tri).

```sh
npx vitest run articles/test-case-design-uu-tien-review-bao-tri --reporter=verbose
```

| Tệp | Nội dung |
|---|---|
| `bo-test-case.ts` | 19 test condition, 8 rủi ro, ưu tiên yêu cầu (giả định bài tập), 15 test case R-PAY + R-IDEM viết thành dữ liệu |
| `uu-tien.ts` | Risk-based, requirements-based, coverage-based (tổng và additional), sửa thứ tự theo phụ thuộc (CTFL 5.1.5) |
| `uu-tien.test.ts` | 9 test: khẳng định mọi thứ tự và số liệu in trong bài |
| `ban-nhap.ts`, `review-tu-dong.ts`, `review.test.ts` | Bản nháp 9 ca cài sẵn anti-pattern, bản sửa 8 ca, phần máy kiểm được của checklist CL-TC — 4 test |
| `thuc-thi.test.ts` | 15 test case chạy thật qua API theo lịch risk-based + 1 test kiểm đủ cài đặt — 16 test |
| `bao-tri.test.ts` | R-PAY-04 giả định đổi 2 → 3 lần thử lại: phân tích ảnh hưởng và chạy lại — 11 test |

Kết quả khi viết bài (Node 22, Vitest 3.2.7): **4 file, 40 test passed**.

- `TC-IDEM-006` chạy bằng `it.fails`: kết quả mong đợi theo SPEC (gửi lại sau 500 ⇒ 201) đang đỏ vì defect DR-050-01 đã ghi ở bài TEST-050 (thực tế 409 `QUOTE_ALREADY_USED`). `SHOW_RED=1` để thấy đỏ thật: 1 failed, 15 passed trong `thuc-thi.test.ts`.
- Hết giờ cổng thanh toán rút còn 20 ms; SPEC là 3 giây.
- Không sửa `src/`.
