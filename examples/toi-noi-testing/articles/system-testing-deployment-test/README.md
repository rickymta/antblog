# SYS-16 — Deployment Test

[Bài học](../../../../docs/system-testing/16-deployment-test.md) · [Báo cáo](../../../../docs/system-testing/16-test-report.md)

Từ `examples/toi-noi-testing`:

```powershell
npm ci
npm test -- articles/system-testing-deployment-test
npm run typecheck
```

`deployment.test.ts` (DP-01…05) khởi động `src/server.ts` thành tiến trình riêng bằng `node --import tsx`, trên cổng trống do hệ điều hành cấp, với cấu hình giống triển khai: `NODE_ENV=production`, `TOINOI_DB=pglite`, `TOINOI_START_AT=2026-11-04T09:00:00+07:00`. Test chờ `/api/health` trả 200 trong hạn chót 15 giây (hỏi lại mỗi 100 ms, không sleep cố định), rồi kiểm qua HTTP thật: API kiểm thử phải 404, log khởi động ghi đúng cấu hình, post-deploy smoke báo giá → đặt → hoàn tất → đọc lại. DP-05 cố ý khởi động một bản cấu hình sai (`TOINOI_TEST_API=1`) để chứng minh phép kiểm của DP-02 bắt được nó. Test xoá `NODE_ENV=test` mà Vitest tự đặt trước khi khởi động tiến trình con, và dừng mọi tiến trình con trong `afterEach`.

`migration-compat.test.ts` (MG-01…03) dùng mã hiện tại trong `src/` làm "bản cũ" sau rollback và áp migration lên PGlite riêng cho từng test: expand an toàn (cột nullable), expand sai (NOT NULL không default) và contract quá sớm (đổi tên cột).

Giới hạn: một tiến trình trên một máy, không có build artifact, container, load balancer, TLS, secret manager hay rolling/canary thật. Lab không có endpoint version; bằng chứng cấu hình lấy từ log khởi động. PGlite một kết nối: không mô phỏng migration chạy cùng traffic, khoá bảng hay thời gian migration trên dữ liệu lớn. Trên Windows `kill()` dừng cưỡng bức, nên không kiểm graceful shutdown.
