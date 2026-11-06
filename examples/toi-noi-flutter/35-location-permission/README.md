# Lab 35: vị trí với geolocator, quyền foreground/background và cấu hình iOS, Android

Lab của bài **FL-35** trong loạt **Flutter qua Tới Nơi**: [Vị trí: geolocator, quyền foreground/background và cấu hình iOS, Android](../../../api/seed/content/flutter-vi-tri-geolocator-permission-foreground-background.md).

Lớp vị trí của app khách và app tài xế Tới Nơi: `LocationSource` bọc plugin, `reduce` là máy trạng thái thuần cho quyền vị trí (bảy trạng thái), `LocationAccessController` nối máy này với hệ điều hành (giữ kết quả lần xin, vì `checkPermission` của Android không bao giờ báo `deniedForever`), `PickupLocationPanel` giải thích rồi mới xin quyền, `PositionTracker` mở và đóng `getPositionStream` theo quyền. Cấu hình native của hai app nằm trong `native-config/` và có test đọc file. Dùng [`toi_noi_core`](../packages/toi_noi_core) (`TripStatus`) để chọn profile theo trạng thái chuyến. Mọi số liệu (`distanceFilter`, chu kỳ) là minh hoạ.

## Mức kiểm chứng: đọc kỹ

Máy viết lab này **không** có Android SDK, CocoaPods hay Xcode đầy đủ, và không có `adb` hay `xcrun simctl`. Vì vậy **chưa có hộp thoại quyền thật nào được hiện, chưa có vị trí thật hay giả lập nào chảy qua plugin**.

| Phần | Đã chạy ở đây? |
|---|---|
| `flutter analyze` | Có: 0 issue |
| `flutter test`: máy trạng thái, controller, widget, tracker, 91 test | Có |
| Lớp bọc `GeolocatorLocationSource` với `GeolocatorPlatform` và `PermissionHandlerPlatform` giả | Có: kiểm ánh xạ quyền, ánh xạ lỗi, settings truyền xuống plugin |
| `native-config/` đúng khoá và giá trị (đọc bằng test Dart) | Có |
| `native-config/` build được bằng Gradle, Xcode, App Store Connect hay Play Console | **Chưa chạy** |
| Hộp thoại quyền thật, Settings, Precise/Approximate, nâng "Always" | **Chưa chạy** |
| Giả lập vị trí (`xcrun simctl location`, `adb emu geo fix`) | **Chưa chạy** |
| Foreground service, thông báo, pin | **Chưa chạy** |

Mức theo curriculum: **Kiểm chứng trên thiết bị (TB)**. Phần logic được kiểm tự động; phần cần thiết bị phải chạy theo hướng dẫn dưới đây.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/location_models.dart` | `SystemPermission`, `LocationPrecision`, `AppRole`, `LocationFix`, `TrackingProfile` (ba profile), `profileFor`, ba exception của lớp nguồn |
| `lib/location_source.dart` | `LocationSource`: interface duy nhất mà phần còn lại biết |
| `lib/geolocator_location_source.dart` | `GeolocatorLocationSource` (bọc `geolocator`, và `permission_handler` cho bước xin nền) và `buildLocationSettings` |
| `lib/location_access.dart` | Bảy trạng thái, `reduce`, `adviceFor`: thuần, không gọi plugin |
| `lib/location_access_controller.dart` | `LocationAccessController`, `AskedStore`, `MemoryAskedStore` |
| `lib/pickup_location_panel.dart` | Giải thích, rồi xin quyền; nút Cài đặt khi bị chặn |
| `lib/position_tracker.dart` | Mở luồng vị trí khi có quyền, huỷ khi mất quyền |
| `lib/main.dart` | App thử: `--dart-define=APP_ROLE=driver` để thử vai tài xế |
| `native-config/customer/`, `native-config/driver/` | `Info.plist`, `AndroidManifest.xml` của từng app; `customer/ios/build-ios.example.sh` (lệnh build SPM); `Podfile.location-fragment.rb` chỉ cho nhánh CocoaPods |
| `test/` | Test; `test/support/fake_location_source.dart` là fake điều khiển được |

File trong `native-config/` là **đoạn cần ghép** vào project do `flutter create` sinh, không phải file đầy đủ.

## Kiểm chứng tự động (không cần thiết bị)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/35-location-permission
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (91 test). Test không dùng mạng, thiết bị hay thời gian thật.

## Chạy trên thiết bị (tự thử)

Sinh runner một lần rồi ghép cấu hình của app muốn thử:

```bash
flutter create --platforms=ios,android --project-name toi_noi_35 --org vn.toinoi .
```

1. **App khách**: ghép các khoá trong `native-config/customer/ios/Info.plist` vào `ios/Runner/Info.plist` và các thẻ `uses-permission` của `native-config/customer/android/AndroidManifest.xml` vào `android/app/src/main/AndroidManifest.xml`. Build iOS bằng Swift Package Manager (mặc định của Flutter 3.47 stable, `flutter create` không sinh Podfile) với `BYPASS_PERMISSION_LOCATION_ALWAYS=1 flutter build ios` (xem `build-ios.example.sh`). Build từ Xcode.app cần `PERMISSION_HANDLER_INFO_PLIST` trỏ tới `ios/Runner/Info.plist`, và đổi biến môi trường nào cũng phải xoá DerivedData. `Podfile.location-fragment.rb` chỉ dùng khi app đã tắt SPM và có Podfile.
2. **App tài xế**: làm tương tự với `native-config/driver/`; SPM không cần macro nào vì `Package.swift` của `permission_handler_apple` bật macro theo khoá có trong `Info.plist`. Chạy với `flutter run --dart-define=APP_ROLE=driver`.

Giả lập vị trí khi đã có thiết bị ảo (chưa chạy ở máy viết lab):

```bash
# Android emulator: kinh độ trước, vĩ độ sau (developer.android.com/studio/run/emulator-console)
adb emu geo fix 106.7009 10.7769
# iOS Simulator: xem cú pháp đúng trên máy của bạn
xcrun simctl help location
```

## Phiên bản đã kiểm

Flutter 3.47.4 / Dart 3.13.3. `geolocator` 14.1.2 (kéo theo `geolocator_android` 5.2.1, `geolocator_apple` 2.4.3, `geolocator_platform_interface` 4.4.1), `permission_handler` 13.0.2 (`permission_handler_android` 14.1.1, `permission_handler_apple` 9.6.5), `xml` 7.1.0 và `plugin_platform_interface` cho test. Các nhận định về hành vi plugin lấy từ đọc mã nguồn các phiên bản này.

## Chưa làm

- Chạy trên thiết bị, simulator, emulator (xem bảng đầu README).
- `AskedStore` bền vững: lab dùng bản trong bộ nhớ; bản thật lưu vào `shared_preferences` (bài 22).
- Thông báo của foreground service trên Android 13+ cần quyền `POST_NOTIFICATIONS` (bài 37) và chạy nền thật (bài 38).
- `ACCESS_BACKGROUND_LOCATION` không có trong manifest nào: foreground service đã được tính là foreground, và nếu có quyền này `geolocator_android` thêm nó vào mọi yêu cầu xin quyền (kể cả nâng chính xác), mà Android 11+ bỏ qua yêu cầu gộp. Vì vậy bước xin nền chỉ có trên iOS (`LocationSource.canRequestBackground`).
- Người dùng đặt "Ask every time" trong Cài đặt Android sau khi đã bị chặn: cờ `blockedByRequest` chỉ xoá khi quyền được cấp.
- Gửi vị trí lên server (bài 36), vẽ lên bản đồ (bài 34).
