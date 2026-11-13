# SYS-11 — Compatibility Test

[Bài đọc](../../../../docs/system-testing/11-compatibility-test.md) · [Báo cáo](../../../../docs/system-testing/11-test-report.md).

Từ `examples/toi-noi-testing`, Node >=22.12, Chrome và Edge cài sẵn; `npm ci` nếu cần:

```sh
npx playwright test --config compatibility.config.ts
npm run typecheck
```

Expected: hai cases × ba projects = sáu executions pass. Server PGlite riêng cổng 3116, không reuse, một worker, reset/đặt giờ mỗi case. Projects: Chrome desktop vi-VN/VN; Edge desktop en-US/New York; Chrome viewport 390×844 với mobile/touch emulation. Đây không phải máy Android/iOS thật; Chrome và Edge đều thuộc họ Chromium.

Lệnh config mặc định của lab có thể phát hiện spec nhưng không chạy đủ ma trận này. Chạy riêng: `npx playwright test --config compatibility.config.ts --project=edge-desktop-en`. Mỗi case attach metadata browser/version/config; trace/screenshot chỉ lưu khi lỗi dưới reports/compatibility. Không có visual snapshot baseline.
