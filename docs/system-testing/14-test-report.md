# SYS-14 — Báo cáo Recovery Test

Ngày chạy: 13/11/2026, Windows. Mã test và script nằm trong cùng commit với báo cáo. Không thay đổi mã nghiệp vụ.

## 1. API và lỗi dependency

Từ `examples/toi-noi-testing`:

```powershell
npm test -- articles/system-testing-recovery-test
npm run typecheck
```

Node.js 24.15.0, Vitest 3.2.7, PGlite 0.3.16; FixedClock, dữ liệu seed reset từng test; ví giả, timeout 30 ms, tối đa hai retry.

- **5/5 test pass**, 1 file; tổng 3,35 giây, phần test 2,17 giây.
- **Typecheck pass**, exit code 0.
- RC-01 (3 mode): lỗi một lần rồi thành công; hai lần gọi cùng khóa, một khoản thu 35.000, DB paid, một completed event, tài xế online.
- RC-02: outage timeout hết ba attempts; sau gỡ lỗi, chuyến mới paid, chuyến cũ failed; complete lại chuyến cũ trả 409, không gọi ví thêm.
- RC-03: mất cả ba phản hồi; ví đã thu 35.000 một lần, DB failed; gỡ lỗi không tự reconcile. Đây là giới hạn nghiệp vụ được ghi nhận, không phải chứng nhận đã phục hồi khoản thu.

## 2. Logical backup/restore PostgreSQL

Từ root repository:

```powershell
& ./examples/toi-noi-testing/articles/system-testing-recovery-test/restore-drill.ps1
```

PostgreSQL **18.3**, cluster mới trên 127.0.0.1:55484, dữ liệu tổng hợp bảng recovery_probe, backup custom format. Kết quả lần chạy hoàn chỉnh:

```json
{
  "postgres": "pg_dump (PostgreSQL) 18.3",
  "recoveredMarker": 1,
  "missingPostBackupMarker": 2,
  "newWriteMarker": 3,
  "restoreAndValidationMs": 774,
  "applicationRtoMeasured": false,
  "rpoTimeMeasured": false
}
```

Nguồn có hai marker trước DELETE. Database đích khôi phục đúng `1:35000`, không có marker 2 được ghi sau backup. Insert marker 3 thành công, count cuối bằng 2. Script exit code 0; log xác nhận database system is shut down.

Evidence local được giữ tại `C:\Users\PC\AppData\Local\Temp\toinoi-sys14-0b4827dff8524e359d3124716da0e629`: probe.dump, result.json, postgres.log, log khởi động và data. Thư mục này không thuộc repository và có thể bị hệ điều hành dọn; chạy lại script để tạo evidence mới.

Hai lần thử đầu gặp vấn đề chờ tiến trình nền trên Windows, chưa hoàn thành restore. Đã dừng các cluster thử đó, sửa khởi động thành Start-Process ẩn với output ra file và WaitForExit chỉ chờ pg_ctl. Kết quả trên thuộc lần chạy sau sửa, không lấy số liệu từ lần lỗi.

## 3. Phạm vi kết luận

Đạt tiêu chí bài tập: retry/phục hồi việc mới được kiểm chứng; trạng thái khoản thu mơ hồ được thể hiện; logical restore khôi phục snapshot và chấp nhận ghi mới; cluster dừng sau chạy.

774 ms chỉ bao gồm tạo DB đích, restore và xác minh, không phải RTO nghiệp vụ. Chưa đo RPO theo thời gian; chỉ xác nhận một marker sau backup bị thiếu. Chưa restore schema Tới Nơi, chưa chạy PITR, crash-restart, failover, backup hỏng, mất vùng hoặc ledger thật. Backup và data ở cùng máy nên không chứng minh khả năng chịu mất storage.

Không chạy lại toàn bộ suite/build website: thay đổi gồm tài liệu, suite độc lập và script fixture. Bài tiếp theo SYS-15 cần dùng các giới hạn này khi xây dựng hồ sơ nghiệm thu.
