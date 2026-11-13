# SYS-03 — Functional Test

[Bài đọc](../../../../docs/system-testing/03-functional-test.md) · [Báo cáo](../../../../docs/system-testing/03-test-report.md).

Từ `examples/toi-noi-testing`, Node >=22.12, cài theo lockfile bằng `npm ci` nếu chưa có:

```sh
npm test -- articles/system-testing-functional-test
npm run typecheck
```

Expected: 24 tests pass: 8 dataset biên, 8 quy tắc bảng quyết định, 4 dataset validation và 4 ca nghiệp vụ. Mỗi execution có system/app/clock riêng; kho memory, thời tiết/ví giả; không cần server hoặc DB ngoài. Không sửa `src/` dùng chung.
