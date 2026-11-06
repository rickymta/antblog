# Lab 36: Realtime WebSocket

Lab của bài **FL-36** trong loạt **Flutter qua Tới Nơi**: [Realtime WebSocket: reconnect, backoff và theo dõi tài xế](../../../api/seed/content/flutter-realtime-websocket-reconnect-backoff-theo-doi-tai-xe.md).

`RealtimeClient` nói chuyện với Realtime Gateway giả của `tools/fake_api` (`RealtimeModule`, hợp đồng JSON ghi trong [README của fake_api](../tools/fake_api/README.md)): kết nối sau xác thực bằng token, nghe chuyến, nhận sự kiện bền có `sequence` và `version`, vị trí tài xế tạm thời và lời mời cuốc của tài xế. Mọi con số thời gian (heartbeat 15 giây, backoff 1 đến 30 giây, ổn định sau 30 giây, cửa sổ vị trí 1 giây) là minh hoạ.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/realtime_protocol.dart` | Đọc frame của server thành kiểu Dart (`ServerFrame`, `TripEventEnvelope`, `DriverLocation`, `DriverOffer`), mã hoá frame client gửi, `closeCodeUnauthorized` |
| `lib/realtime_transport.dart` | `RealtimeTransport` / `RealtimeConnection` (interface để test không socket), `WebSocketTransport` (bọc `WebSocket` của `dart:io`: tự `connectTimeout`, đóng socket tới muộn, đổi HTTP 401 thành `RealtimeAuthRejected`) |
| `lib/backoff.dart` | `Backoff`: exponential có trần với full jitter |
| `lib/realtime_client.dart` | `RealtimeClient`: bốn trạng thái, reconnect, heartbeat, resume theo `sequence` và `logEpoch`, khử trùng, lấp lỗ hổng, `log_reset` và `replay_unavailable`, gộp vị trí, kênh `offers`, huỷ khi `dispose`, tách lỗi xác thực |
| `lib/realtime_lifecycle.dart` | `RealtimeLifecycleBinding`: ngắt khi app `paused`, nối lại khi `resumed` |
| `test/support.dart`, `test/gateway_harness.dart` | Transport giả điều khiển được; fake API thật nghe ở localhost |

## Kiểm chứng (không cần simulator, không cần mạng ngoài)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/36-realtime-websocket
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (131 test). Mức kiểm chứng: **Kiểm chứng tự động**. Logic reconnect, heartbeat và gộp vị trí chạy trên `fake_async` với transport giả (không chờ thời gian thật). `test/real_socket_test.dart` (14 test) mở socket thật tới `shelf_io` ở `127.0.0.1`: file này là test Dart thuần, không khởi tạo `TestWidgetsFlutterBinding`, và chạy được trong `flutter test`. Nó dùng backoff vài mili giây và chỉ chờ sự kiện; ngoại lệ là test `connectTimeout`, dùng độ trễ 300 ms của fake API để tạo một bắt tay chậm. `test/binding_and_sockets_test.dart` ghi lại điều quan sát được với SDK đã kiểm về `HttpClient` trong `testWidgets`; đó là hành vi hiện tại chứ không phải hợp đồng của SDK.

Đã kiểm với Flutter 3.47.4 / Dart 3.13.3: `web_socket_channel` 3.0.3 (bản mới nhất lúc kiểm, ràng buộc `^3.0.3`), `shelf` 1.4.3, `shelf_web_socket` 3.0.0 (qua `toi_noi_fake_api` 0.3.0), `fake_async` 1.3.3, `flutter_lints` 6.0.0. Mọi phiên bản này khớp §4 của khung (`web_socket_channel` 3.0).

## Điểm cần nhớ khi dùng `RealtimeClient`

- `tokenProvider` phải trả token **còn hạn** (làm mới ở đó). Kết nối đang mở chỉ bị gateway đóng (4401) khi token hết hạn; token cũ khi nối lại thì dừng ở `closed(unauthorized)` và `resume()` không mở lại, phải `connect()` sau khi đăng nhập lại.
- `events` là stream một người nghe và **đệm** tới khi có người `listen` (dùng `asBroadcastStream()` nếu cần nhiều người). `locations`, `offers`, `issues`, `states` là broadcast.
- `log_reset` và `replay_unavailable` (từ gateway, hoặc do client tự phát hiện khi `logEpoch` đổi, `latestSequence` nhỏ hơn đã nhận, hay lỗ hổng không lấp được) làm client **thôi nghe** chuyến đó và báo qua `issues`; app tải lại bản chụp REST rồi `subscribe(tripId, lastSequence:, logEpoch:)` từ bản chụp.
- "Phát lại rồi đăng ký trong một lượt đồng bộ" chỉ đúng với gateway giả một tiến trình; kiểm `sequence` ở client mới là chốt chặn thật.

## Chạy với fake API trên máy (tự thử)

```bash
cd examples/toi-noi-flutter/tools/fake_api
dart pub get
dart run bin/server.dart --with-realtime
```

Lấy token bằng `POST /auth/otp/request` rồi `POST /auth/otp/verify` (mã OTP giả cố định, xem README của fake_api), đặt chuyến bằng `POST /trips`, rồi đẩy sự kiện bằng simulator:

```bash
curl -X POST localhost:8787/realtime/trips/<id>/events -H 'content-type: application/json' -d '{"type":"searchStarted"}'
curl -X POST localhost:8787/realtime/trips/<id>/location -H 'content-type: application/json' -d '{"driverId":"D-7","lat":10.7769,"lng":106.7009}'
```

Lab không có `main.dart` và không kèm `android/`, `ios/`. Muốn nhúng vào app: `RealtimeClient(transport: const WebSocketTransport(), url: Uri.parse('ws://10.0.2.2:8787/realtime'), tokenProvider: ...)` rồi `RealtimeLifecycleBinding(client)`. Android emulator gọi máy dev qua `10.0.2.2`, iOS Simulator dùng `localhost`; cleartext `ws://` trên thiết bị thật cần cấu hình mạng native (bài 25 và Phần 7). Sinh runner nếu muốn: `flutter create --platforms=ios,android --project-name toi_noi_36 --org vn.toinoi .`

## Chưa làm

- Chưa chạy trên simulator, emulator hay thiết bị: không kiểm được việc hệ điều hành thực sự cắt socket khi app đi nền, hay hành vi qua mạng di động và proxy. `RealtimeLifecycleBinding` được test bằng `handleAppLifecycleStateChanged` giả lập, không phải vòng đời thật.
- App tài xế giữ kết nối khi chạy nền (foreground service Android, vị trí nền): bài 38.
- Push khi app không foreground: bài 37.
- Tự làm mới token khi gateway từ chối (4401 hoặc HTTP 401): client dừng và để lớp xác thực (bài 24) quyết định; làm mới token trước khi trả từ `tokenProvider`.
- Lời mời cuốc: client chỉ nhận và đăng ký lại `offer`; đếm ngược 15 giây, nhận cuốc và điều hướng thuộc bài 47.
- Không có Riverpod provider hay màn hình; nối `events` và `locations` vào state của app là việc của bài 45 và 47.
- Web (`dart:html`/`package:web`): `WebSocketTransport` dùng `IOWebSocketChannel`, chỉ cho nền tảng có `dart:io`.
- Giới hạn kích thước frame, hàng đợi gửi khi mất kết nối, nén, subprotocol: chưa làm.
