# SYS-06 — Báo cáo Regression Test

Ngày 13/11/2026. Windows, Node v24.15.0, npm 11.12.1, Vitest 3.2.7; dependency theo lockfile lab. Working tree sau commit SYS-05 `ff8600d` với nội dung SYS-06 mới.

| Kiểm tra | Kết quả thực đo |
|---|---|
| `npm test -- articles/system-testing-smoke-test articles/system-testing-sanity-test articles/system-testing-functional-test articles/system-testing-integration-test articles/system-testing-regression-test` tại lab | PASS: 5 files, 50 tests, 4,36 giây |
| Regression mới | 8 tests pass, 112 ms |
| Smoke / Sanity / Functional / Integration | 3 / 8 / 24 / 7 tests pass |
| `npm run typecheck` tại lab | PASS, exit 0 |

Fault fixture sai vẫn trả 43.000 ở ca mưa/giờ thường, nhưng trả 54.000 ở ca mưa/cao điểm, khác oracle 52.000. Hàm sản phẩm hiện tại trả đúng. API cases ride/parcel/food đáp ứng oracle: 52.000 / 54.000 / 103.000 với breakdown được kiểm trong test.

Không sửa `src/`; fixture là giả lập một refactor sai, không phải bug sản phẩm đã phát hiện/sửa. Không chạy build lỗi đỏ rồi sửa xanh và không chạy Stryker, do đó không có mutation score. Chỉ xác nhận một fault cụ thể được phân biệt bởi oracle.

Phạm vi 50 tests là các bài SYS-01…04 và SYS-06, không phải toàn bộ test repository. SYS-05 browser không chạy lại vì không thay UI/config/shared source; báo cáo lần chạy trước nằm ở `05-test-report.md`. Chưa kiểm performance regression, provider thật, lựa chọn test tự động theo diff hoặc mọi consumer. Không rebuild web/admin, chưa xuất bản CMS.

Kết luận: phạm vi regression được chọn đạt expected, không phát hiện defect sản phẩm. Lệnh trên tái hiện lượt chạy; dùng `npm ci` tại lab nếu thiếu dependency. API tests mới tạo memory system riêng; các suite SQL tự quản lý PGlite.
