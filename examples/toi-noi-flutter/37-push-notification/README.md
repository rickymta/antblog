# Lab 37: push notification với FCM, APNs, thông báo cục bộ và deep link

Lab của bài **FL-37** trong loạt **Flutter qua Tới Nơi**: [Push notification: FCM, APNs, local notification và deep link từ thông báo](../../../api/seed/content/flutter-push-notification-fcm-apns-deep-link.md).

Một app khách rất nhỏ: bật thông báo, đăng ký thiết bị với backend, nhận tin ở ba trạng thái (foreground, background, terminated) và mở đúng màn hình chuyến bằng `go_router`. Logic nằm sau hai interface tự viết (`PushSource`, `LocalNotifier`) và được test bằng bản giả; `firebase_messaging` và `flutter_local_notifications` chỉ có mặt ở hai lớp bọc mỏng.

## Mức kiểm chứng: đọc kỹ

Máy viết lab **không** có Android SDK, CocoaPods hay Xcode đầy đủ, **không có Firebase project thật** và không có thiết bị. Cụ thể:

| Phần | Đã chạy ở đây? |
|---|---|
| `flutter analyze` toàn bộ lab (kể cả `FirebasePushSource`, `FlutterLocalNotifier`, `main.dart`) | Có: kiểm biên dịch và tên API của `firebase_messaging` 17.0.1, `flutter_local_notifications` 23.0.0 |
| `flutter test`: `PushRouter`, `PushNavigator` (ba trạng thái), `PushRegistrar`, `HttpDeviceApi`, luồng go_router, ánh xạ `RemoteMessage`, đọc `native-config/` | Có |
| Nhận push thật từ FCM/APNs trên Android và iOS | **Chưa chạy** |
| Hộp thoại quyền thông báo thật (Android 13+, iOS) | **Chưa chạy** |
| `FirebasePushSource`, `FlutterLocalNotifier` chạy với plugin thật | **Chưa chạy** |
| `native-config/` build được bằng Gradle và Xcode | **Chưa chạy**: test Dart chỉ kiểm giá trị trong file |

Mức theo curriculum: **Kiểm chứng trên thiết bị (TB)**. Phần logic được kiểm tự động; phần cần Firebase project, APNs key và thiết bị phải do bạn chạy.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/push_router.dart` | `PushRouter`: payload `{"type":"trip_update","tripId":"..."}` thành `/trip/<id>`; từ chối payload lạ; `safeSummary` cho log |
| `lib/push_navigator.dart` | `PushNavigator`: nối foreground, background, terminated và chạm thông báo cục bộ với một hàm `open` |
| `lib/push_registrar.dart` | `PushRegistrar`: xin quyền, `register`, gửi FID lên backend, `signOut` (xoá ở backend trước, `unregister` sau) |
| `lib/push_source.dart`, `lib/local_notifier.dart` | Hai interface ranh giới tới plugin |
| `lib/firebase_push_source.dart` | Lớp bọc `FirebaseMessaging` (và hai hàm ánh xạ thuần) |
| `lib/background_handler.dart` | Handler nền: top-level, `@pragma('vm:entry-point')` |
| `lib/device_api.dart` | `DeviceApi` và bản `dio`: `PUT/DELETE /devices/{deviceId}` |
| `lib/notification_channel.dart` | Id kênh Android, dùng chung với manifest |
| `lib/app.dart`, `lib/main.dart` | go_router (`/` và `/trip/:id` lồng nhau), hai màn hình, khởi tạo |
| `native-config/` | Mẫu cấu hình: manifest, `google-services.json.example`, plist, entitlements, `AppDelegate.swift` (UIScene), đoạn Gradle (compileSdk 37, desugaring, AGP), và `backend/fcm-send.json.example` (tin mẫu của backend) |
| `test/` | 121 test; `native_config_test.dart` đọc `native-config/` và quét bí mật |

## Kiểm chứng tự động

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/37-push-notification
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!`. Không test nào dùng mạng, Firebase hay thiết bị.

## Chạy trên thiết bị (tự thử)

Cần: một Firebase project của bạn, app Android và app iOS đã đăng ký trong project đó, một APNs authentication key (`.p8`) đã tải lên mục Cloud Messaging của Firebase, và thiết bị thật (các bước thử trong tài liệu Firebase đều giả định thiết bị thật).

1. Sinh runner một lần: `flutter create --platforms=ios,android --project-name toi_noi_37 --org vn.toinoi .`
2. Tải `google-services.json` và `GoogleService-Info.plist` **của project bạn** từ Firebase console; đặt vào `android/app/` và `ios/Runner/` (thêm vào target Runner bằng Xcode). Hai file này nằm trong `.gitignore`, đừng commit.
3. Đổi package Android và bundle ID trong runner cho khớp app đã đăng ký (mẫu dùng `vn.toinoi.customer.dev`).
4. Chép các mục trong `native-config/` vào runner theo chú thích đầu mỗi file: manifest (quyền, hai `meta-data`), đoạn Gradle, `Info.plist`, `AppDelegate.swift`, capability Push Notifications và Background Modes (Background fetch, Remote notifications) trong Xcode.
5. `flutter run` rồi bấm **Bật thông báo**. Gửi thử một tin từ Firebase console hoặc từ backend của bạn tới FID của máy (in FID ra log khi thử; đừng đưa vào bài viết hay commit).
6. Kiểm ba trạng thái: app đang mở, app ở nền, app bị vuốt tắt hẳn.

`flutter_local_notifications` 23.0.0 yêu cầu `compileSdk` 37, desugaring và AGP từ 9.1.1 (README của plugin); template Flutter 3.47.4 sinh `compileSdk` 36 và AGP 9.1.0 nên phải đổi theo đoạn Gradle trong `native-config/`. Các thay đổi này chưa build thử.

## Backend giả định

Lab giả định backend có hai endpoint (không có trong `tools/fake_api`):

- `PUT /devices/{deviceId}` với body `{"provider":"fcm","fid":"...","platform":"android|ios"}`.
- `DELETE /devices/{deviceId}`; `404` được coi là đã xoá.

Tin push gửi bằng trường `fid` của FCM HTTP v1 (trường `token` đã deprecated theo tài liệu Firebase), `data` là map chuỗi sang chuỗi: `{"type":"trip_update","tripId":"T-1002"}`.

## Phiên bản đã kiểm

Flutter 3.47.4 / Dart 3.13.3. `firebase_core` 5.1.0, `firebase_messaging` 17.0.1, `flutter_local_notifications` 23.0.0, `go_router` 18.1.3, `dio` 5.11.1, `xml` 7.0.1 (dev), `flutter_lints` 6.0.0. Các số liệu về Firebase/Android đối chiếu với tài liệu chính thức tháng 11/2026.

## Chưa làm

- Chạy trên thiết bị, nhận push thật, hộp thoại quyền thật, APNs thật (xem bảng đầu README).
- Ba flavor với ba app Firebase: `google-services.json.example` có ba client để minh hoạ, nhưng cách chọn file theo flavor khi build thuộc bài khác của loạt.
- Guard đăng nhập cho `/trip/:id` (bài 23): lab không kiểm người dùng đã đăng nhập khi mở chuyến từ thông báo.
- Lưu `deviceId` bền vững (bài 22) và hàng đợi gửi lại khi đăng xuất mà backend lỗi (bài 46).
- Tải trạng thái chuyến từ server khi mở `TripScreen`: màn hình chỉ hiện mã chuyến.
- Thông báo có hành động, nhóm thông báo, badge, tin chỉ có `data` xử lý ở nền.
