# toi_noi_api: client HTTP và DTO của Tới Nơi

Package dùng chung của loạt **Flutter qua Tới Nơi**, tạo ở bài **FL-19** ([HTTP, dio và JSON](../../../../api/seed/content/flutter-http-dio-json-dto-goi-api-bao-gia.md)). Dart thuần (không import Flutter), dựng trên `dio`, phụ thuộc `toi_noi_core`; `toi_noi_core` không biết gì về HTTP hay JSON.

## API công khai

`package:toi_noi_api/toi_noi_api.dart`:

| Tên | Vai trò |
|---|---|
| `ToiNoiApiClient` | `quote(QuoteRequestDto)`, `createTrip(QuoteRequestDto, idempotencyKey:)`, `getTrip(id)`, `acceptTrip(id, expectedVersion:)`, `close()`; `baseUrl` truyền vào |
| `QuoteRequestDto` | `serviceKind` + `distanceKm`, `toJson` / `fromJson` |
| `QuoteDto` | `serviceKind`, `distanceMeters`, `fareVnd`; `toMoney()` |
| `TripDto` | `id`, `serviceKind`, `status`, `distanceMeters`, `fareVnd`, `driverId`, `requestedAt`, `version`; `fromJson`, `toJson`, `fromDomain`, `toDomain()`, `toSnapshot()` |
| `TripSnapshot` | `Trip trip` + `int version` (version làm `expectedVersion` khi tài xế nhận) |
| `ApiException` | HTTP không 2xx: `statusCode`, `title`, `fieldErrors`, `traceId` |
| `DtoFormatException` | payload sai hợp đồng: `dto`, `field`, thông báo |
| `SafeLogInterceptor` | log request/response không lộ dữ liệu nhạy cảm |
| `ApiFailure` (sealed) | `NetworkFailure`, `TimeoutFailure`, `ServerFailure`, `ClientFailure`, `ParseFailure` (bài 20) |
| `toApiFailure`, `guardApi` | đổi `DioException` / `ApiException` / `DtoFormatException` sang `ApiFailure` (bài 20) |
| `RetryInterceptor` | thử lại lỗi tạm thời của request lặp lại được, backoff có jitter, `Retry-After`, hạn tổng `maxElapsed` (bài 20) |

`InProcessAdapter(handler)` và `RecordedRequest` nằm ở `package:toi_noi_fake_api/testing.dart`.

## Dùng

```dart
final client = ToiNoiApiClient(
  baseUrl: Uri.parse('http://localhost:8787'),
  interceptors: [SafeLogInterceptor(print)],
);
final quote = await client.quote(
  const QuoteRequestDto(serviceKind: ServiceKind.ride, distanceKm: 5),
);
print(quote.toMoney()); // 25.500 VND
```

## Dùng adapter giả trong test (không socket)

Thêm `toi_noi_fake_api` làm `dev_dependency` của project test (`path: ../tools/fake_api`, tính từ vị trí project), rồi:

```dart
import 'package:toi_noi_fake_api/testing.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

final adapter = InProcessAdapter(createFakeApiHandler());
final client = ToiNoiApiClient(
  baseUrl: Uri.parse('http://fake.test'),
  adapter: adapter,
);
// adapter.requests ghi lại method, uri, header, body đã gửi.
```

Khi binding của `flutter_test` đã khởi tạo (`testWidgets` hoặc `TestWidgetsFlutterBinding.ensureInitialized()`), `HttpClient` toàn cục bị thay bằng bản luôn trả 400, nên client dùng adapter mặc định của `dio` nhận 400. `test()` thuần chưa khởi tạo binding thì vẫn mở socket thật. Adapter trong tiến trình không đi qua `HttpClient`, nên chạy giống nhau ở cả hai. Package này không phụ thuộc `shelf` khi chạy; `shelf` và `toi_noi_fake_api` chỉ là dev dependency.

## Hợp đồng dữ liệu

Tiền là `int` đồng; khoảng cách truyền là `int` mét (`distanceMeters`), riêng đầu vào khách nhập là `distanceKm` (số thực); thời điểm là UTC ISO-8601 có `Z` hoặc `+hh:mm` (thiếu múi giờ bị từ chối; ngày giờ tràn như `2026-13-45` cũng bị từ chối); trạng thái là tên enum của `TripStatus`, đọc không phân biệt hoa thường để nhận cả `pickedUp` (fake API) lẫn `PickedUp` (service .NET); `fare` nhận `25500` và `25500.0` nhưng không nhận `25500.5`; `currency` chỉ nhận `VND`; trường lạ được bỏ qua.

Service .NET của bài 31 đến 38 hiện thiếu `serviceKind`, `distanceMeters`, `requestedAt` trong `GET /trips/{id}` và trả `{tripId, fare, currency}` cho `POST /trips`, nên `TripDto.fromJson` từ chối view hiện tại của nó bằng `DtoFormatException` nêu đúng trường thiếu (có test). Bảng khác biệt đầy đủ nằm ở README của `tools/fake_api`.

## Kiểm chứng

```bash
cd examples/toi-noi-flutter/packages/toi_noi_api
dart pub get
dart analyze
dart test
```

Các test của bài 19 là 86 (DTO 43, client 33, log 9, loopback 1); thư mục `test/` còn có test của bài 20 (retry, failure). `loopback_test.dart` mở một socket loopback tạm để chứng minh `InProcessAdapter` cho cùng kết quả với adapter thật; các test còn lại không dùng socket. Mức kiểm chứng: **Kiểm chứng tự động**. Đã kiểm với Flutter 3.47.4 / Dart 3.13.3, `dio` 5.11.1.

## Chưa làm

- Timeout riêng cho từng loại request (hiện một bộ `connectTimeout` / `sendTimeout` / `receiveTimeout` cho cả client) và `Retry-After` dạng ngày giờ HTTP (chỉ đọc dạng số giây): chưa làm.
- Xác thực, gắn token, refresh khi 401: bài 24.
- Realtime (WebSocket): bài 36.
- Sinh DTO bằng `json_serializable` / `freezed`: bài 26.
- Huỷ request (`CancelToken`), tải lên, phân trang.

## Lỗi, retry và test lỗi mạng (bài 20)

```dart
final client = ToiNoiApiClient(baseUrl: baseUrl, interceptors: [SafeLogInterceptor(print)]);
client.dio.interceptors.add(RetryInterceptor(dio: client.dio)); // sau interceptor log

try {
  final quote = await guardApi(() => client.quote(request));
} on ApiFailure catch (failure) {
  switch (failure) { // không cần nhánh mặc định: sealed
    case NetworkFailure(): ...
    case TimeoutFailure(): ...
    case ServerFailure(): ...
    case ClientFailure(): ...
    case ParseFailure(): ...
  }
}
```

Test lỗi tầng mạng: `package:toi_noi_api/fault_injection.dart` có `FaultyAdapter(inner, script: [NetworkFault.refuse, null])`. Với `fake_async`, `hang` và `connectTimeout` chờ đúng hạn trên đồng hồ giả nên test không chờ thật. `test/real_socket_test.dart` đối chiếu với socket thật ở loopback (kết nối đứt giữa body, bắt tay TLS hỏng, cổng đóng, server im lặng).

Package import `dart:io` (nhận ra `HttpException`, `SocketException`, `TlsException`), nên chỉ dùng cho iOS và Android, không dùng cho Flutter web.
