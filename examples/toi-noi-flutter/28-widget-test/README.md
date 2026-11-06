# Lab 28: widget test cho màn hình báo giá

Lab của bài **FL-28** trong loạt **Flutter qua Tới Nơi**: [Widget test: pumpWidget, finder, matcher và màn hình báo giá](../../../api/seed/content/flutter-widget-test-pumpwidget-finder-matcher-man-hinh-bao-gia.md).

Màn hình báo giá của app khách có đủ ba trạng thái (đang tải, lỗi, có dữ liệu), ô nhập quãng đường kiểm tra miền 0–100 km, nút "Đặt chuyến" nằm dưới phần cuộn và chuyển sang `/trip/:id` bằng `go_router`. Phần lớn lab là **test**: chúng dựng màn hình trong `ProviderScope(overrides: ...)`, điều khiển repository giả bằng `Completer`, và kiểm `pump`, `pumpAndSettle`, finder, `enterText`, `tap`, `ensureVisible`, `runAsync`, kích thước màn hình và điều hướng. Dùng [`toi_noi_core`](../packages/toi_noi_core) (`Trip`, `Money`, `calculateFare`) và [`toi_noi_ui`](../packages/toi_noi_ui) (theme, `MoneyText`, `TripStatusChip`). Bảng giá là minh hoạ; mọi con số cước do `calculateFare` tính hoặc đối chiếu với công thức (5 km → 25.500 VND, 2,001 km → 12.005 VND, 100 km → 453.000 VND).

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/quote.dart` | `Quote`: dịch vụ, quãng đường, cước (`Money`) |
| `lib/quote_repository.dart` | `QuoteRepository` (interface), `QuoteUnavailable`, `FakeQuoteRepository` có độ trễ |
| `lib/providers.dart` | `quoteRepositoryProvider`, `quoteRequestProvider`, `quoteProvider` (tắt retry tự động) |
| `lib/quote_screen.dart` | `QuoteScreen`: chip dịch vụ, ô quãng đường, ba trạng thái, bảng chi tiết, nút đặt, lớp phủ "đang đặt" |
| `lib/trip_screen.dart`, `lib/router.dart`, `lib/app.dart`, `lib/main.dart` | Màn hình chuyến, `createRouter`, `ToiNoiApp`, điểm vào |
| `test/support.dart` | `ControlledQuoteRepository` (Completer), `pumpQuoteScreen`, `pumpApp`, `usePhoneScreen` |
| `test/pump_test.dart` | `pump`, `pump(Duration)`, `pumpAndSettle`, hết hạn khi vòng quay chạy mãi, thiếu một nhịp |
| `test/finders_test.dart` | `byKey/byType/text/widgetWithText/descendant`, `findsNWidgets`, text với key |
| `test/input_test.dart` | `enterText`, ca biên 0 / 100 / 100,5 / NaN, phím Done, chip dịch vụ |
| `test/states_test.dart` | Ba trạng thái qua `overrideWithValue` và qua repository, nút Thử lại |
| `test/tap_test.dart` | `tap`, `warnIfMissed`, `ensureVisible`, lớp phủ chặn chạm thứ hai |
| `test/view_test.dart` | Kích thước 800x600 mặc định, font FlutterTest, tràn ngang và `takeException`, hệ số chữ, kích thước còn nguyên tới teardown |
| `test/navigation_test.dart` | go_router: đặt chuyến thành công và lỗi, Back, deep link, `go` và `push` |
| `test/run_async_test.dart` | `runAsync` với I/O thật, I/O bắt đầu trong cây widget không tự xong (lặp runAsync + pump) |
| `test/fixtures/quote_ride_5km.json` | Phản hồi mẫu cho test `runAsync` |

## Kiểm chứng (không cần simulator)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/28-widget-test
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (70 test). Mức kiểm chứng: **Kiểm chứng tự động**. Test không gọi mạng và không cần thiết bị; `flutter test` chạy với thư mục làm việc là gốc package nên test đọc được `test/fixtures/` bằng đường dẫn tương đối.

## Chạy trên máy (tự thử)

Thư mục lab không chứa `android/` và `ios/`. Sinh runner một lần rồi chạy:

```bash
flutter create --platforms=ios,android --project-name toi_noi_28 --org vn.toinoi .
flutter run -d <id>
```

Repository mặc định của app là bản giả trả báo giá sau 400 ms; không có backend thật.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_riverpod` 3.5.0, `go_router` 18.1.3, `flutter_lints` 6.0.0, `toi_noi_core` 0.1.0 và `toi_noi_ui` 0.1.0 (path), Material 3.

## Chưa làm

- Chưa có golden test và Semantics test (bài 29), chưa test Riverpod bằng `ProviderContainer` thuần hay `fakeAsync` (bài 30), chưa có integration test trên thiết bị (bài 31).
- Font trong test là FlutterTest (ký tự gần như hình vuông), nên mọi phép đo bề rộng chữ chỉ có giá trị tương đối; chưa kiểm giao diện với font thật của iOS, Android.
- Chưa chạy trên simulator, emulator hay máy thật; điều hướng chỉ được kiểm trong cây widget của test, không qua nút Back hệ thống hay deep link của hệ điều hành.
- Repository thật (`toi_noi_api`, bài 19) chưa nối vào lab này; `FakeQuoteRepository` chỉ tính cước bằng `calculateFare`.
