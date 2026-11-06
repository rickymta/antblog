# Lab 20: lỗi có phân loại, retry, timeout và mất mạng

Lab của bài **FL-20** trong loạt **Flutter qua Tới Nơi**: [Xử lý lỗi, retry, timeout và mất mạng: mạng di động không ổn định](../../../api/seed/content/flutter-xu-ly-loi-retry-timeout-connectivity-mang-di-dong.md).

Cùng màn hình báo giá và đặt chuyến của lab 19, nhưng mọi lỗi đã được phân loại thành `ApiFailure` (package [`toi_noi_api`](../packages/toi_noi_api) 0.2.0), request lặp lại được tự thử lại có backoff, và màn hình lỗi có nút hành động: **Thử lại**, **Mở cài đặt**. Connectivity qua `connectivity_plus` chỉ là gợi ý (banner và câu chữ), không bao giờ chặn request. Bảng giá (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền 0 đến 100 km) và mọi con số timeout, số lần thử là minh hoạ.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/providers.dart` | `NetworkConfig` (timeout, số lần thử, backoff, hạn tổng `maxElapsed`), `apiClientProvider` gắn `SafeLogInterceptor` rồi `RetryInterceptor`, `noProviderRetry`, `linkProvider`, `QuoteController`, `BookingController` (`retry()` làm lại đúng thao tác vừa lỗi); cả hai huỷ request đang bay khi form đổi hoặc provider bị bỏ |
| `lib/connectivity.dart` | `LinkType`, `linkTypeOf`, interface `ConnectivityMonitor`, `ConnectivityPlusMonitor` (bọc `connectivity_plus`), `linkUpdates` (nghe thay đổi trước rồi mới hỏi giá trị hiện tại) |
| `lib/system_settings.dart` | interface `SystemSettings` (nhãn nút + mở cài đặt), `AppSettingsOpener` (bọc `app_settings`; iOS mở trang cài đặt của chính app) |
| `lib/failure_text.dart` | `describeFailure`: `switch` vét cạn trên năm loại `ApiFailure`, trả câu chữ, mã hỗ trợ và tập hành động; `isCancellation` |
| `lib/failure_view.dart` | `OfflineBanner`, `FailureView` (nút Thử lại, Mở cài đặt) |
| `lib/quote_screen.dart`, `lib/app.dart`, `lib/main.dart` | màn hình, vỏ app, `ProviderScope(retry: noProviderRetry)` |
| `test/support.dart` | `Harness`: fake API + `FaultyAdapter` + connectivity giả + settings giả; khoảng chờ backoff ghi lại, không chờ thật; `runFake` chạy trên `fake_async` |
| `test/failure_text_test.dart` | 14 test: hành động và câu chữ theo từng loại lỗi, 408/429, huỷ request, nhãn nút theo nền tảng |
| `test/connectivity_test.dart` | 7 test: bảng ánh xạ `ConnectivityResult`, platform giả, thứ tự nghe/hỏi của `linkUpdates` |
| `test/controllers_test.dart` | 18 test: retry bên dưới controller, đúng 3 request, hạn tổng 30 giây trên đồng hồ giả, khoá idempotency khi đặt chuyến, huỷ request, 429 và Retry-After |
| `test/quote_screen_test.dart` | 18 test widget: Thử lại, Mở cài đặt, banner, connectivity khác Internet, reset và TLS, 429, timeout trên đồng hồ giả |
| `test/riverpod_retry_test.dart` | 4 test: retry mặc định của Riverpod nhân với interceptor (33 request), tắt thì còn 3 |

## Kiểm chứng (không cần simulator, không cần mạng)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/20-error-retry-connectivity
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (61 test). Package đi kèm:

```bash
cd examples/toi-noi-flutter/packages/toi_noi_api && dart pub get && dart analyze && dart test
```

Test không dùng thời gian thật: khoảng chờ backoff được tiêm (ghi lại rồi trả ngay), còn timeout chạy trên `fake_async` hoặc đồng hồ giả của `testWidgets`, nên ca "server treo, 3 lần quá hạn, dừng ở hạn tổng 30 giây" chạy trong chưa tới một giây thật. Mức kiểm chứng: **Kiểm chứng tự động**.

## Chạy trên máy (tự thử)

Thư mục lab không chứa `android/` và `ios/`. Sinh runner một lần:

```bash
cd examples/toi-noi-flutter/20-error-retry-connectivity
flutter create --platforms=ios,android --project-name toi_noi_20 --org vn.toinoi .
```

Mở một terminal chạy fake API, terminal khác chạy app:

```bash
cd examples/toi-noi-flutter/tools/fake_api && dart pub get && dart run bin/server.dart --fail-first 2
cd examples/toi-noi-flutter/20-error-retry-connectivity && flutter run -d <id>
# Android emulator: máy dev nằm ở 10.0.2.2
flutter run -d <id> --dart-define=API_BASE_URL=http://10.0.2.2:8787
```

Thử: `--fail-first 2` (hai request đầu lỗi 500, interceptor thử lại rồi thành công, người dùng chỉ thấy giá), `--fail-first 5` (hết lượt thử, hiện nút Thử lại), `--latency-ms 12000` (vượt `receiveTimeout` 10 giây, hiện "phản hồi quá chậm" sau tối đa 30 giây), tắt server (mất kết nối, hiện Thử lại và Mở cài đặt). Log của từng lần thử và từng lần chờ hiện ở console của `flutter run`.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_riverpod` 3.5.0 (ràng buộc `^3.4.3`), `dio` 5.11.1, `connectivity_plus` 7.3.2, `app_settings` 9.0.1, `fake_async` 1.3.3, `flutter_lints` 6.0.0, `toi_noi_api` 0.2.0, `toi_noi_core` 0.1.0, `toi_noi_ui` 0.1.0, `toi_noi_fake_api` 0.1.0 (path), Material 3.

## Chưa làm

- Chưa chạy trên iOS Simulator, Android emulator hay thiết bị thật: `connectivity_plus` và `app_settings` là plugin native, lab chỉ kiểm phần Dart qua interface và platform giả. Chưa kiểm cài đặt Gradle/Podfile của hai plugin (yêu cầu phiên bản nằm trong README của `connectivity_plus`), chưa thấy màn hình cài đặt nào thực sự mở ra. `AppSettingsType.wireless` chỉ hỗ trợ Android theo tài liệu plugin; trên iOS, đọc `AppSettingsPlugin.swift` của `app_settings` 9.0.1 cho thấy mọi loại không biết rơi vào `openSettingsURLString`, tức trang cài đặt của chính app, không có Wi-Fi (nhãn nút iOS nói đúng điều đó; chưa chạy trên thiết bị).
- Chưa kiểm cấu hình mạng native để app gọi được máy dev (cleartext HTTP, quyền mạng): bài 25 và Phần 7.
- Không hiện tiến độ "đang thử lại lần 2/3" trên giao diện (chỉ ghi log qua `onRetry`).
- Timeout là một bộ cho cả client; chưa có timeout riêng cho từng loại request. `Retry-After` chỉ đọc dạng số giây. `maxElapsed` không với tới lần gửi đầu (một body nhỏ giọt không bao giờ chạm `receiveTimeout`).
- Xác thực và refresh khi 401: bài 24. Realtime: bài 36. Cấu hình flavor: bài 25. Sinh DTO và `ApiFailure` bằng `freezed`: bài 26.
- Chưa có hàng đợi gửi lại khi có mạng trở lại (outbox, idempotency key lưu bền): bài 46.
