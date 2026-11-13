# SYS-03 — Báo cáo Functional Test

Ngày 13/11/2026. Môi trường Windows, Node v24.15.0, npm 11.12.1, Vitest 3.2.7; dependency theo lockfile lab. Chạy trên working tree sau commit SYS-02 `6b402db` với nội dung SYS-03 mới.

| Phép kiểm | Kết quả thực đo |
|---|---|
| `npm test -- articles/system-testing-functional-test articles/system-testing-sanity-test articles/system-testing-smoke-test` tại lab | PASS: 3 file, 35 tests, tổng 8,15 giây |
| Functional mới | 24 tests pass, 379 ms |
| Sanity bài 02 | 8 tests pass, 276 ms |
| Smoke bài 01 | 3 tests pass, 1.448 ms |
| `npm run typecheck` tại lab | PASS, exit 0 |

Phạm vi: tám dataset quãng đường, tám dòng quyết định cao điểm/mưa/voucher, bốn tình huống validation, sàn giá, trần giảm, snapshot giá và chuyển trạng thái bị từ chối không làm đổi chuyến. Oracle tiền từ SPEC được viết tường minh trong bảng bài đọc và test.

Kết luận: hành vi trong phạm vi chọn đạt expected; không phát hiện defect sản phẩm trong lượt này. Không sửa mã sản phẩm dùng chung. 8/8 dòng bảng quyết định là coverage của mô hình ba điều kiện cho bike 5 km, không phải 100% chức năng tính giá.

Giới hạn: suite functional dùng memory repository, clock/weather/payment giả và HTTP trong tiến trình. Chưa kiểm UI, DB nhiều kết nối, tích hợp thanh toán thật, toàn bộ dịch vụ food/parcel/car, concurrency, biên giờ/ngày, property hoặc mutation testing. Không chạy lại build web/admin vì chỉ bổ sung tài liệu và lab độc lập. Chưa xuất bản CMS.

Chạy lại bằng lệnh trong bảng; nếu chưa cài dependency, chạy `npm ci` tại `examples/toi-noi-testing` trước. Mỗi case tạo instance riêng nên không cần cleanup dữ liệu trên môi trường ngoài.
