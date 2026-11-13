# SYS-04 — Báo cáo Integration Test

Ngày 13/11/2026. Windows, Node v24.15.0, npm 11.12.1, Vitest 3.2.7; dependency theo lockfile lab. Chạy trên working tree sau commit SYS-03 `004719c` với bài/lab SYS-04 mới.

| Kiểm tra | Kết quả thực đo |
|---|---|
| `npm test -- articles/system-testing-integration-test articles/system-testing-functional-test articles/system-testing-sanity-test articles/system-testing-smoke-test` tại lab | PASS: 4 file, 42 tests, 3,95 giây |
| Integration mới | 7 tests pass, 2.628 ms gồm setup/reset PGlite |
| Functional / Sanity / Smoke trước đó | 24 / 8 / 3 tests pass |
| `npm run typecheck` tại lab | PASS, exit 0 |

Quan sát: replay cùng key/body trả response cũ, SQL có một chuyến và hai events khởi tạo; foreign key chặn event mồ côi với code 23503. Năm dataset thanh toán cho số calls 1/2/1/3/3 tương ứng none/lost_response/decline/timeout/error. Lost response thu đúng một lần 35.000 dù gọi hai lần; các lỗi vĩnh viễn trong case trả completed/failed và tài xế online theo SPEC. DB và response khớp nhau, mỗi chuyến chỉ có một completed event.

Timeout cấu hình 50 ms thay vì mặc định SPEC 3 giây để kiểm logic retry; không đo SLA hoặc khẳng định kiểm ngưỡng 3 giây. API/use cases/repository/schema/PGlite dùng implementation thật, payment/weather/clock là fake. Không sửa `src/` dùng chung.

Chưa kiểm provider sandbox, HTTP payment adapter, lỗi ghi DB sau thu tiền, transaction nhiều kết nối, crash/restart, migration, broker hoặc browser. Chưa chạy toàn bộ regression của mọi series. Không rebuild web/admin vì chỉ thay tài liệu và lab độc lập. Chưa xuất bản CMS.

Kết luận: đạt phạm vi tích hợp đã chọn; không phát hiện defect sản phẩm trong lượt chạy. Không suy ra tính nguyên tử SQL–ví hoặc tính tương thích với provider thật. Chạy lại bằng lệnh trên sau `npm ci` tại lab nếu chưa có dependency; reset trước mỗi case và đóng DB sau suite.
