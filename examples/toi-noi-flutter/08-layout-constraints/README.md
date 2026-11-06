# Lab 08: Layout và constraints, thẻ chuyến của Tới Nơi

Lab của bài **FL-08** trong loạt **Flutter qua Tới Nơi**: [Layout và constraints: Row, Column, Expanded và thẻ chuyến](../../../api/seed/content/flutter-layout-constraints-row-column-the-chuyen.md).

Lab dựng `TripCard`, thẻ chuyến nhận một `Trip` của `toi_noi_core`, rồi dùng test với kích thước cửa sổ giả (`tester.view.physicalSize`) để chứng minh: thẻ không overflow ở màn hình 320 px với địa chỉ dài; một bản viết sai có chủ đích thì overflow thật; `LayoutBuilder` đổi bố cục theo bề rộng thẻ được cấp. Bảng giá (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền 0 đến 100 km) là giả định để học; mọi con số cước trong lab do `calculateFare` của `toi_noi_core` tính.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/trip_card.dart` | `TripCard` (`LayoutBuilder`, `Row`, `Column`, `Expanded`, `Wrap`, `Stack`), `formatKm`, `statusColor` |
| `lib/constraint_demos.dart` | Ba widget nhỏ để đo bằng test: `TightParentDemo`, `FlexRatioRow`, `FlexFitRow` |
| `lib/broken_examples.dart` | Ca sai có chủ đích: `BrokenTripCard`, `ListInColumnBroken`, `ExpandedInHorizontalScrollBroken` và bản sửa `ListInColumnFixed` |
| `lib/sample_trips.dart` | `tripAt(status)`: dựng chuyến bằng `Trip.request` + `apply`; hai địa chỉ mẫu ngắn và dài (hư cấu) |
| `lib/main.dart` | `ToiNoiApp` và `TripCardGallery`: bốn thẻ mẫu |
| `test/trip_card_test.dart` | Dữ liệu từ `Trip`, 320 px không overflow, `LayoutBuilder` theo ngưỡng 600, `Stack` |
| `test/overflow_test.dart` | Tái hiện overflow và hai lỗi unbounded bằng `takeException` / gom lỗi |
| `test/constraints_test.dart` | Constraints chặt và lỏng, `Expanded` flex, `Flexible` so với `Expanded` |
| `test/gallery_test.dart` | Màn hình mẫu ở 320, 390, 768 px |
| `test/support.dart` | `setScreen`, `app`, `pumpCollectingErrors` |

## Kiểm chứng (không cần simulator)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/08-layout-constraints
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (32 test).

Test chạy dưới `flutter test`, nơi mọi chữ dùng font test mặc định (FlutterTest): mỗi ký tự là một ô vuông có cạnh bằng cỡ chữ. Chữ trong test vì vậy thường rộng hơn chữ thật, nên test hẹp là phép thử khắt khe; tuy vậy đo bằng font này không thay được việc nhìn thẻ trên máy thật với font thật.

## Chạy trên máy (tự thử)

Thư mục lab không chứa `android/` và `ios/`. Sinh runner một lần rồi chạy:

```bash
flutter create --platforms=ios,android --project-name toi_noi_08 --org vn.toinoi .
flutter run -d <id>
```

Muốn thấy overflow bằng mắt: trong `TripCardGallery` thay `TripCard` bằng `BrokenTripCard` (chỉ cần `trip` và `pickup`), chạy ở debug và thử địa chỉ `longAddress`: Flutter vẽ vạch vàng đen ở mép phải và in `A RenderFlex overflowed by ... pixels on the right` ra console.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_lints` 6.0.0, `toi_noi_core` 0.1.0 (path), Material 3.

## Chưa làm

- Chưa chạy trên iOS Simulator, Android emulator hay thiết bị thật: máy kiểm chưa có Android SDK và Xcode đầy đủ. Mức kiểm chứng là tự động (`flutter analyze` và `flutter test`).
- Chưa xử lý cỡ chữ hệ thống gấp đôi: test chỉ kiểm tới 1,5 lần ở 320 px. Accessibility và responsive đầy đủ thuộc bài 16.
- Chưa có địa chỉ trong `Trip` (domain chưa có): `TripCard` nhận địa chỉ qua tham số riêng.
- Chưa dùng `intl` cho tiền và số km (`formatKm` viết tay); chưa có theme riêng, localization và dark mode.
- Chưa có golden test; kiểm bố cục bằng kích thước thật của widget, golden thuộc bài 29.
