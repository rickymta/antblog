# Changelog

## 0.1.0

- Bản đầu tiên, tạo ở bài 13 của loạt Flutter qua Tới Nơi.
- `ToiNoiTheme.light()` / `ToiNoiTheme.dark()`: `ThemeData` Material 3 sinh từ cùng một seed (`ToiNoiTheme.seed`) bằng `ColorScheme.fromSeed`; `TextTheme` gộp lên mặc định; component theme cho `FilledButton`, `Card`, `InputDecoration`; đăng ký `ToiNoiColors`. Theme được cache theo (độ sáng, `defaultTargetPlatform`): cùng nền tảng trả về cùng đối tượng, đổi `debugDefaultTargetPlatformOverride` thì `ThemeData.platform` đổi theo.
- `ToiNoiColors` (`ThemeExtension`): màu `success`, `warning`, `danger`, mỗi màu có bốn vai trò (màu chính, `on...`, `...Container`, `on...Container`); `light`, `dark`, `of(context)`, `copyWith`, `lerp`, `==`. Bộ `danger` đúng bằng `error`, `onError`, `errorContainer`, `onErrorContainer` của `ColorScheme`.
- `ToiNoiSpacing`: hằng khoảng cách `xs` 4, `sm` 8, `md` 16, `lg` 24, `xl` 32.
- `ToiNoiRadius`: hằng bo góc `button` 12, `input` 12, `card` 16.
- `TripStatusChip`: hiển thị một `TripStatus` của `toi_noi_core` (đủ 8 trạng thái, `switch` vét cạn); tham số tuỳ chọn `label` thay nhãn mặc định (cho bản dịch).
- `MoneyText`: hiển thị một `Money` bằng chữ số đều độ rộng; màu chữ kế thừa từ `DefaultTextStyle` (onPrimary khi nằm trong `FilledButton`); tham số tuỳ chọn `formatter` thay `Money.toString()` (cho định dạng theo locale).
