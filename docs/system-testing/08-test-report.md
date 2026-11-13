# SYS-08 — Báo cáo Load Test

Ngày 13/11/2026, kết thúc 21:09:09 giờ Việt Nam (14:09:09 UTC). Working tree sau SYS-07 `3015185`. Windows 10.0.26300, Node 24.15.0, i7-10700K, 16 logical CPUs, RAM 34.245.918.720 bytes. Không đo CPU nền của ứng dụng khác.

Lệnh: `npx tsx articles/system-testing-load-test/load.ts` tại lab, exit 0. Warm-up 20 rồi reset; 300 slots đo, 300 gửi, 0 drops, 0 lỗi. Cả ba gate giả định đạt. Typecheck lab đạt, exit 0. Kiểm JSON độc lập xác nhận planned=sent+dropped, số mẫu/drop slots, failure count và p95 khớp summary.

| Bước | Lịch | Sent / drop / lỗi | p95 ms | p99 ms | Valid RPS gồm drain | Lag tối đa ms |
|---|---|---|---:|---:|---:|---:|
| low | 10/s × 3s | 30 / 0 / 0 | 11,229 | 13,231 | 9,992 | 15,516 |
| target | 30/s × 8s | 240 / 0 / 0 | 6,309 | 10,366 | 29,952 | 16,483 |
| return-low | 10/s × 3s | 30 / 0 / 0 | 10,087 | 10,208 | 9,970 | 15,242 |

Mọi request hoàn tất trong cửa sổ của bước tương ứng; peak in-flight=1 cả ba bước vì latency nhỏ hơn khoảng slot. Dù scheduler là open-model, lượt này chưa tạo overlap lớn hoặc saturation. p99 của bước chỉ có 30 mẫu chính là max theo nearest-rank, không phải tail estimate ổn định.

Evidence: `examples/toi-noi-testing/articles/system-testing-load-test/load-observed.json`, gồm raw samples và cấu hình. Lần chạy mới ghi vào reports/load và không ghi đè evidence đã commit.

Gate là zero drops/errors và p95<500 ms mỗi bước, chỉ phục vụ bài học. Giới hạn 20 in-flight, timeout 2s, bỏ slot nếu lag>100ms, dừng gửi mới khi đã có 10 failures. Các nhánh drop/stop không được kích hoạt ở lượt này; chưa khẳng định đã kiểm nghiệm hành vi quá tải. Bài Stress tiếp theo cần thí nghiệm riêng.

Client/server/PGlite cùng process, một DB connection, profile 14 giây theo lịch có drain giữa bước, dữ liệu nhỏ và một customer. Không có CPU/DB telemetry tách biệt. Không suy ra tải production, khả năng duy trì nhiều giờ hay recovery sau lỗi. Không so chênh lệch với SYS-07 như một phép đo riêng tác động DB vì cả workload và storage đã đổi.

Không sửa src dùng chung, không chạy lại browser/regression vì chỉ thêm script/tài liệu riêng, không rebuild web/admin, chưa xuất bản CMS. Kết luận: đạt tải giả định local với oracle nghiệp vụ đúng; chưa xác định capacity hoặc bottleneck hệ thống.
