# SYS-12 — Usability Test

[Bài đọc](../../../../docs/system-testing/12-usability-test.md) · [Bộ tài liệu nghiên cứu](../../../../docs/system-testing/12-study-kit.md) · [Báo cáo](../../../../docs/system-testing/12-test-report.md).

Từ `examples/toi-noi-testing`, Chrome cài sẵn, Node >=22.12, `npm ci` nếu cần:

```sh
npx playwright test --config usability.config.ts
npm run typecheck
```

Expected: 2 supporting checks pass. Đây là kiểm nhãn, một đoạn Tab/Enter và error recovery trong DOM/UI; không phải nghiên cứu người dùng hoặc đánh giá WCAG đầy đủ. Một worker, server PGlite riêng cổng 3117, không reuse, reset/đặt giờ từng case. Trace/screenshot chỉ khi lỗi.

Chưa có người tham gia study. Bộ tài liệu nghiên cứu có script, nhiệm vụ, scoring, mẫu ghi chép và finding để tổ chức sau; không điền số liệu giả vào mẫu.
