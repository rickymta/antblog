# toi_noi_fake_api: server giả của Tới Nơi

Công cụ của loạt **Flutter qua Tới Nơi**, tạo ở bài **FL-19** ([HTTP, dio và JSON](../../../../api/seed/content/flutter-http-dio-json-dto-goi-api-bao-gia.md)). Dart thuần, dựng bằng `shelf`, `shelf_router` và `shelf_web_socket`, phụ thuộc `toi_noi_core` để báo giá bằng đúng `calculateFare`. Không phải backend thật: xác thực chỉ có khi bật `--with-auth` và là đồ giả, không lưu xuống đĩa, mất hết dữ liệu khi tắt. Bảng giá (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền 0 đến 100 km) là minh hoạ.

## Chạy

```bash
cd examples/toi-noi-flutter/tools/fake_api
dart pub get
dart run bin/server.dart                  # http://127.0.0.1:8787
dart run bin/server.dart --port 9000 --latency-ms 800 --fail-first 2
dart run bin/server.dart --with-auth       # bật xác thực giả (bài 24)
dart run bin/server.dart --with-realtime   # bật Realtime Gateway giả (bài 36)
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

## Realtime Gateway giả (`RealtimeModule`, từ bài 36)

Chỉ có khi truyền `RealtimeModule` vào `modules` hoặc chạy `--with-realtime`. Không truyền thì mọi route và mọi response chạy như 0.2.0 (kể cả JSON của chuyến). **Chỉ dành cho lab:** gateway thật nhận sự kiện từ Kafka hoặc pub/sub nội bộ, còn ở đây sự kiện đi vào bằng hàm Dart hoặc các route simulator (không đòi đăng nhập). Log nằm trong bộ nhớ, mất khi tắt server. **Hợp đồng JSON dưới đây là ổn định**: bài 45 (máy trạng thái chuyến trên client) và bài 47 (app tài xế) dựa vào nó; đổi thì phải tăng phiên bản chính và sửa các bài đó.

`--with-realtime` mount cả `AuthModule` (để lấy token bằng OTP giả) nhưng chỉ đòi Bearer ở `/trips` khi có thêm `--with-auth`.

```dart
final auth = AuthModule();
final realtime = RealtimeModule(
  verifyToken: auth.verifyAccessToken,
  authorize: (who, trip) => true, // tuỳ chọn: ai được nghe chuyến nào
  maxLogPerTrip: 100,             // tuỳ chọn: cắt log để thử replay_unavailable
  rejectBeforeUpgrade: false,     // true: token xấu bị HTTP 401 ngay ở bước nâng cấp
);
final handler = createFakeApiHandler(modules: [auth, realtime]);
// shelf_io.serve(handler, host, port) mở socket; WebSocket cần server thật.

realtime.publish('T-1', const DriverAccepted('D-7'));   // sự kiện bền, ghi vào kho
realtime.publishLocation('T-1', const DriverLocationUpdate(driverId: 'D-7', lat: 10.77, lng: 106.7));
realtime.publishOffer('U-1001', 'T-1');                  // lời mời cuốc cho tài xế U-1001
await realtime.dropConnections();                        // mất kết nối: đóng mã 4001, giữ log
await realtime.restart();                                // mất log: logEpoch mới, log rỗng
await realtime.closeExpired();                           // đóng (4401) kết nối có token quá hạn
```

### Kết nối

`GET ws://127.0.0.1:8787/realtime` với header `Authorization: Bearer <access token>` (token từ `POST /auth/otp/verify`). Không đặt token trong URL.

- Mặc định server **luôn nâng cấp** lên WebSocket rồi mới kiểm token: token thiếu, lạ hoặc hết hạn thì server đóng ngay bằng mã **4401** (lý do `unauthorized`). Với `rejectBeforeUpgrade: true`, server trả **HTTP 401** ở bước nâng cấp (kiểu của nhiều proxy); client `dart:io` đọc mã đó ở `WebSocketException.httpStatusCode`. Client phải coi cả hai là lỗi xác thực và không nối lại.
- Token được kiểm lúc bắt tay **và** khi hết hạn: kết nối đang mở bị đóng bằng **4401** (lý do `token expired`) đúng hạn của access token (hoặc khi gọi `closeExpired()`). Client phải làm mới token rồi `connect` lại.
- Mã đóng khác mà client cần biết: **4001** (gateway chủ động ngắt, lý do `server restart`, nối lại được) và 1000 (đóng bình thường). `web_socket_channel` chỉ gửi được mã 1000 hoặc 3000 đến 4999.

Mỗi frame là một text frame chứa **một JSON object có `type`**. Frame không phải JSON object thì server trả `error` và giữ kết nối. Tên trường là camelCase, thời điểm là UTC ISO-8601 (đuôi `Z`).

### Client gửi server

| `type` | Trường | Ý nghĩa |
|---|---|---|
| `subscribe` | `tripId` (string), `lastSequence` (số nguyên không âm, mặc định 0), `logEpoch` (string, tuỳ chọn) | Nghe chuyến. `lastSequence` và `logEpoch` là điểm client đã nhận tới (lấy từ frame `subscribed`, hoặc từ `lastSequence`, `logEpoch` của REST). Server phát lại mọi sự kiện có `sequence > lastSequence`, rồi sự kiện mới. Gửi lại cho chuyến đã nghe là hợp lệ. Nếu `logEpoch` khác `logEpoch` hiện tại, hoặc `lastSequence` lớn hơn log hiện có, server trả `error` `log_reset` và **không** đăng ký (gỡ luôn đăng ký cũ của kết nối với chuyến đó) |
| `subscribe` | `channel`: `"offers"` | Nghe lời mời cuốc **của chính người dùng trong token**; không có tham số mã tài xế nên không thể nghe của người khác. Chỉ vai trò `driver`, nếu không thì `error` `forbidden` kèm `channel: "offers"` |
| `unsubscribe` | `tripId` hoặc `channel: "offers"` | Thôi nghe |
| `ping` | `id` (giá trị JSON bất kỳ, tuỳ chọn) | Server trả `pong` cùng `id` |

### Server gửi client

| `type` | Trường | Ý nghĩa |
|---|---|---|
| `welcome` | `connectionId`, `userId` | Gửi ngay sau khi token được chấp nhận; từ lúc này kết nối đã xác thực |
| `subscribed` | `tripId`, `latestSequence`, `replayed`, `logEpoch` | Xác nhận `subscribe`, gửi **trước** các sự kiện phát lại. `replayed` là số sự kiện sắp được phát lại; `logEpoch` là định danh log hiện tại |
| `subscribed` | `channel`: `"offers"` | Xác nhận đăng ký kênh lời mời |
| `event` | `tripId`, `sequence`, `version`, `occurredAt`, `status`, `event` | **Sự kiện bền.** `sequence` bắt đầu từ 1 và tăng đúng 1 mỗi sự kiện của chuyến trong một `logEpoch`. `version` là `version` của chuyến sau sự kiện, cùng thước đo với REST. `status` là `TripStatus.name` sau sự kiện. `event` xem dưới |
| `location` | `tripId`, `driverId`, `lat`, `lng`, `headingDegrees?`, `speedMetersPerSecond?`, `at` | **Dữ liệu tạm thời.** Không có `sequence`, không được phát lại; server chỉ nhớ bản mới nhất của chuyến và gửi cho người `subscribe` đến sau (sau phần phát lại). `at` là thời điểm server nhận |
| `offer` | `offerId`, `tripId`, `serviceKind`, `distanceMeters`, `fare` (đồng), `currency`, `expiresAt` | **Lời mời cuốc cho một tài xế.** Tạm thời: không lưu, không phát lại; tài xế offline thì mất. Chỉ kết nối của đúng người dùng đã `subscribe` kênh `offers` nhận |
| `unsubscribed` | `tripId` hoặc `channel` | Xác nhận `unsubscribe` |
| `pong` | `id` | Trả lời `ping` |
| `error` | `code`, `message`, `tripId?`, `channel?` | Lỗi giao thức, kết nối vẫn mở. `code`: `bad_frame`, `unknown_type`, `trip_not_found`, `forbidden`, `log_reset` (log khác: client phải tải lại bản chụp REST rồi `subscribe` lại với `lastSequence` và `logEpoch` của bản chụp), `replay_unavailable` (sự kiện sau `lastSequence` đã bị cắt khỏi log, cùng cách xử lý) |

`event.event` là một `TripEvent` của `toi_noi_core`, có `type` là tên lớp viết camelCase:

| `event.type` | Trường thêm | Chuyển trạng thái |
|---|---|---|
| `searchStarted` | | `requested` sang `searching` |
| `driverAccepted` | `driverId` | `searching` sang `accepted` |
| `driverArrived` | | `accepted` sang `arriving` |
| `pickedUp` | | `arriving` sang `pickedUp` |
| `tripStarted` | | `pickedUp` sang `inTransit` |
| `completed` | | `inTransit` sang `completed` |
| `cancelled` | `by`: `customer` hoặc `driver` | từ `searching`, `accepted` hoặc `arriving` sang `cancelled` |

Ví dụ:

```json
{"type":"event","tripId":"T-1","sequence":2,"version":2,"occurredAt":"2026-11-06T03:30:12.000Z","status":"accepted","event":{"type":"driverAccepted","driverId":"D-7"}}
{"type":"location","tripId":"T-1","driverId":"D-7","lat":10.7769,"lng":106.7009,"headingDegrees":90.0,"at":"2026-11-06T03:30:15.000Z"}
{"type":"offer","offerId":"O-0001","tripId":"T-1","serviceKind":"ride","distanceMeters":5000,"fare":25500,"currency":"VND","expiresAt":"2026-11-06T03:30:30.000Z"}
```

### REST và realtime dùng chung một chuyến

`publish` (và route simulator) **ghi vào kho**: áp dụng `Trip.apply` lên chuyến trong kho và tăng `version` thêm 1, nên `GET /trips/{id}` luôn thấy trạng thái mà realtime vừa báo. Khi `RealtimeModule` được mount, mọi response REST trả chuyến (`POST /trips`, `GET /trips/{id}`, `POST /trips/{id}/accept`) có thêm hai trường:

| Trường | Ý nghĩa |
|---|---|
| `lastSequence` | `sequence` của sự kiện bền mới nhất của chuyến trong log hiện tại (0 nếu chưa có) |
| `logEpoch` | định danh log mà `lastSequence` thuộc về |

Cách hợp nhất cho client (bài 45): lấy bản chụp REST rồi `subscribe` với `lastSequence` và `logEpoch` của nó (không bỏ sót, không phát lại), và lấy `version` để so bản chụp với `event` (`event.version` không lớn hơn `version` đang có thì là cũ). `POST /trips/{id}/accept` của REST đổi trạng thái và tăng `version` nhưng **không phát sự kiện realtime** và không tăng `lastSequence`; hãy chọn một cách đi chuyến: simulator, hoặc REST accept.

### Log, `logEpoch` và `restart`

Mỗi chuyến có một log sự kiện bền trong bộ nhớ, gắn với `logEpoch` của tiến trình (mỗi lần khởi động một giá trị mới). `sequence` chỉ so được trong cùng `logEpoch`. `restart()` giả lập gateway khởi động lại và mất log: log và vị trí cuối bị xoá, sang `logEpoch` mới, mọi kết nối bị đóng (mã 4001); trạng thái và `version` của chuyến trong kho giữ nguyên, `sequence` kế tiếp bắt đầu lại từ 1. Với `maxLogPerTrip`, sự kiện cũ nhất bị cắt; `subscribe` xin phần đã cắt nhận `replay_unavailable`.

**Đơn giản hoá của lab so với gateway thật** (hồ sơ kiến trúc mục Realtime chỉ giao gateway quản lý kết nối, subscription và fan-out): gateway thật không nhất thiết giữ log (có thể lấy lại từ dịch vụ chuyến), có thể nhiều node nên "phát lại rồi đăng ký trong một lượt đồng bộ" chỉ đúng với fake này; quyền nghe mặc định là mọi người dùng đã xác thực nghe được mọi chuyến có thật (gateway thật phải kiểm khách hoặc tài xế có thuộc chuyến; `authorize` là chỗ cắm); và fake luôn nâng cấp rồi mới kiểm token.

### Simulator HTTP

Thay cho Kafka. Không đòi đăng nhập. Chuyến phải có trong kho (tạo bằng `POST /trips`).

| Method và đường dẫn | Body | Thành công | Lỗi |
|---|---|---|---|
| `POST /realtime/trips/{id}/events` | một `event` như bảng trên, ví dụ `{"type":"driverAccepted","driverId":"D-7"}` | 201 `{tripId, sequence, version, status}` | 404 không có chuyến; 409 sự kiện không hợp lệ ở trạng thái hiện tại (theo `Trip.apply`), `sequence` và `version` không đổi; 422 `type`, `driverId` hoặc `by` sai |
| `POST /realtime/trips/{id}/location` | `{"driverId":"D-7","lat":10.77,"lng":106.7,"headingDegrees":90,"speedMetersPerSecond":8.3}` (hai trường cuối tuỳ chọn) | 202 `{tripId, delivered}` (số kết nối đã nhận) | 404; 422 thiếu hoặc ngoài miền (`lat` -90..90, `lng` -180..180) |
| `POST /realtime/drivers/{userId}/offers` | `{"tripId":"T-1","ttlSeconds":15}` (`ttlSeconds` tuỳ chọn, 1 đến 600, mặc định 15) | 202 `{offerId, delivered, expiresAt}`. `{userId}` là `userId` trong frame `welcome` của tài xế (ví dụ `U-1001`); `serviceKind`, `distanceMeters`, `fare` lấy từ chuyến trong kho | 404 không có chuyến; 409 chuyến không còn ở `requested` hoặc `searching`; 422 `tripId` hoặc `ttlSeconds` sai |

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

Thêm route bằng một `FakeApiModule` và truyền vào `modules`; thêm kiểm tra chung (token, giới hạn tốc độ) bằng `middleware`. Bài 24 dùng cả hai cho xác thực giả (`AuthModule` và `requireBearer`); bài 36 dùng `modules` cho `RealtimeModule` (WebSocket).

## Kiểm chứng

```bash
cd examples/toi-noi-flutter/tools/fake_api
dart pub get
dart analyze
dart test
```

Kỳ vọng: `No issues found!` và 132 test xanh (báo giá 16, chuyến 19, lỗi tiêm + mở rộng + tuỳ chọn dòng lệnh 19, adapter 9, xác thực giả 28, realtime 41). Test realtime mở socket thật tới `shelf_io` ở `127.0.0.1` cổng do hệ điều hành chọn, nên cần `dart test` (không phải `flutter test` có binding, vì binding chặn HTTP thật). Mức kiểm chứng: **Kiểm chứng tự động**. Đã kiểm với Flutter 3.47.4 / Dart 3.13.3, `shelf` 1.4.3, `shelf_router` 1.1.5, `shelf_web_socket` 3.0.0, `web_socket_channel` 3.0.3, `args` 2.7.0, `dio` 5.11.1.

## Chưa làm

- Xác thực thật: `AuthModule` là đồ giả (OTP cố định, JWT không ký, không có khoảng ân hạn cho refresh token bị mất response, không giới hạn tốc độ gửi OTP, không có mật khẩu hay quản lý thiết bị).
- Realtime thật: `RealtimeModule` không có Kafka hay pub/sub, không lưu bền log sự kiện (khởi động lại là mất và sang `logEpoch` mới), không kiểm quyền nghe theo khách hay tài xế của chuyến (mặc định mọi người dùng đã xác thực nghe được mọi chuyến có thật), không giới hạn kích thước frame, không đóng kết nối rảnh, không xử lý subscriber chậm, lời mời cuốc chỉ có `offer` (chưa có hết hạn hay thu hồi do server, nhận cuốc vẫn qua `POST /trips/{id}/accept`).
- Chỉ có ba thao tác với chuyến (đặt, đọc, nhận); chưa có huỷ, `arriving`, `pickedUp`... và không có danh sách chuyến.
- Không lưu bền, không phân trang, không giới hạn tốc độ, không kiểm `Content-Type` của request.
- Chưa kiểm cấu hình mạng native để app trên thiết bị gọi được máy dev (cleartext HTTP, quyền mạng): bài 25 và Phần 7.
