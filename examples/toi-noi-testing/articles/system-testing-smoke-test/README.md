# SYS-01 — Smoke Test

Bài đọc: [Smoke Test](../../../../docs/system-testing/01-smoke-test.md).
Test basis: [SPEC](../../SPEC.md), R-API-03, R-STATE-02, R-PAY-03 và bảng cước.

Từ `examples/toi-noi-testing`, dùng Node >=22.12 và cài dependency bằng `npm ci` nếu chưa có:

```sh
npx vitest run articles/system-testing-smoke-test
npm run typecheck
```

Expected: 3 test pass, exit code 0. Mỗi case reset dữ liệu, dựng app mới; cuối suite đóng PGlite. Không cần server ngoài hoặc DB production. Ví và thời tiết là fake; HTTP dùng supertest, không kiểm DNS/TLS/reverse proxy. Kết quả thực đo nằm ở `docs/system-testing/test-report.md`.
