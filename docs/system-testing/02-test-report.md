# SYS-02 — Báo cáo Sanity Test

Ngày 13/11/2026. Windows, Node v24.15.0, npm 11.12.1, Vitest 3.2.7; dependency theo lockfile của lab. Chạy trên working tree sau commit bài 01 `8239123`, có nội dung SYS-02 mới.

| Kiểm tra | Kết quả thực đo |
|---|---|
| `npm test -- articles/system-testing-sanity-test articles/system-testing-smoke-test` | PASS, 2 file, 11 tests, 4,01 giây |
| Sanity mới | 8 tests pass, 160 ms |
| Smoke bài 01 | 3 tests pass, 2.178 ms |
| `npm run typecheck` tại lab | PASS, exit 0 |

Sanity gồm sáu ID, SN-04 chạy ba dataset. Fixture thiếu trim khác oracle như dự kiến; hàm hiện tại trả mã chuẩn. API trả đúng discount/total cho mã hợp lệ, giữ hành vi mã trống, báo lỗi mã không tồn tại, lưu mã chuẩn khi đặt và chặn lần dùng tiếp theo.

Đây là tình huống lỗi giả lập, không phải bản sửa bug sản phẩm. Không chạy một build API lỗi rồi sửa: chỉ so fixture lỗi tại cấp hàm với oracle và kiểm API hiện tại. Không thay `src/` dùng chung.

Giới hạn: memory repository, clock/weather/payment giả; chưa kiểm browser, PostgreSQL, concurrency, Unicode mở rộng hoặc property testing. Smoke dùng PGlite được chạy kèm để kiểm lại bài trước. Không chạy lại build web/admin vì thay đổi chỉ ở tài liệu và lab độc lập. Không xuất bản CMS.

Kết luận: đạt phạm vi sanity của tình huống chuẩn hóa voucher; tiếp tục thiết kế Functional Test. Không suy ra release production đạt mọi yêu cầu.
