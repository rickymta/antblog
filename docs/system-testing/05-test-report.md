# SYS-05 — Báo cáo System Test

Ngày 13/11/2026. Windows, Node v24.15.0, npm 11.12.1, Chrome 157.0.8103.51 headless; dependency theo lockfile lab. Working tree sau commit SYS-04 `ad2123b` với bài SYS-05 mới.

| Kiểm tra | Kết quả thực đo |
|---|---|
| `npx playwright test --config system-testing.config.ts` tại lab, lượt cuối | PASS: 3 tests, 11,4 giây, không retry |
| SY-01 hoàn tất chuyến hai vai trò | PASS, 5,3 giây |
| SY-02 huỷ khi searching | PASS, 1,1 giây |
| SY-03 sửa dữ liệu/vô hiệu quote/lỗi UI | PASS, 695 ms |
| `npm run typecheck` tại lab | PASS, exit 0 |
| `npm run typecheck` tại root (web/admin/shared/ui) | PASS, exit 0 |

Lượt đầu cũng pass 3/3 trong 11,9 giây; chạy lại sau khi bỏ URL tài xế hardcode, lấy origin từ trang khách để tương thích config mặc định. Không có lỗi sản phẩm được phát hiện trong ba case. Có warning NO_COLOR/FORCE_COLOR của terminal, không làm test fail.

Server riêng cổng 3115, PGlite, không reuse server, một worker, rate limit 5/60 giây, test API bật để reset/đặt giờ. Mọi thao tác đặt/nhận/hoàn tất/huỷ đều qua UI; API đọc bổ trợ kiểm kết quả và quyền đọc C002. Payment/weather/danh tính là mô phỏng; chưa kiểm provider thật, JWT, nhiều browser, staging, artifact build hoặc khả năng chịu tải.

SY-01 quan sát hai UI completed và API completed/paid/fare=35.000, C002 bị 404; SY-02 phí 0 và paymentStatus none; SY-03 quote cũ ẩn, nút đặt disabled, response 422 DISTANCE_OUT_OF_RANGE và UI hiện lỗi. Không suy ra ba case phủ toàn bộ system requirements hoặc mọi field invalidation.

Không rebuild web/admin, không sửa src dùng chung, chưa xuất bản CMS. Trace/screenshot được cấu hình chỉ lưu khi lỗi nên lượt pass không sinh bằng chứng hình ảnh; kết quả trên dựa vào assertions và output runner. Chạy lại từ thư mục lab bằng lệnh trong bảng, cần Chrome cài sẵn và `npm ci` nếu thiếu dependency.
