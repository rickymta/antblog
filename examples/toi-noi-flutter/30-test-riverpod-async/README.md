# Lab 30: kiểm thử Riverpod, async và Stream

Lab của bài **FL-30** trong loạt **Flutter qua Tới Nơi**: [Kiểm thử Riverpod, async và Stream: ProviderContainer, fakeAsync, fake clock](../../../api/seed/content/flutter-test-riverpod-async-stream-fake-async-providercontainer.md).

Ba mảnh logic bất đồng bộ của Tới Nơi và bộ test cho chúng: báo giá có hạn hiệu lực (`FutureProvider` gọi fake API), đếm ngược (hạn báo giá và hạn tài xế nhận cuốc) và vị trí tài xế (`StreamProvider` có timeout). Test dùng `ProviderContainer.test`, `container.listen`, `fake_async`, `Clock` của package `clock`, `emitsInOrder` và một widget test để đối chiếu với `testWidgets`. Mọi con số (hạn báo giá 2 phút, hạn nhận cuốc 15 giây, 10 giây không có vị trí, lịch thử lại 0,5/1/2 giây) và bảng giá (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền 0 đến 100 km) là minh hoạ; cước do fake API tính bằng `calculateFare`.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/quote_providers.dart` | `ValidQuote` (báo giá + `expiresAt`, `isExpired` đọc `clock.now()`), `apiClientProvider`, `quoteProvider` (`FutureProvider.autoDispose.family`, khoá là record) |
| `lib/retry_policy.dart` | `retryTransient`: chỉ thử lại `ApiFailure.retryable`, tối đa 3 lần, 0,5/1/2 giây |
| `lib/countdown.dart` | `remainingUntil` (Stream tính từ hạn, không đếm tick), `countdownProvider`, `offerDeadline`, `formatRemaining` |
| `lib/driver_position.dart` | `DriverPosition`, `DriverLocationSource`, `driverPositionProvider` (`StreamProvider` có `timeout`, tắt retry) |
| `lib/countdown_text.dart` | `CountdownText`, widget để thử với `testWidgets` |
| `test/support.dart` | `ApiHarness` (fake API trong tiến trình + `FaultyAdapter`), `record`, `stateStream`, `FakeLocationSource` |
| `test/quote_provider_test.dart` | 10 test: `.future`, chuỗi state qua `listen`, autoDispose cần listener |
| `test/retry_test.dart` | 8 test: tắt retry, retry trên đồng hồ giả, retry mặc định gọi 422 tới 11 lần |
| `test/clock_test.dart` | 11 test: `withClock`, `fakeAsync(initialTime:)`, ca biên hết hạn báo giá |
| `test/countdown_test.dart` | 11 test: đếm ngược trên đồng hồ giả, máy ngủ, huỷ Timer, `countdownProvider` |
| `test/driver_position_test.dart` | 12 test: `emitsInOrder`, timeout, retry mặc định với Stream, huỷ subscription khi dispose |
| `test/fake_async_pitfalls_test.dart` | 9 test: microtask và Timer, Timer ngoài zone, I/O thật, Timer còn sót |
| `test/countdown_widget_test.dart` | 7 test: `testWidgets` có sẵn FakeAsync, `clock` và `DateTime.now()`, `runAsync` |

## Kiểm chứng (không cần simulator, không cần mạng)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/30-test-riverpod-async
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (68 test). Mức kiểm chứng: **Kiểm chứng tự động**. Không test nào chờ thời gian thật hay gọi mạng: request đi qua `InProcessAdapter` của `toi_noi_fake_api`, thời gian đi bằng `fake_async`.

Đã kiểm với Flutter 3.47.4 / Dart 3.13.3: `flutter_riverpod` 3.5.0 (ràng buộc `^3.4.3`), `fake_async` 1.3.3, `clock` 1.1.3, `dio` 5.11.1.

## Chạy trên máy (tự thử)

Lab chỉ có logic và test, không có `main.dart`. Muốn nhúng vào app, bọc `ProviderScope(retry: retryTransient, overrides: [...])` và ghi đè `apiClientProvider` bằng client thật; thư mục lab không chứa `android/`, `ios/`. Nếu muốn sinh runner: `flutter create --platforms=ios,android --project-name toi_noi_30 --org vn.toinoi .`

## Chưa làm

- Nguồn vị trí thật (WebSocket, reconnect, backoff): bài 36. Ở đây chỉ có `StreamController` giả.
- Test tích hợp trên thiết bị: bài 31.
- `mocktail` để thay `FakeLocationSource`: bài 27 đã dùng; lab này chọn fake tự viết để thấy rõ `onCancel`.
- Test cho `riverpod_generator` và `ProviderObserver` ghi log: chưa làm.
