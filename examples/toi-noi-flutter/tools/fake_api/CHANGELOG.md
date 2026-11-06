# Changelog

## 0.3.0

- Bài 36 của loạt Flutter qua Tới Nơi. Chỉ thêm, hành vi mặc định không đổi: không truyền `RealtimeModule` thì mọi route và mọi response (kể cả JSON của chuyến) chạy như 0.2.0.
- `RealtimeModule` (`FakeApiModule`): WebSocket `GET /realtime` xác thực bằng header `Authorization: Bearer`.
  - Chuyến: `subscribe` với `lastSequence` và `logEpoch` để phát lại; `event` bền có `sequence` (tăng đúng 1 trong một `logEpoch`) và `version` (cùng thước đo với REST); `location` tạm thời; `ping`/`pong`; `error` (`bad_frame`, `unknown_type`, `trip_not_found`, `forbidden`, `log_reset`, `replay_unavailable`) không làm rớt kết nối.
  - Kênh `offers` theo người dùng trong token (chỉ vai trò `driver`): frame `offer` cho FL-47.
  - Xác thực: token xấu bị đóng mã 4401 (hoặc HTTP 401 ở bước nâng cấp khi `rejectBeforeUpgrade: true`); kết nối đang mở bị đóng 4401 khi token hết hạn. `dropConnections()` đóng mã 4001 và giữ log; `restart()` xoá log, sang `logEpoch` mới.
  - `publish` ghi vào kho (áp dụng `Trip.apply`, tăng `version`), nên REST và realtime cùng một chuyến. Khi mount, response REST trả chuyến có thêm `lastSequence` và `logEpoch`.
  - Simulator thay Kafka: `POST /realtime/trips/{id}/events` (409 nếu sai thứ tự), `POST /realtime/trips/{id}/location`, `POST /realtime/drivers/{userId}/offers`; hàm Dart `publish`, `publishLocation`, `publishOffer`, `dropConnections`, `restart`, `closeExpired`. Tuỳ chọn `maxLogPerTrip`, `authorize`, `rejectBeforeUpgrade`, `offerTtl`.
- `AuthModule.verifyAccessToken(token)` (hàm công khai mới, trả cả `expiresAt`) cho bước bắt tay realtime.
- `FakeApiContext` bỏ `const` (có thêm danh sách `tripJsonExtras`, rỗng thì response không đổi); `tripToJson` nhận thêm tham số tuỳ chọn `extras`.
- Sửa nhỏ không đổi hành vi hiện có: middleware bắt lỗi thành problem+json cho `HijackException` đi qua thay vì trả 500, để nâng cấp WebSocket hoạt động trong cùng pipeline.
- `bin/server.dart --with-realtime` (kèm `AuthModule` để lấy token); `ServerOptions.withRealtime` (mặc định `false`).
- Thêm dependency `shelf_web_socket` ^3.0.0 và `web_socket_channel` ^3.0.3. 41 test mới (`test/realtime_test.dart`, socket thật tới `shelf_io` ở localhost), tổng 132.

## 0.2.0

- Bài 24 của loạt Flutter qua Tới Nơi. Chỉ thêm, hành vi mặc định không đổi: không truyền `AuthModule` thì mọi route chạy như 0.1.0.
- `AuthModule` (`FakeApiModule`): `POST /auth/otp/request`, `POST /auth/otp/verify`, `POST /auth/refresh` (xoay vòng: dùng lại refresh token đã xoay thì thu hồi cả họ token và trả 401), `POST /auth/logout` (thu hồi họ token, 204).
- OTP là hằng cố định `fakeOtpCode` (`123456`), **chỉ dành cho fake**. Access token là JWT **không ký** (`alg: none`, chữ ký trống); server tra nguyên chuỗi token trong bảng đã phát nên token bị sửa payload bị từ chối.
- `AuthModule.requireBearer` (middleware, tuỳ chọn): đòi `Authorization: Bearer` ở `/trips` (tiền tố cấu hình được bằng `protectedPrefixes`).
- `bin/server.dart --with-auth` bật `AuthModule` cùng `requireBearer`; `ServerOptions.withAuth` (mặc định `false`).
- 28 test mới (`test/auth_test.dart`), tổng 91.

## 0.1.0

- Bản đầu tiên, tạo ở bài 19 của loạt Flutter qua Tới Nơi.
- `createFakeApiHandler({...})` trả một shelf `Handler` không mở socket; `bin/server.dart` bọc nó bằng `shelf_io.serve` (`--host`, `--port`, mặc định `127.0.0.1:8787`).
- Endpoint: `GET /health`, `POST /quotes` (cước bằng `calculateFare` của `toi_noi_core`), `POST /trips` (header `Idempotency-Key`), `GET /trips/{id}`, `POST /trips/{id}/accept` (body `expectedVersion`, header `x-fake-driver-id` thay claim `sub` của JWT).
- Lỗi dạng `application/problem+json` (`title`, `status`, `errors`, `traceId`): 400 body hỏng, 403 thiếu danh tính tài xế, 404 không có chuyến hoặc route, 409 đặt trùng khoá khác nội dung / nhận chuyến trùng hoặc sai `expectedVersion`, 422 ngoài miền 0..100 km hoặc sai trường, 500 do `FaultConfig` tiêm.
- `FaultConfig(latency, failFirst, failEvery)` và các cờ `--latency-ms`, `--fail-first`, `--fail-every`; `delay` truyền vào được để test không chờ thật.
- Điểm mở rộng: `FakeApiModule` (`mount(Router, FakeApiContext)`) và danh sách `middleware` cho bài xác thực (24) và WebSocket (36).
- `package:toi_noi_fake_api/testing.dart`: `InProcessAdapter` và `RecordedRequest` (chuyển từ `toi_noi_api`), thêm phụ thuộc `dio`. Handler ném `SocketException` thành `DioException.connectionError`.
