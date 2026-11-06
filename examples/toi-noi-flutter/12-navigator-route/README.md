# Lab 12: Navigator, route và truyền dữ liệu

Lab của bài **FL-12** trong loạt **Flutter qua Tới Nơi**: [Navigator, route và truyền dữ liệu: từ báo giá sang xác nhận chuyến](../../../api/seed/content/flutter-navigator-route-truyen-du-lieu-xac-nhan-chuyen.md).

Luồng của app: `QuoteScreen` (báo giá 5 km, 25.500 VND) mở `ConfirmTripScreen` bằng `Navigator.push<Trip>`. Màn hình xác nhận nhận `Quote` qua constructor, cho chọn phương thức trả tiền bằng `showModalBottomSheet`, chặn back khi đã đổi lựa chọn bằng `PopScope` và hộp thoại xác nhận, rồi trả `Trip?` khi đóng. Bảng giá (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền 0 đến 100 km), địa chỉ và mã chuyến là số liệu minh hoạ.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/quote.dart` | `Quote` (dữ liệu truyền đi, cước tính bằng `calculateFare` của `toi_noi_core`) và `demoQuote` |
| `lib/quote_screen.dart` | `QuoteScreen`: `push<Trip>`, kiểm `mounted` sau `await` |
| `lib/confirm_trip_screen.dart` | `ConfirmTripScreen`, `PopScope<Trip>`, `askToLeave` (kiểm `context.mounted`) |
| `lib/payment_method.dart`, `lib/payment_method_sheet.dart` | `PaymentMethod` và bottom sheet trả về `PaymentMethod?` |
| `lib/routes.dart` | `AppRoutes`, `onGenerateAppRoute` (named route có tham số), màn hình lỗi |
| `lib/app.dart`, `lib/main.dart` | `ToiNoiApp` (bảng `routes`, `onGenerateRoute`, `onUnknownRoute`) và `main()` |
| `lib/clock.dart` | `Clock` để test cố định `requestedAt` |
| `test/navigation_test.dart` | 17 test: truyền `Quote`, push/pop có kiểu, bottom sheet, `PopScope`, `mounted` |
| `test/named_route_test.dart` | 6 test: giới hạn của named route |
| `test/ios_swipe_test.dart` | 2 test: vuốt từ mép trái trên iOS khi `canPop` đúng và sai |

Trong lab này `Quote` và `PaymentMethod` là kiểu cục bộ, vì `toi_noi_core` chưa có hai kiểu đó. `Trip` chưa có trường phương thức trả tiền, nên lựa chọn thanh toán chỉ tồn tại trong màn hình xác nhận.

## Kiểm chứng (không cần simulator)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/12-navigator-route
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (25 test). Mức: **Kiểm chứng tự động**.

## Chạy trên máy (tự thử)

Lab chỉ commit `pubspec.yaml`, `analysis_options.yaml`, `lib/` và `test/`. Muốn chạy trên simulator hoặc emulator, sinh runner một lần:

```bash
flutter create --platforms=ios,android --project-name toi_noi_12 --org vn.toinoi .
flutter run
```

`android/` và `ios/` đã nằm trong `.gitignore` của lab. Chạy iOS cần macOS với Xcode đầy đủ và CocoaPods; chạy Android cần Android SDK.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_lints` 6.0.0, Material 3, `toi_noi_core` 0.1.0 (path).

## Chưa làm

- Chưa chạy trên iOS Simulator, Android emulator hay thiết bị thật: máy kiểm chưa có Android SDK và Xcode đầy đủ. Hành vi back của hệ thống được mô phỏng bằng `tester.pageBack()`, `tester.binding.handlePopRoute()` và thao tác vuốt trong test; predictive back của Android 14 trở lên chưa kiểm.
- Chưa có `go_router`, deep link, `Navigator 2.0`/`Router`: để dành cho bài 23.
- Chưa gửi chuyến lên API; `Trip.request` chỉ tạo đối tượng cục bộ. Chưa lưu phương thức thanh toán vào `Trip`.
- Chưa có theme riêng, localization hay animation chuyển trang tuỳ chỉnh (bài 13 trở đi).
