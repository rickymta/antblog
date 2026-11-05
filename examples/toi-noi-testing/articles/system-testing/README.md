# TEST-014 · System Testing

Bài: `/articles/system-testing`.

- `e2e/dat-o-to.spec.ts` — 2 test Playwright chạy trên **toàn bộ hệ thống lab** (máy chủ thật, trang khách `/`, trang tài xế `/driver.html`, hai phiên trình duyệt riêng): TC-SYS-01…05 (báo giá ô tô 107.000 đ, bấm đúp «Đặt chuyến» chỉ tạo một chuyến, tài xế xe máy D101 không thấy chuyến ô tô, D103 nhận → tới đón → đã đón → chở → hoàn thành, trang khách tự cập nhật trong ≤ 3 giây — có in thời gian đo) và TC-SYS-06 (bấm «Hoàn thành» sai thứ tự bị từ chối, trạng thái giữ nguyên).
- `playwright.config.ts` — cấu hình riêng: máy chủ lab ở cổng **3114** (không đụng cổng 3100), `NODE_ENV=test`, đồng hồ 09:00 giờ VN 04/11/2026, tắt giới hạn tần suất. Spec vẫn chạy được bằng cấu hình gốc (`npm run test:e2e`).

```bash
npx playwright test -c articles/system-testing/playwright.config.ts
```

Kết quả mong đợi: `2 passed`. Cần Google Chrome cài sẵn (lab không tải trình duyệt). Thời gian lan truyền đo trên máy dev chỉ minh hoạ phương pháp (trang khách hỏi lại mỗi 2 giây nên số đo quanh 1,9–2,5 giây).
