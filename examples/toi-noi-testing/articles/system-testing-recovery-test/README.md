# SYS-14 — Recovery Test

[Bài học](../../../../docs/system-testing/14-recovery-test.md) · [Báo cáo](../../../../docs/system-testing/14-test-report.md)

Từ thư mục lab:

```powershell
npm ci
npm test -- articles/system-testing-recovery-test
npm run typecheck
```

5 test API/PGlite: lỗi tạm thời rồi phục hồi (3 mode), việc mới sau outage, và khoản đã thu nhưng mất cả ba phản hồi. Timeout test 30 ms; không đo RTO/RPO. Trạng thái failed còn tồn sau khi ví trở lại là giới hạn cần đối soát, không tự sửa trong bài.

Trên Windows có PostgreSQL binaries trên PATH, từ root repo:

```powershell
& ./examples/toi-noi-testing/articles/system-testing-recovery-test/restore-drill.ps1
```

Script tạo cluster tạm riêng trên 127.0.0.1:55484 (đổi bằng `-Port`), trust auth cho dữ liệu tổng hợp. Backup marker 1, commit marker 2 sau backup, xoá nguồn rồi restore vào DB mới: chỉ marker 1 trở lại; kiểm ghi marker 3. Dừng cluster trong finally, giữ dump/log/result.json trong TEMP. Không chạm cluster hiện có; không cần credential của người dùng. Không đo application RTO, RPO thời gian, PITR hay restore schema Tới Nơi.
