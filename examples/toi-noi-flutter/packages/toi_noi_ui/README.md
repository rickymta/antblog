# toi_noi_ui

Design system dùng chung của **Tới Nơi**, tạo ở bài **FL-13** của loạt **Flutter qua Tới Nơi**: [Theme Material 3, màu, chữ và dark mode: design system Tới Nơi](../../../../api/seed/content/flutter-theme-material-3-dark-mode-design-system-toi-noi.md). Package nội bộ (`publish_to: none`), phụ thuộc `toi_noi_core` bằng path.

## API công khai

Chỉ những tên export ở `lib/toi_noi_ui.dart`:

| Tên | Vai trò |
|---|---|
| `ToiNoiTheme.light()`, `ToiNoiTheme.dark()`, `ToiNoiTheme.seed` | `ThemeData` Material 3 sinh từ cùng một seed; có `TextTheme` và component theme cho `FilledButton`, `Card`, `InputDecoration` |
| `ToiNoiColors` | `ThemeExtension`: màu `success`, `warning`, `danger` (mỗi màu bốn vai trò; `danger` bằng bộ `error` của scheme); `light`, `dark`, `of(context)`, `copyWith`, `lerp` |
| `ToiNoiSpacing` | Hằng khoảng cách `xs` 4, `sm` 8, `md` 16, `lg` 24, `xl` 32 |
| `ToiNoiRadius` | Hằng bo góc `button` 12, `input` 12, `card` 16 |
| `TripStatusChip` | Chip cho một `TripStatus` của `toi_noi_core`, đủ 8 trạng thái; `label` tuỳ chọn thay nhãn mặc định |
| `MoneyText` | Hiển thị một `Money` bằng chữ số đều độ rộng; màu kế thừa từ `DefaultTextStyle`; `formatter` tuỳ chọn thay `Money.toString()` |

Dùng trong app:

```dart
MaterialApp(
  theme: ToiNoiTheme.light(),
  darkTheme: ToiNoiTheme.dark(),
  themeMode: ThemeMode.system,
)
```

```yaml
dependencies:
  toi_noi_ui:
    path: ../packages/toi_noi_ui
```

## Kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/packages/toi_noi_ui
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (85 test: `ToiNoiColors` 22, theme 21, `TripStatusChip` 25, `MoneyText` 9, `textContrastGuideline` 7, `ToiNoiSpacing` 1). Mức: **Kiểm chứng tự động**.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_lints` 6.0.0, Material 3, `toi_noi_core` 0.1.0 (path).

## Chưa làm

- Chưa dùng font tuỳ chỉnh (bài 14), chưa localization và định dạng tiền bằng `intl` (bài 14); `MoneyText` mặc định dùng `Money.toString()`, định dạng khác truyền qua `formatter`.
- Chưa có nhãn `Semantics` riêng cho `MoneyText`, text scale lớn và adaptive iOS/Android (bài 16).
- Chưa có theme tương phản cao (`highContrastTheme`) và `Cupertino` theme.
- Chưa chạy trên thiết bị: màu và chữ chỉ được kiểm bằng test, chưa nhìn trên màn hình thật.
