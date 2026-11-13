# SYS-10 — Báo cáo Security Test – Pen test lab

Ngày 13/11/2026. Windows, Node v24.15.0, npm 11.12.1, Vitest 3.2.7; dependency theo lockfile. Working tree sau SYS-09 `b3f9306` với nội dung SYS-10 mới.

| Kiểm tra | Kết quả |
|---|---|
| `npm test -- articles/system-testing-security-pentest` tại lab | PASS: 1 file, 11 executions, 1,19 giây; test time 114 ms |
| `npm run typecheck` tại lab | PASS, exit 0 |

SEC-01 có ba dataset; tổng chín case ID. Control checks: danh tính thiếu/sai 401, sai role 403; C002 đọc/huỷ trip C001 bị 404 không đổi chuyến; quote C001 không bị C002 tiêu; field nhạy cảm client gửi không ghi đè giá/chủ/trạng thái; malformed/oversized body bị chặn; lỗi inject không lộ marker giả/stack; test reset không mount mặc định.

**Known limitation LAB-IDENTITY-01 được tái hiện:** đổi x-user từ C002 sang C001 cho phép đọc trip C001. SPEC R-API-03 đã xác định x-user là auth mô phỏng. SEC-09 pass nghĩa là hành vi yếu này được tái hiện đúng, không phải đã khắc phục. Không có thay đổi src hoặc cơ chế auth. Nếu triển khai nguyên cơ chế ra production, đây là blocker cần thay bằng nguồn actor đáng tin.

Phạm vi chỉ app local trong process, seed giả, kho memory, không domain/IP ngoài. Đây không phải pentest toàn diện hay báo cáo an toàn của website antblog. Chưa kiểm JWT/session/TLS, XSS, SQL injection, dependency vulnerabilities, mạng/hạ tầng, rate-limit abuse hoặc mọi role/action. Không tính severity/CVSS cho deployment chưa đánh giá.

Lỗi nội bộ dùng marker giả, không secret thật; không kiểm redaction log. Test endpoint chỉ được kiểm ở createApp mặc định, không xác minh deployment env. Không chạy lại browser/build/regression không liên quan vì chỉ thêm bài/test độc lập. Chưa push hoặc xuất bản CMS.

Kết luận: các control trong phạm vi đạt expected, cùng một giới hạn identity đã biết còn tồn tại và được chứng minh. Không được suy “11/11 xanh” thành hệ thống sẵn sàng production.
