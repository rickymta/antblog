# Lab 16: Accessibility, responsive và adaptive, app Tới Nơi cuối Phần 2

Lab của bài **FL-16** trong loạt **Flutter qua Tới Nơi**: [Accessibility, responsive và adaptive: một giao diện cho iOS và Android](../../../api/seed/content/flutter-accessibility-responsive-adaptive-ios-android.md).

Lab ghép các màn hình của Phần 2 thành một app nhỏ dùng [`toi_noi_core`](../packages/toi_noi_core) và [`toi_noi_ui`](../packages/toi_noi_ui) (theme, `MoneyText`, `TripStatusChip`): báo giá (chọn dịch vụ, kéo quãng đường, cước, đặt chuyến), lịch sử chuyến, chi tiết chuyến có huỷ, cài đặt. Test chứng minh nhãn `Semantics` đọc được cho số tiền và trạng thái, ba guideline `androidTapTargetGuideline`, `iOSTapTargetGuideline`, `labeledTapTargetGuideline` (và `textContrastGuideline`), cỡ chữ gấp đôi không overflow, `SafeArea` tránh tai thỏ, bố cục đổi theo bề rộng cửa sổ, và khác biệt iOS/Android có chủ đích (`Theme.of(context).platform`, hộp thoại, cử chỉ quay lại, vật lý cuộn). Bảng giá (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền 0 đến 100 km) là minh hoạ; mọi con số cước do `calculateFare` tính.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/window_class.dart` | `WindowClass` compact / medium / expanded, ngưỡng 600 và 840 |
| `lib/adaptive_shell.dart` | Khung app: `NavigationBar`, `NavigationRail`, hai cột; giữ danh sách chuyến |
| `lib/quote_screen.dart` | Báo giá: `ChoiceChip`, `Slider` có nhãn, nút ghim đáy trong `SafeArea` |
| `lib/history_screen.dart`, `lib/trip_tile.dart` | Danh sách và thẻ chuyến (một nút, nhãn đọc trọn câu, `Wrap`) |
| `lib/trip_detail.dart` | `TripDetailView` (dùng cả trang riêng lẫn cột phải) và `TripDetailPage` |
| `lib/platform_adaptive.dart` | `isApplePlatform`, `confirmCancelTrip` (`showAdaptiveDialog`) |
| `lib/settings_screen.dart` | `SwitchListTile.adaptive` |
| `lib/accessible.dart` | `spokenMoney`, `spokenTrip`, `AccessibleMoney`, `AccessibleStatus` |
| `lib/broken_examples.dart` | Ca sai có chủ đích để chứng minh guideline bắt được lỗi |
| `lib/sample_trips.dart`, `lib/app.dart`, `lib/main.dart` | Chuyến mẫu dựng bằng `Trip.request` + `apply`; `ToiNoiApp`; `main()` |
| `test/semantics_test.dart` | 12 test: câu đọc, nhãn tiền và trạng thái, thẻ chuyến, slider, live region |
| `test/guidelines_test.dart` | 21 test: app thật qua ba guideline ở 3 bề rộng x 3 đích, đo riêng thanh điều hướng (guideline bỏ qua node sát mép), contrast sáng/tối, ca sai |
| `test/text_scale_test.dart` | 13 test: cỡ chữ x1 đến x3 ở 320 px, chip xuống dòng, ca chiều cao cố định |
| `test/safe_area_test.dart` | 9 test: `view.padding`, AppBar, NavigationBar, xoay ngang, ca thiếu `SafeArea` |
| `test/breakpoint_test.dart` | 12 test: ngưỡng, ba bố cục, hai cột, giữ state khi đổi cỡ cửa sổ |
| `test/platform_test.dart` | 18 test: `Theme.platform`, hộp thoại, công tắc, back gesture, vật lý cuộn |
| `test/support.dart` | `setScreen`, `setTextScale`, `setInsets`, `withPlatform`, `systemBack` |

## Kiểm chứng (không cần simulator)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/16-a11y-responsive-adaptive
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (85 test). Mức kiểm chứng: **Kiểm chứng tự động**. Test chạy dưới `flutter test` với font kiểm thử của `flutter test` (phần lớn ký tự là ô vuông bằng cỡ chữ) và mặc định `TargetPlatform.android`; mỗi ca iOS đặt nền tảng bằng `TargetPlatformVariant` hoặc `withPlatform` (luôn trả `debugDefaultTargetPlatformOverride` về `null` trong thân test).

## Chạy trên máy (tự thử)

Thư mục lab không chứa `android/` và `ios/`. Sinh runner một lần rồi chạy:

```bash
flutter create --platforms=ios,android --project-name toi_noi_16 --org vn.toinoi .
flutter run -d <id>
```

Thử trên máy thật: bật VoiceOver (iOS) hoặc TalkBack (Android) rồi vuốt qua màn báo giá; tăng cỡ chữ hệ thống lên mức lớn nhất; xoay ngang máy có tai thỏ; so hộp thoại huỷ chuyến giữa hai hệ điều hành.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_lints` 6.0.0, `flutter_localizations` (SDK), `toi_noi_core` 0.1.0 và `toi_noi_ui` 0.1.0 (path), Material 3.

## Chưa làm

- Chưa chạy trên iOS Simulator, Android emulator hay thiết bị thật; chưa nghe bằng VoiceOver/TalkBack. Test kiểm cây semantics (nhãn, cờ, thao tác), không kiểm cách trình đọc màn hình thật phát âm câu.
- Chưa có màn nhập điểm đón và điểm đến (form, bài 11) và chưa có song ngữ vi/en (bài 14).
- Chưa có localization đầy đủ: `MaterialApp` chỉ gắn `Locale('vi')` và `GlobalMaterialLocalizations.delegates` (gồm Material, Widgets, Cupertino) để chuỗi hệ thống (nút Back, "Tab 1 trong tổng số 3") bằng tiếng Việt; chuỗi của app còn viết cứng, `intl` thuộc bài 14.
- Chưa có reduce motion, bàn phím rời và focus order, `SemanticsService.announce`, chế độ độ tương phản cao của hệ điều hành.
- Đổi cỡ cửa sổ từ expanded sang compact lúc đang xem cột chi tiết thì cột đó biến mất chứ không tự đẩy thành trang riêng.
- State (danh sách chuyến, đích đang mở) giữ bằng `setState` ở `AdaptiveShell`; không lưu, không gọi mạng. Quản lý state thuộc Phần 3.
