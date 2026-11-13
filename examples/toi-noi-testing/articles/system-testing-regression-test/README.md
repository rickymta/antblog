# SYS-06 — Regression Test

[Bài đọc](../../../../docs/system-testing/06-regression-test.md) · [Báo cáo](../../../../docs/system-testing/06-test-report.md).

Tại thư mục `examples/toi-noi-testing`, Node >=22.12, `npm ci` nếu chưa có dependency:

```sh
npm test -- articles/system-testing-regression-test
npm run typecheck
```

Expected: 8 tests pass. RG-05 là thí nghiệm fault fixture cố ý sai, được assert như lỗi đã biết; không sửa `src/`, không giả vờ đã chạy mutation engine hoặc sửa bug sản phẩm. Ba API cases có system memory riêng, giờ cao điểm và mưa cố định. Không dùng DB/server ngoài.

Regression rộng cho các bài API đã viết:

```sh
npm test -- articles/system-testing-smoke-test articles/system-testing-sanity-test articles/system-testing-functional-test articles/system-testing-integration-test articles/system-testing-regression-test
```

System journeys (cần Chrome) dùng lệnh riêng `npx playwright test --config system-testing.config.ts`; Vitest không chạy browser specs.
