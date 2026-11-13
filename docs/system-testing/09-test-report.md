# SYS-09 — Báo cáo Stress Test

Ngày 13/11/2026, kết thúc 21:29:50 giờ Việt Nam (14:29:50 UTC). Working tree sau SYS-08 `e48cc2b`. Windows 10.0.26300, Node 24.15.0, i7-10700K, 16 logical CPUs, RAM 34.245.918.720 bytes.

Lệnh thực chạy từ root: `npx --prefix examples/toi-noi-testing tsx examples/toi-noi-testing/articles/system-testing-stress-test/stress.ts`, exit 0. JSON ban đầu ở root `reports/stress`; đã chuyển đúng file đó vào evidence của bài. Lệnh README chạy từ thư mục lab là cách tái hiện thông thường. `npm run typecheck` tại lab đạt. Kiểm JSON độc lập xác nhận counts, error count và p95 khớp raw samples.

| Bước | Target | Sent / dropped / errors | p95 ms | p99 ms | Peak in-flight | Valid RPS gồm drain |
|---|---:|---|---:|---:|---:|---:|
| baseline | 30/s × 3s | 90 / 0 / 0 | 10,489 | 11,357 | 1 | 29,989 |
| elevated | 150/s × 4s | 600 / 0 / 0 | 14,057 | 22,390 | 6 | 149,763 |
| high | 600/s × 4s | 2400 / 0 / 0 | 15,155 | 22,803 | 24 | 598,342 |
| return-low | 30/s × 3s | 90 / 0 / 0 | 6,102 | 8,674 | 1 | 29,930 |

Tổng 3.180 slots đo đều được gửi; thêm 20 warm-up, một quote probe cuối và một health request. Tại high có 2.393 request hoàn tất trong scheduled window, 7 hoàn tất ở phần đuôi; elapsed gồm drain khoảng 4.011,08 ms. Không gọi sentPerScheduledSecond=600 là mọi response đều hoàn tất trong cửa sổ 4 giây.

Quote probe đúng oracle, HTTP 200, latency 2,83 ms; health 200 với status ok. Cả bốn bước đều trong reference envelope. Latency và in-flight tăng khi tăng tải, nhưng chưa có lỗi/drop hay vi phạm ngưỡng local; chưa tìm thấy breakpoint. Return-low đạt chỉ chứng minh bước/probe đã chọn sau drain hoạt động, không chứng minh recovery đầy đủ hay RTO.

Evidence: `examples/toi-noi-testing/articles/system-testing-stress-test/stress-observed.json`. Nhánh cap-drop, scheduler-late và failure-stop không được kích hoạt; guard hiện diện trong code nhưng chưa có thực nghiệm nhánh đó trong lượt này. Không có lỗi sản phẩm được phát hiện.

Client/server/PGlite cùng process và một DB connection, dữ liệu nhỏ, profile ngắn, không telemetry CPU/DB riêng. Không suy ra capacity production hoặc độ ổn định dài hạn. Không chạy SUT tới crash, không test restart/restore, không sửa src dùng chung. Không rebuild web/admin hoặc chạy lại browser/regression không liên quan; chưa xuất bản CMS.
