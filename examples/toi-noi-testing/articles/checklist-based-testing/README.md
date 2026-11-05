# TEST-053 · Checklist-based Testing

Bài: `/articles/checklist-based-testing`.

- `checklist-smoke-v1.0.md` — checklist CL-SMOKE v1.0: 20 mục (11 read-do, 9 do-confirm; ưu tiên P1/P2/P3) cho 4 luồng đặt xe, bưu kiện, đồ ăn, huỷ có phí + 4 mục chung; tiêu chí vào của phiên; các mục bị loại và lý do (CTFL 4.4.3).
- `e2e/smoke-checklist.spec.ts` — chạy phiên checklist trên trình duyệt thật: mỗi bước mang mã SM-xx. Mục read-do có giá trị mong đợi ⇒ khẳng định cứng; mục do-confirm ⇒ chỉ ghi bằng chứng (chữ trên màn hình, trạng thái qua API), người đánh giá trong bài. Cuối phiên in nhật ký 20 dòng.
- `playwright.config.ts` — máy chủ lab riêng ở cổng **3153**, `NODE_ENV=test`, đồng hồ 09:00 giờ VN 04/11/2026, tắt giới hạn tần suất, múi giờ trình duyệt `Asia/Ho_Chi_Minh` (SM-18 mở thêm một phiên `Asia/Tokyo`).

```bash
npx playwright test -c articles/checklist-based-testing/playwright.config.ts
```

Kết quả mong đợi: `5 passed` và nhật ký SM-01…SM-20. Các quan sát dẫn tới «Không đạt» trong bài (không làm test đỏ, vì là mục do-confirm): SM-05 trang khách không hiện trạng thái thanh toán; SM-16 trang tài xế không biết chuyến bị huỷ; SM-17 thông báo lộ `"arrive"`/`"cancelled"`; SM-18 hạn báo giá «09:05:xx» (giờ VN) và «11:05:xx» (Tokyo) không ghi múi giờ; SM-19 danh sách tài xế ghi «7.4 km» và mã `ride`. Cần Google Chrome cài sẵn.
