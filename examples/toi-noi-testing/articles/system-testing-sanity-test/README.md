# SYS-02 — Sanity Test

Bài đọc: [Sanity Test](../../../../docs/system-testing/02-sanity-test.md).

Từ thư mục `examples/toi-noi-testing` (Node >=22.12, `npm ci` nếu chưa cài):

```sh
npm test -- articles/system-testing-sanity-test
npm run typecheck
```

Expected: 8 tests pass (SN-04 có ba dataset), không skip. Mỗi case tạo system/app mới, đồng hồ cố định, kho memory và ví giả. Không cần server ngoài. `beforeFix` là fault fixture thiếu trim, không phải lịch sử lỗi thật của sản phẩm. Test SN-01 kiểm output sai của fixture và output đúng của hàm hiện tại; không phải một lượt chạy toàn API trên build lỗi.

Kết quả thực đo: [báo cáo](../../../../docs/system-testing/02-test-report.md).
