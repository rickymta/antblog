# Lab 18: Riverpod cho luồng đặt chuyến của Tới Nơi

Lab của bài **FL-18** trong loạt **Flutter qua Tới Nơi**: [Riverpod: Provider, Notifier và AsyncNotifier cho luồng đặt chuyến](../../../api/seed/content/flutter-riverpod-provider-notifier-asyncnotifier-dat-chuyen.md).

Một màn hình đặt chuyến nhỏ, mọi state nằm trong provider (không có `setState`): khách chọn dịch vụ và quãng đường, app lấy báo giá từ một repository giả, rồi đặt chuyến và xem chuyến vừa tạo. Dùng [`toi_noi_core`](../packages/toi_noi_core) (`Trip`, `Money`, `calculateFare`) và [`toi_noi_ui`](../packages/toi_noi_ui) (theme, `MoneyText`, `TripStatusChip`). Bảng giá (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền 0 đến 100 km) là minh hoạ; mọi con số cước do `calculateFare` tính.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/quote.dart` | `Quote` bất biến, có `==` |
| `lib/quote_repository.dart` | `QuoteRepository` (interface), `FakeQuoteRepository`, `QuoteUnavailable`, `TripNotFound` |
| `lib/trip_draft.dart` | `TripDraft` và `TripDraftNotifier` (`Notifier` đồng bộ), `tripDraftProvider` |
| `lib/providers.dart` | `quoteRepositoryProvider` (`Provider`), `quoteProvider` (`FutureProvider.autoDispose`), `tripByIdProvider` (`FutureProvider.autoDispose.family`), `PlaceTripNotifier` (`AsyncNotifier`) và `placeTripProvider`, `retryTransientOnce` |
| `lib/booking_screen.dart` | `BookingScreen` (`ref.watch`, `ref.listen`, `ref.read`, `switch` trên `AsyncValue`), `ServicePicker`, `DistanceStepper` |
| `lib/trip_screen.dart` | `TripScreen` đọc `tripByIdProvider(id)` |
| `lib/error_text.dart`, `lib/app.dart`, `lib/main.dart` | Câu lỗi hiển thị, `ToiNoiApp`, `main()` với `ProviderScope` |
| `test/draft_notifier_test.dart` | 8 test: `Notifier` qua `ProviderContainer`, lọc cập nhật bằng `==`, thiếu `==`, sửa tại chỗ |
| `test/quote_provider_test.dart` | 18 test: phụ thuộc bản nháp, `invalidate` sau lỗi, phản hồi sai thứ tự, auto-dispose, `keepAlive`, `onDispose`, watch/read/listen |
| `test/retry_test.dart` | 19 test: retry mặc định của Riverpod 3 (đồng hồ giả), retry tự viết, `invalidate` sau khi hết lượt, chính sách của lab |
| `test/place_trip_test.dart` | 8 test: `AsyncNotifier`, `ref.mounted` (cả sau khi tính lại), `ProviderException` |
| `test/trip_by_id_test.dart` | 7 test: family, auto-dispose theo từng đối số, `overrideWith`, `overrideWithValue` cho một id |
| `test/fake_repository_test.dart` | 6 test: bản giả tính đúng cước, mã chuyến, độ trễ |
| `test/booking_screen_test.dart` | 16 test: ba trạng thái, `ref.listen` và SnackBar, thử lại, đặt chuyến, `ProviderScope(overrides:)` |
| `test/ref_usage_test.dart` | 5 test: `select`, `watch` và `read` trong `build`, `ref.listen` sai chỗ, thiếu `ProviderScope` |
| `test/support.dart` | `StubQuoteRepository`, `pumpApp`, dữ liệu mẫu |

## Kiểm chứng (không cần simulator)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/18-riverpod
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (87 test). Mức kiểm chứng: **Kiểm chứng tự động**. Các test về retry dùng `fakeAsync` nên thời gian chờ 38 giây chạy trong vài mili giây; test không gọi mạng.

## Chạy trên máy (tự thử)

Thư mục lab không chứa `android/` và `ios/`. Sinh runner một lần rồi chạy:

```bash
flutter create --platforms=ios,android --project-name toi_noi_18 --org vn.toinoi .
flutter run -d <id>
```

Repository mặc định trong `main()` là bản giả có độ trễ 400 ms và báo `QuoteUnavailable` khi quãng đường trên 30 km: bấm `+` quá 30 km để thấy spinner "đang thử lại" rồi SnackBar và nút "Thử lại".

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_riverpod` 3.4.3 (kéo theo `riverpod` 3.4.3), `fake_async` 1.3.3 (dev), `flutter_lints` 6.0.0, `toi_noi_core` 0.1.0 và `toi_noi_ui` 0.1.0 (path), Material 3. Không dùng code generator (`riverpod_generator`, `riverpod_lint`): thuộc bài 26.

## Chưa làm

- Chưa gọi HTTP: `FakeQuoteRepository` sống trong bộ nhớ; client `dio`, DTO và `tools/fake_api` thuộc bài 19. Hai bản sau chỉ khác nhau ở provider `quoteRepositoryProvider`.
- Chưa có phân loại lỗi mạng, timeout, `connectivity_plus`, backoff cho request idempotent (bài 20). `retryTransientOnce` chỉ là chính sách tối thiểu.
- Chưa có mutation (`package:riverpod/experimental/mutation.dart` còn là API thử nghiệm), `StreamProvider`, persist offline, `ProviderObserver`, scoped provider với `dependencies:`.
- Chưa chạy trên iOS Simulator, Android emulator hay thiết bị thật.
- `Quote` và `TripDraft` viết tay, chưa dùng `freezed` (bài 26).
