# Lab 13: Theme Material 3, màu, chữ và dark mode

Lab của bài **FL-13** trong loạt **Flutter qua Tới Nơi**: [Theme Material 3, màu, chữ và dark mode: design system Tới Nơi](../../../api/seed/content/flutter-theme-material-3-dark-mode-design-system-toi-noi.md).

Lab là app demo dùng package [`toi_noi_ui`](../packages/toi_noi_ui): một màn hình "bảng mẫu" cho thấy thẻ báo giá (`Card`, `MoneyText`, `FilledButton`), ô nhập, 8 `TripStatusChip`, các ô màu của `ColorScheme` và `ToiNoiColors`, các kiểu chữ. `MaterialApp` dùng `ThemeMode.system`; một `SegmentedButton` cho ghi đè Sáng/Tối rồi quay lại Hệ thống. Bảng giá (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt) là minh hoạ.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/app.dart` | `ThemeGalleryApp`: giữ `ThemeMode`, truyền `ToiNoiTheme.light()` và `dark()` cho `MaterialApp` |
| `lib/gallery_screen.dart` | `GalleryScreen` và các khối: báo giá, ô nhập, chip, ô màu, kiểu chữ |
| `lib/main.dart` | `main()` |
| `test/theme_mode_test.dart` | 9 test: `ThemeMode.system` theo `platformBrightness`, ghi đè bằng bộ chọn |
| `test/gallery_test.dart` | 6 test: 8 chip, "5 km", 25.500 VND, component theme, cả bảng mẫu qua `textContrastGuideline` (viewport 800x3000, sáng/tối) |
| `test/support.dart` | Hàm phụ: đặt `platformBrightness`, đọc độ sáng của theme |

## Kiểm chứng (không cần simulator)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/13-theme-design-system
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (15 test). Package dùng chung kiểm riêng (85 test), xem README của `toi_noi_ui`. Mức: **Kiểm chứng tự động**.

## Chạy trên máy (tự thử)

Lab chỉ commit `pubspec.yaml`, `analysis_options.yaml`, `lib/` và `test/`. Muốn chạy trên simulator hoặc emulator, sinh runner một lần:

```bash
flutter create --platforms=ios,android --project-name toi_noi_13 --org vn.toinoi .
flutter run
```

Trong lúc app chạy, đổi chế độ Sáng/Tối trong phần cài đặt của simulator, emulator hoặc điện thoại để thấy `ThemeMode.system` đổi theo. `android/` và `ios/` đã nằm trong `.gitignore` của lab.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_lints` 6.0.0, Material 3, `toi_noi_core` 0.1.0 và `toi_noi_ui` 0.1.0 (path).

## Chưa làm

- Chưa chạy trên iOS Simulator, Android emulator hay thiết bị thật: máy kiểm chưa có Android SDK và Xcode đầy đủ. Việc đổi độ sáng của hệ thống được mô phỏng bằng `platformBrightnessTestValue`. Chưa thử đổi chế độ Sáng/Tối trên máy thật hay trình giả lập.
- Chưa lưu lựa chọn Sáng/Tối giữa các lần mở app (cần lưu trữ cục bộ, bài 22).
- Chưa có font tuỳ chỉnh, localization, animation đổi theme tuỳ chỉnh, `highContrastTheme`.
