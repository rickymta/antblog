# Lab 09: ListView, Sliver và cuộn, danh sách lịch sử chuyến

Lab của bài **FL-09** trong loạt **Flutter qua Tới Nơi**: [ListView, Sliver và cuộn: danh sách lịch sử chuyến](../../../api/seed/content/flutter-listview-sliver-scroll-lich-su-chuyen.md).

App có một màn hình lịch sử chuyến chạy trên dữ liệu giả (47 chuyến, không có mạng). Bảng giá (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền 0 đến 100 km) là giả định để học, không phải bảng giá thực tế. Lab dùng package `toi_noi_core` của bài 06 qua path dependency.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/trip_history_source.dart` | `TripHistorySource` (giao diện phân trang), `TripPage`, `FakeTripHistorySource` (dữ liệu giả, có độ trễ, đếm số lần gọi, `prepend`) |
| `lib/trip_history_screen.dart` | `TripHistoryScreen`: `CustomScrollView` + `SliverAppBar` + `SliverList.separated`, `RefreshIndicator`, `ScrollController` tải thêm cuối trang |
| `lib/trip_history_tile.dart` | `TripHistoryTile` (có state mở/gập) và `formatTripTime` |
| `lib/recent_trips_list.dart` | `RecentTripsList`: `ListView.separated` đơn giản nhất |
| `lib/main.dart` | `main()` và `ToiNoiApp` |
| `test/support.dart` | `ManualSource` (test tự quyết lúc nào nguồn trả lời hoặc lỗi), `loadTrips`, `app` |
| `test/lazy_build_test.dart` | `ListView.builder` chỉ build phần tử trong vùng nhìn (đếm số lần build), nhảy xa với và không có `itemExtent` (412 so với 28 lần build) |
| `test/refresh_physics_test.dart` | danh sách ngắn kéo để làm mới: không controller thì được, có controller mà thiếu `AlwaysScrollableScrollPhysics` thì không (Android và iOS) |
| `test/keys_test.dart` | đảo danh sách: không key sai, `ValueKey` + `findChildIndexCallback` đúng, chỉ `ValueKey` thì mất state |
| `test/trip_history_screen_test.dart` | tải trang đầu, SliverAppBar, tải thêm không gọi trùng, lỗi và Thử lại, kéo để làm mới, bỏ kết quả quá hạn, ghim giữ state |
| `test/trip_history_source_test.dart` | phân trang, cước đúng công thức, ca biên của nguồn giả |
| `test/app_test.dart` | `ToiNoiApp` mở thẳng màn hình lịch sử |

## Kiểm chứng (không cần simulator)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/09-listview-sliver
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (51 test).

## Chạy trên simulator hoặc emulator (tự thử)

Thư mục lab không chứa `android/` và `ios/`. Sinh runner một lần khi muốn chạy trên máy:

```bash
flutter create --platforms=ios,android --project-name toi_noi_09 --org vn.toinoi .
flutter run -d <id>
```

Hai thư mục vừa sinh đã nằm trong `.gitignore`. Chạy iOS cần macOS với Xcode đầy đủ và CocoaPods; chạy Android cần Android SDK và một emulator.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_lints` 6.0.0, Material 3.

## Chưa làm

- Chưa chạy app trên iOS Simulator, Android emulator hay thiết bị thật: máy kiểm chưa có Android SDK và Xcode đầy đủ. Mức kiểm chứng là tự động (`flutter analyze` và `flutter test`). Cảm giác cuộn thật (BouncingScrollPhysics trên iOS, ClampingScrollPhysics trên Android) chưa được kiểm.
- Phân trang theo vị trí (`offset`) chỉ phù hợp dữ liệu ít đổi. Dữ liệu thật nên phân trang theo con trỏ (cursor) để chuyến mới chen vào không làm trang kế bị trùng hoặc sót; lab chưa làm và chưa khử trùng theo `id`.
- Chưa có `intl` (ngày giờ và tiền theo locale: bài 14), chưa có theme riêng (bài 13), chưa có điều hướng sang chi tiết chuyến (bài 12).
- `StatefulWidget` và `setState` chỉ dùng ở mức tối thiểu; vòng đời đầy đủ ở bài 10, quản lý state tách khỏi widget ở bài 17.
- Chưa có `TripHistorySource` gọi HTTP (bài 19).
