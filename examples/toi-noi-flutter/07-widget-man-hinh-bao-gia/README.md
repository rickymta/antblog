# Lab bài 07: Widget, cây widget và hot reload

Lab của bài **FL-07** trong loạt **Flutter qua Tới Nơi**: [Widget, cây widget và hot reload: màn hình báo giá đầu tiên](../../../api/seed/content/flutter-widget-cay-widget-hot-reload-man-hinh-bao-gia.md). Đây là bài Flutter UI đầu tiên, mở đầu Phần 2.

App có một màn hình báo giá: `ToiNoiApp` dựng `MaterialApp` (theme Material 3), `QuoteScreen` nhận `distanceKm` và hiện cước từ `calculateFare` của `toi_noi_core`. Mặc định 5 km cho 25.500 VND. Bảng giá (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền 0 đến 100 km) là giả định để học, không phải bảng giá thực tế.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/main.dart` | `main()` gọi `runApp(const ToiNoiApp())` |
| `lib/toi_noi_app.dart` | `ToiNoiApp`: `StatelessWidget` dựng `MaterialApp` và theme |
| `lib/quote_screen.dart` | `QuoteScreen` (`Scaffold`, `AppBar`, `Padding`, `Column`, `Icon`, `Text`, `Container`) và `formatKm` |
| `test/quote_screen_test.dart` | widget test: 5 km thành 25.500 VND, các ca 0, 2, 2,001, 100 km, ngoài miền, `ToiNoiApp`, `formatKm` (16 test) |
| `test/widget_tree_test.dart` | Widget/Element/RenderObject, element được giữ khi dựng lại, `reassembleApplication`, `build` ném lỗi (9 test) |

`QuoteScreen` phụ thuộc `toi_noi_core` qua path dependency `../packages/toi_noi_core`; không có công thức cước nào được viết lại trong lab.

## Kiểm chứng (không cần simulator)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/07-widget-man-hinh-bao-gia
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (25 test).

## Chạy trên simulator hoặc emulator (tự thử)

Repo không commit thư mục `android/` và `ios/`. Sinh runner một lần trong thư mục lab, rồi chạy:

```bash
flutter create --platforms=ios,android --project-name toi_noi_07 --org vn.toinoi .
flutter run -d <id>
```

Lệnh `flutter create` trong thư mục đã có `pubspec.yaml` chỉ thêm file còn thiếu (đã kiểm: `pubspec.yaml` và `lib/main.dart` không đổi), nhưng nó thêm `test/widget_test.dart` mẫu của bộ khung counter, file này không biên dịch được với lab. Xoá nó: `rm test/widget_test.dart`. Khi `flutter run` đang chạy: `r` hot reload, `R` hot restart, `q` thoát. Thử sửa chữ trong `QuoteScreen` rồi bấm `r`; sau đó đổi `main()` thành `runApp(const ToiNoiApp(distanceKm: 10))` và so sánh `r` với `R` (theo tài liệu hot reload, `main()` không chạy lại khi bấm `r`). Chạy iOS cần macOS với Xcode đầy đủ và CocoaPods; chạy Android cần Android SDK và emulator.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_lints` 6.0.0, Material 3.

## Chưa làm

- Chưa chạy app trên iOS Simulator, Android emulator hay thiết bị thật: máy kiểm chưa có Android SDK và Xcode đầy đủ. Hot reload và hot restart chỉ được mô tả theo tài liệu chính thức và mô phỏng phần dựng lại bằng `reassembleApplication` trong test; chưa quan sát trên `flutter run` thật.
- Chưa có `StatefulWidget`, nhập liệu, điều hướng, theme riêng, định dạng số bằng `intl`; các phần đó thuộc bài sau.
- `formatKm` chưa theo locale.
