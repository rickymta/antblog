# toi_noi_fake_api: server giả của Tới Nơi

Công cụ của loạt **Flutter qua Tới Nơi**, tạo ở bài **FL-19** ([HTTP, dio và JSON](../../../../api/seed/content/flutter-http-dio-json-dto-goi-api-bao-gia.md)). Dart thuần, dựng bằng `shelf` và `shelf_router`, phụ thuộc `toi_noi_core` để báo giá bằng đúng `calculateFare`. Không phải backend thật: xác thực chỉ có khi bật `--with-auth` và là đồ giả, không lưu xuống đĩa, mất hết dữ liệu khi tắt. Bảng giá (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền 0 đến 100 km) là minh hoạ.

## Chạy

```bash
cd examples/toi-noi-flutter/tools/fake_api
dart pub get
dart run bin/server.dart                  # http://127.0.0.1:8787
dart run bin/server.dart --port 9000 --latency-ms 800 --fail-first 2
dart run bin/server.dart --with-auth       # bật xác thực giả (bài 24)
dart run bin/server.dart --help
```

Mặc định server chỉ nghe trên loopback. Android emulator gọi máy dev qua `http://10.0.2.2:8787`, iOS Simulator dùng `http://localhost:8787`. Thiết bị thật trong cùng mạng LAN cần `--host 0.0.0.0` và địa chỉ IP của máy dev; hãy chỉ làm vậy trên mạng tin cậy vì server không có xác thực thật.

## Endpoint

| Method và đường dẫn | Body / header | Thành công | Lỗi có chủ đích |
|---|---|---|---|
| `GET /health` | | 200 `{"status":"ok"}` (không bị `FaultConfig` tiêm lỗi) | |
| `POST /quotes` | `{"serviceKind":"ride","distanceKm":5}` | 200 `{serviceKind, distanceMeters, fare, currency}` | 400 body hỏng; 422 sai trường hoặc ngoài miền 0..100 km |
| `POST /trips` | cùng body; header `Idempotency-Key` (1..128 ký tự `A-Za-z0-9_-`) | 201 + `Location` + chuyến `requested`; gửi lại cùng khoá và nội dung trả đúng chuyến cũ | 422 thiếu hoặc sai khoá, sai trường; 409 cùng khoá khác nội dung |
| `GET /trips/{id}` | | 200 chuyến | 404 không có chuyến |
| `POST /trips/{id}/accept` | `{"expectedVersion":0}`; header `x-fake-driver-id` | 200 chuyến `accepted`, `version` tăng 1 | 403 thiếu danh tính; 404; 422 thiếu `expectedVersion`; 409 sai phiên bản hoặc đã có tài xế |

### Xác thực giả (`AuthModule`, từ bài 24)

Chỉ có khi truyền `AuthModule` vào `modules` hoặc chạy `--with-auth`. **Chỉ dành cho lab:** mã OTP là hằng `fakeOtpCode` (`123456`) cho mọi yêu cầu và không nằm trong response nào; access token là JWT không ký (`alg: none`, chữ ký trống), server tra nguyên chuỗi token trong bảng đã phát chứ không đọc payload, nên sửa một ký tự là bị từ chối. Client không được coi payload là đáng tin, kể cả `role`. Số điện thoại chỉ cần đúng dạng `+` và 8 đến 15 chữ số; dùng số giả như `+84000000001`.

| Method và đường dẫn | Body | Thành công | Lỗi có chủ đích |
|---|---|---|---|
| `POST /auth/otp/request` | `{"phone":"+84000000001","app":"customer"}` (`app`: `customer` hoặc `driver`) | 200 `{requestId, expiresInSeconds}` (300 s) | 400 body hỏng; 422 sai `phone` hoặc `app` |
| `POST /auth/otp/verify` | `{"requestId":"otp-000001","code":"123456"}` | 200 `{accessToken, refreshToken, tokenType, expiresIn, user:{id,role}}` (access 15 phút, refresh 30 ngày: số minh hoạ); mã dùng được một lần | 422 mã sai, hết hạn (5 phút), đã dùng hoặc `requestId` lạ; 429 ở lần sai thứ ba và yêu cầu bị huỷ |
| `POST /auth/refresh` | `{"refreshToken":"rt_..."}` | 200 cặp token mới (token cũ thành "đã dùng") | 401 token lạ, hết hạn, họ đã bị thu hồi, hoặc **dùng lại token đã xoay** (thu hồi cả họ, token mới nhất cũng hỏng); 422 thiếu trường |
| `POST /auth/logout` | `{"refreshToken":"rt_..."}` | 204, thu hồi cả họ token; token lạ cũng 204 | 422 thiếu trường |

Access token không bị thu hồi khi đăng xuất hay khi họ bị thu hồi: JWT vẫn hợp lệ tới `exp`. Module không bắt buộc đăng nhập ở route nào; thêm `auth.requireBearer` vào `middleware` (hoặc dùng `--with-auth`) để `/trips` đòi `Authorization: Bearer <access token còn hạn>` (thiếu, sai hoặc hết hạn thì 401 kèm `www-authenticate`; `/health`, `/quotes`, `/auth/*` không bị kiểm). Lỗi tiêm bằng `FaultConfig` cũng áp lên các route này.

```dart
final auth = AuthModule(); // otpCode, otpTtl, accessTokenTtl... cấu hình được
final handler = createFakeApiHandler(
  clock: () => now, // test đổi `now` để access token hết hạn
  modules: [auth],
  middleware: [auth.requireBearer],
);
```

Hình dạng chuyến (camelCase): `id`, `serviceKind` (`ride`/`food`/`parcel`), `status` (tên enum của `TripStatus`, ví dụ `requested`, `pickedUp`), `distanceMeters` (số nguyên mét), `fare` (số nguyên đồng), `currency` (`VND`), `driverId` (hoặc `null`), `requestedAt` (UTC ISO-8601), `version`. Lỗi là `application/problem+json` với `title`, `status`, `traceId` (dạng `fake-000001`) và, khi sai đầu vào, `errors` theo từng trường.

### Khác với service .NET của bài 31 đến 38

| Điểm | Service .NET (`examples/toi-noi-dotnet/31-38-trip-service`) | Fake API |
|---|---|---|
| Xác thực | JWT, `Customer` / `Driver` policy | Mặc định không; `x-fake-driver-id` thay claim `sub`. Có `AuthModule` giả khi bật (JWT không ký, không có policy theo role) |
| `POST /trips` trả | 201 `{tripId, fare, currency}` | 201 cả chuyến (có `id`, không có `tripId`) |
| `GET /trips/{id}` trả | `{id, customerId, fare, currency, status, driverId, version}` | thêm `serviceKind`, `distanceMeters`, `requestedAt`; không có `customerId` |
| `status` | PascalCase, chỉ `Requested` và `Accepted` | camelCase, đủ 8 trạng thái |
| Lỗi nhập liệu | 400 | 422 (400 chỉ cho body hỏng) |
| `POST /trips/{id}/accept` trả | `{tripId, status, version}` | cả chuyến |
| Body `POST /trips` | `{distanceKm}` | thêm `serviceKind` |
| Khoá `Idempotency-Key` sai | 400 | 422 |
| Phạm vi khoá | theo từng customer (JWT) | một kho chung, không có customer |
| `distanceKm` | tối đa 6 chữ số thập phân | làm tròn tới mét |
| Báo giá | không có endpoint | `POST /quotes` (fake API mô phỏng thêm) |

Nhận chuyến trong fake đi qua `searching` rồi `accepted` bằng `Trip.apply` của `toi_noi_core`, vì không có bộ ghép cặp.

## Dùng trong test (không socket)

```dart
final handler = createFakeApiHandler(
  clock: () => DateTime.utc(2026, 11, 6, 3, 30),
  newId: () => 'T-1',
);
final response = await handler(Request('GET', Uri.parse('http://x/health')));
```

Tham số: `faults` (`FaultConfig`), `store` (`FakeApiStore`, để test xem dữ liệu sau khi gọi), `clock`, `newId`, `delay` (thay `Future.delayed` để không chờ thật), `middleware`, `modules`. Để gọi qua `dio` trong test, dùng `InProcessAdapter` của `package:toi_noi_fake_api/testing.dart` (xem mục dưới).

## Adapter dio chạy trong tiến trình

`package:toi_noi_fake_api/testing.dart` có `InProcessAdapter(handler)` (một `HttpClientAdapter` của dio chạy thẳng shelf `Handler`, không socket) và `RecordedRequest`. Package phụ thuộc `dio` vì file này; server (`bin/server.dart`) không dùng nó.

```dart
final adapter = InProcessAdapter(createFakeApiHandler());
final client = ToiNoiApiClient(baseUrl: Uri.parse('http://fake.test'), adapter: adapter);
// adapter.requests: method, uri, header (chữ thường), body đã gửi
```

Giống adapter thật: `receiveTimeout` quá hạn ném `DioException.receiveTimeout`; handler ném `SocketException` thì thành `DioException.connectionError`. Dùng được trong `dart test` và `flutter test`: khi binding của `flutter_test` đã khởi tạo (`testWidgets` hoặc `TestWidgetsFlutterBinding.ensureInitialized()`), `HttpClient` toàn cục bị thay bằng bản luôn trả 400 và adapter mặc định của dio không gọi được gì; `test()` thuần chưa khởi tạo binding thì vẫn mở socket thật. Adapter này không đi qua `HttpClient` nên chạy giống nhau ở cả hai.

## Tiêm lỗi và độ trễ

| Cờ / tham số | Ý nghĩa |
|---|---|
| `--latency-ms N` / `FaultConfig(latency:)` | chờ N ms trước mỗi request |
| `--fail-first N` / `failFirst` | N request đầu trả 500 |
| `--fail-every N` / `failEvery` | cứ N request thì một request trả 500 |

## Mở rộng

Thêm route bằng một `FakeApiModule` và truyền vào `modules`; thêm kiểm tra chung (token, giới hạn tốc độ) bằng `middleware`. Bài 24 dùng cả hai cho xác thực giả (`AuthModule` và `requireBearer`); bài 36 (WebSocket) sẽ dùng `modules`.

## Kiểm chứng

```bash
cd examples/toi-noi-flutter/tools/fake_api
dart pub get
dart analyze
dart test
```

Kỳ vọng: `No issues found!` và 91 test xanh (báo giá 16, chuyến 19, lỗi tiêm + mở rộng + tuỳ chọn dòng lệnh 19, adapter 9, xác thực giả 28). Mức kiểm chứng: **Kiểm chứng tự động**. Đã kiểm với Flutter 3.47.4 / Dart 3.13.3, `shelf` 1.4.3, `shelf_router` 1.1.5, `args` 2.7.0, `dio` 5.11.1.

## Chưa làm

- Xác thực thật: `AuthModule` là đồ giả (OTP cố định, JWT không ký, không có khoảng ân hạn cho refresh token bị mất response, không giới hạn tốc độ gửi OTP, không có mật khẩu hay quản lý thiết bị).
- WebSocket và realtime: bài 36.
- Chỉ có ba thao tác với chuyến (đặt, đọc, nhận); chưa có huỷ, `arriving`, `pickedUp`... và không có danh sách chuyến.
- Không lưu bền, không phân trang, không giới hạn tốc độ, không kiểm `Content-Type` của request.
- Chưa kiểm cấu hình mạng native để app trên thiết bị gọi được máy dev (cleartext HTTP, quyền mạng): bài 25 và Phần 7.
