# SYS-05 — System Test

[Bài đọc](../../../../docs/system-testing/05-system-test.md) · [Báo cáo](../../../../docs/system-testing/05-test-report.md).

Từ `examples/toi-noi-testing`, dùng Node >=22.12, Google Chrome cài sẵn; `npm ci` nếu thiếu dependency:

```sh
npx playwright test --config system-testing.config.ts
npm run typecheck
```

Expected: 3 test pass, không retry. Config riêng khởi động server trên 3115, kho PGlite, không tái sử dụng server có sẵn; nếu cổng bận thì dừng, không reset server của người khác. Một worker, reset dữ liệu và giờ trước mỗi case. Trace/screenshot khi lỗi trong `reports/system-testing` (không commit).

Đây là lab cục bộ có API điều khiển test; không trỏ cấu hình này vào môi trường dùng chung. Ví/thời tiết/danh tính là mô phỏng. Config mặc định `npm run test:e2e` cũng phát hiện file này nhưng có thể dùng memory và server tái sử dụng: chỉ config riêng ở trên tạo môi trường được báo cáo cho SYS-05.
