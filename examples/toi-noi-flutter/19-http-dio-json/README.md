# Lab 19: HTTP, dio và JSON, màn hình báo giá gọi fake API

Lab của bài **FL-19** trong loạt **Flutter qua Tới Nơi**: [HTTP, dio và JSON: gọi API báo giá của trip-service](../../../api/seed/content/flutter-http-dio-json-dto-goi-api-bao-gia.md).

Một màn hình nhỏ: chọn dịch vụ, kéo quãng đường, bấm **Báo giá** để gọi `POST /quotes`, bấm **Đặt chuyến** để gọi `POST /trips` với `Idempotency-Key`, bấm **Làm mới** để gọi `GET /trips/{id}`. State do Riverpod 3 giữ (`Notifier` cho form, `AsyncNotifier` cho báo giá và chuyến), mọi cuộc gọi đi qua `ToiNoiApiClient` của [`toi_noi_api`](../packages/toi_noi_api) và chạm tới [`tools/fake_api`](../tools/fake_api). Bảng giá (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền 0 đến 100 km) là minh hoạ; mọi con số cước do `calculateFare` của `toi_noi_core` tính.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/providers.dart` | `apiBaseUrlProvider`, `httpAdapterProvider`, `logSinkProvider`, `idempotencyKeyProvider`, `apiClientProvider`; `QuoteFormNotifier`, `QuoteController`, `BookingController` |
| `lib/error_text.dart` | `describeError`: câu tiếng Việt cho `ApiException`, `DtoFormatException`, `DioException` (bản tối thiểu, bài 20 làm đầy đủ) |
| `lib/quote_screen.dart` | Màn hình báo giá và đặt chuyến, `switch` trên `AsyncValue` |
| `lib/app.dart`, `lib/main.dart` | `ToiNoiApp` (theme của `toi_noi_ui`) và `main()` |
| `test/controllers_test.dart` | 27 test: provider thật + fake API trong tiến trình; response cũ bị bỏ, khoá idempotency, `flutter test` chặn HTTP |
| `test/quote_screen_test.dart` | 11 test widget: vòng chờ, lỗi 500 có mã hỗ trợ, đặt chuyến (ẩn nút sau khi đặt, Đặt chuyến khác), làm mới lỗi giữ thẻ chuyến, log không lộ khoá |
| `test/real_socket_without_binding_test.dart` | 1 test: chưa khởi tạo binding của flutter_test thì `dio` vẫn mở socket thật |
| `test/error_text_test.dart` | 5 test `describeError` |
| `test/retry_in_build_test.dart` | 1 test: `FutureProvider` gọi API trong `build` bị Riverpod 3 thử lại cả với 422 |
| `test/support.dart` | `Harness`: fake API + `InProcessAdapter` (từ `toi_noi_fake_api/testing.dart`) + log + bộ sinh khoá, `driver(id)` (client tài xế gắn header `x-fake-driver-id` bằng interceptor), dựng override cho provider |

## Kiểm chứng (không cần simulator, không cần mạng)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/19-http-dio-json
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (45 test). Cùng lệnh cho hai package đi kèm:

```bash
cd examples/toi-noi-flutter/packages/toi_noi_api && dart pub get && dart analyze && dart test   # 86 test của bài 19 (cộng test của bài 20)
cd examples/toi-noi-flutter/tools/fake_api && dart pub get && dart analyze && dart test           # 63 test
```

Mức kiểm chứng: **Kiểm chứng tự động**. Test không mở socket nào (adapter chạy fake API trong tiến trình); riêng một test của `toi_noi_api` mở socket loopback tạm để đối chứng với adapter thật.

## Chạy trên máy (tự thử)

Thư mục lab không chứa `android/` và `ios/`. Sinh runner một lần:

```bash
cd examples/toi-noi-flutter/19-http-dio-json
flutter create --platforms=ios,android --project-name toi_noi_19 --org vn.toinoi .
```

Mở một terminal chạy fake API, terminal khác chạy app:

```bash
cd examples/toi-noi-flutter/tools/fake_api && dart pub get && dart run bin/server.dart
cd examples/toi-noi-flutter/19-http-dio-json && flutter run -d <id>
# Android emulator: máy dev nằm ở 10.0.2.2
flutter run -d <id> --dart-define=API_BASE_URL=http://10.0.2.2:8787
```

Thử nhận chuyến từ phía tài xế rồi bấm **Làm mới** trong app (thay `<id>` bằng mã chuyến hiện trên màn hình):

```bash
curl -X POST http://localhost:8787/trips/<id>/accept \
  -H 'content-type: application/json' -H 'x-fake-driver-id: D-1' \
  -d '{"expectedVersion":0}'
```

Thử lỗi: chạy server với `--fail-first 2` để hai request đầu trả 500, hoặc `--latency-ms 2000` để thấy vòng chờ. Log của `SafeLogInterceptor` hiện ở console của `flutter run`.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_riverpod` 3.4.3, `dio` 5.11.1, `shelf` 1.4.3 (dev), `flutter_lints` 6.0.0, `toi_noi_core` 0.1.0, `toi_noi_ui` 0.1.0, `toi_noi_api` 0.1.0, `toi_noi_fake_api` 0.1.0 (path), Material 3.

## Chưa làm

- `logSinkProvider` chỉ in khi `kDebugMode` (`debugPrint` vẫn chạy ở bản release). Thông báo của `DtoFormatException` có thể chứa giá trị từ payload: đừng gửi nguyên văn lên crash report (bài 48).
- `Idempotency-Key` chỉ nằm trong RAM của `BookingController` và gắn với đúng form đã gửi. Lỗi mập mờ rồi đổi form thì sinh khoá mới và có thể ra hai chuyến; tắt app thì mất khoá. Lưu khoá bền cùng thao tác chờ gửi là việc của bài 46 (outbox).
- Chưa chạy trên iOS Simulator, Android emulator hay thiết bị thật; chưa kiểm cấu hình mạng native (cleartext HTTP tới máy dev, quyền mạng), việc này thuộc bài 25 và Phần 7.
- Retry, phân loại lỗi đầy đủ, timeout theo loại request, `connectivity_plus`: bài 20. `describeError` chỉ là bản tối thiểu; lỗi tầng mạng còn là `DioException` nguyên bản.
- Xác thực (OTP, JWT, refresh token): bài 24. Header `x-fake-driver-id` chỉ là chỗ đứng tạm.
- Realtime (WebSocket): bài 36. Trạng thái chuyến chỉ đổi khi người dùng bấm Làm mới.
- Cấu hình theo flavor: bài 25; địa chỉ máy chủ tạm lấy từ `--dart-define=API_BASE_URL`.
- Định dạng số theo locale (`intl`, bài 14): màn hình hiện `5.0 km` bằng `toStringAsFixed`; `Money.toString()` cho `25.500 VND`.
- Sinh DTO bằng `json_serializable` / `freezed`: bài 26.
