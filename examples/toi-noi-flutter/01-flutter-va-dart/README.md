# Lab 01: Flutter và Dart, từ source code đến ứng dụng Tới Nơi

Lab của bài **FL-01** trong loạt **Flutter qua Tới Nơi**: [Flutter và Dart: từ source code đến ứng dụng Tới Nơi chạy trên iOS và Android](../../../api/seed/content/flutter-va-dart-tu-source-code-den-ung-dung-toi-noi.md).

App có một màn hình in tiêu đề "Tới Nơi", quãng đường 5 km và cước minh hoạ 25.500 VND. Bảng giá (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền 0 đến 100 km) là giả định để học, không phải bảng giá thực tế.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/fare.dart` | `calculateFareVnd`: hàm thuần Dart, không phụ thuộc Flutter |
| `lib/main.dart` | `main()`, `ToiNoiApp`, `QuoteScreen`, `formatVnd` |
| `test/fare_test.dart` | 7 ca: 0, 2, 5, 2,001, 100 km, -1 km và 100,5 km |
| `test/widget_test.dart` | widget test màn hình báo giá và test `formatVnd` |
| `android/`, `ios/` | project nền tảng do `flutter create` sinh, giữ ở mức tối thiểu |

## Kiểm chứng (không cần simulator)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/01-flutter-va-dart
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (9 test).

## Chạy trên simulator hoặc emulator (tự thử)

```bash
flutter doctor
flutter devices            # liệt kê thiết bị đang chạy, lấy <id>
flutter emulators          # liệt kê emulator/simulator đã tạo
flutter emulators --launch <emulator-id>
flutter run -d <id>
```

Trong lúc `flutter run`: `r` là hot reload, `R` là hot restart, `q` thoát. Chạy iOS cần macOS với Xcode đầy đủ và CocoaPods; chạy Android cần Android SDK và một emulator. Không cần đủ cả hai để học.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_lints` 6.0.0, Material 3.

## Chưa làm

- Chưa chạy app trên iOS Simulator, Android emulator hay thiết bị thật trong lần kiểm này: máy kiểm chưa có Android SDK và Xcode đầy đủ. Mức kiểm chứng là tự động (`flutter analyze` và `flutter test`).
- Chưa có nhập liệu, đa ngôn ngữ, định dạng số bằng `intl`, theme riêng; các phần đó thuộc bài sau.
- Chưa build `flutter build apk` hay `flutter build ipa`.
