# TCD-07 · Test case từ user story và acceptance criteria

Bài: `/articles/test-case-design-user-story-acceptance-criteria`.

User story US-CANCEL-01 «Là khách, tôi muốn huỷ chuyến và biết trước mình mất bao nhiêu phí» (SPEC R-CANCEL-01…07, R-UI-01). Acceptance criteria viết ở hai dạng của CTFL 4.5.2 rồi chạy được như ATDD (CTFL 4.5.3). Không Cucumber, không thêm thư viện, không sửa `src/`.

| Tệp | Nội dung |
|---|---|
| `cau-chuyen.ts` | User story, 8 acceptance criteria dạng rule (AC-01…AC-08, AC-08 là giả định chưa có trong SPEC), bảng vào–ra 10 dòng TD-01…TD-10 và 5 «ví dụ điển hình» VD-1…VD-5 |
| `kich-ban.ts` | 15 kịch bản Given/When/Then SC-01…SC-15 viết thành dữ liệu có kiểu, kèm hàm `vietKichBan` in ra văn bản để review với BA |
| `chay-kich-ban.ts` | Step runner: mỗi loại bước Given/When/Then có đúng một hàm thực thi, gọi API thật bằng supertest trên hệ thống bộ nhớ, `FixedClock` đặt giờ từng bước |
| `buggy-cancel-fee.ts` | Bản chép hàm phí có defect cố ý ở hai phép so sánh biên (`<=` ở mốc 2 phút, `>=` ở mốc tài xế trễ) |
| `acceptance-criteria.test.ts` | 32 ca vitest + 1 todo |

```bash
npx vitest run articles/test-case-design-user-story-acceptance-criteria --reporter=verbose
```

Kết quả mong đợi: `32 passed | 1 todo (33)` —

- 4 ca kiểm chính bộ AC: mã R-… có trong SPEC; mỗi AC có căn cứ có kịch bản; ma trận AC → kịch bản; in Given/When/Then.
- 15 kịch bản chạy qua API, trong đó 5 kịch bản trên biên: SC-03 (1:59,999 sau khi nhận ⇒ 0 đ), SC-04 (đúng 2:00 ⇒ 10.000 đ), SC-08 (đúng nhận + ETA + 10 phút ⇒ vẫn thu), SC-09 (quá mốc 1 ms ⇒ 0 đ, `driver_late`), SC-15 (bấm lại sau mốc 2 phút cùng `Idempotency-Key` ⇒ nhận lại kết quả 0 đ của lần đầu).
- 1 todo: SC-16 cho AC-08 (xem trước phí) — chờ BA quyết vì SPEC chưa có.
- 10 ca bảng rule TD-01…TD-10 trên `customerCancellationFee`.
- 2 ca về defect: 5 ví dụ điển hình không bắt được bản lỗi; bảng có ví dụ trên biên bắt được ở TD-03, TD-04, TD-06.
- 1 ca minh hoạ mơ hồ «now»: màn hình tính lúc 09:02:59,500 ra 0 đ, máy chủ xử lý lúc 09:03:00,200 thu 10.000 đ.

Thêm một kịch bản: thêm một object vào `KICH_BAN` — không phải viết code bước mới, trừ khi cần loại bước mới.
