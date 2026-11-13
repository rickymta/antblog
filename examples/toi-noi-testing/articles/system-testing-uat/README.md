# SYS-15 — UAT supporting checks

[Bài học](../../../../docs/system-testing/15-uat.md) · [Acceptance kit](../../../../docs/system-testing/15-acceptance-kit.md) · [Báo cáo](../../../../docs/system-testing/15-test-report.md)

Từ thư mục lab:

```powershell
npm ci
npm test -- articles/system-testing-uat
npm run typecheck
```

Bốn test API/PGlite: chuyến cash hoàn chỉnh và replay; huỷ ở 1/2 phút; quote hết hạn đúng 5 phút. FixedClock/seed reset từng case, đóng DB sau suite. Không cần browser/server ngoài. Bộ test hỗ trợ tiêu chí UA-01/02/03, không thay UAT người dùng hay sign-off. Các kịch bản UI, vận hành và quyết định giữ trạng thái chưa thực hiện trong kit.
