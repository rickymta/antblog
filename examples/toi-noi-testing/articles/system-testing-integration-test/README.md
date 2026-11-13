# SYS-04 — Integration Test

[Bài đọc](../../../../docs/system-testing/04-integration-test.md) · [Báo cáo](../../../../docs/system-testing/04-test-report.md).

Tại `examples/toi-noi-testing`, dùng Node >=22.12; `npm ci` nếu chưa có dependency:

```sh
npm test -- articles/system-testing-integration-test
npm run typecheck
```

Expected: 7 tests pass (IN-03 có 5 dataset). Express/use cases/repository/PGlite chạy thật trong tiến trình; payment port, weather và clock là fake. Reset DB trước từng execution, dựng app mới, đóng DB cuối suite. Timeout rút xuống 50 ms để kiểm logic, không chứng minh deadline mặc định 3 giây. Không cần PostgreSQL server hoặc thanh toán ngoài.
