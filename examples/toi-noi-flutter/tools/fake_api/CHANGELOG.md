# Changelog

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
