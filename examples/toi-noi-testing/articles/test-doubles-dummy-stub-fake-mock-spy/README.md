# TEST-057 · Test Doubles

Từ thư mục `examples/toi-noi-testing`, sau `npm ci`:

```bash
npx vitest run articles/test-doubles-dummy-stub-fake-mock-spy
```

7 ca trên `driverStep` và `chargeWithRetry`: dummy cho cash, stub từ chối, fake có trạng thái, kỳ vọng tương tác kiểu mock, spy gọi xuyên, retry lỗi tạm thời và không retry declined. R-PAY-02/03/04/06.

Helper `ongoingTrip` đi qua use case và repository trong bộ nhớ thật của lab. Phạm vi rộng hơn unit cô lập một hàm; không có HTTP, PostgreSQL hay ví thật. Không kiểm tên helper private. Mỗi test tạo hệ thống mới; `afterEach` khôi phục spy.
