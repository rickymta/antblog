# Lab 38: tác vụ nền, vị trí nền cho app tài xế và cấu hình native

Lab của bài **FL-38** trong loạt **Flutter qua Tới Nơi**: [Tác vụ nền: workmanager, vị trí nền cho app tài xế và chính sách store](../../../api/seed/content/flutter-background-work-workmanager-vi-tri-nen-app-tai-xe.md).

Phần logic của app tài xế viết thuần Dart và test bằng fake: chính sách "khi nào được gửi vị trí nền" (`DriverLocationPolicy`, tám trạng thái của `toi_noi_core`), bộ đệm vị trí có giới hạn khi mất mạng, bộ gửi theo lô, bộ điều khiển nối chính sách với nguồn vị trí, và hàm việc của task `workmanager`. Plugin (`geolocator`, `workmanager`, `path_provider`) chỉ nằm ở các lớp bọc mỏng. Cấu hình native của app khách và app tài xế khác nhau, đặt trong `native-config/` và được test đọc lại.

## Mức kiểm chứng: đọc kỹ

Máy viết lab này **không** có Android SDK, CocoaPods hay Xcode đầy đủ nên **không chạy được gì trên thiết bị**.

| Phần | Đã chạy ở đây? |
|---|---|
| `flutter analyze` toàn lab (kể cả các lớp bọc plugin: `GeolocatorLocationTracker`, `WorkmanagerScheduler`, `callbackDispatcher`, `main.dart`) | Có: kiểm biên dịch và chữ ký API của `workmanager` 0.10.10, `geolocator` 14.1.2 |
| `flutter test`: chính sách, bộ đệm, bộ gửi, bộ điều khiển, việc của task, cấu hình, màn hình thử | Có, 106 test |
| Test đọc `native-config/` và manifest của plugin trong pub cache | Có: kiểm giá trị trong file, không kiểm Gradle hay Xcode build được |
| Luồng vị trí trong foreground service, thông báo, chỉ báo xanh dương iOS | **Chưa chạy** |
| `workmanager` thật: chu kỳ tối thiểu 15 phút (Android), `BGTaskScheduler` (iOS) | **Chưa chạy** |
| `AppDelegate.swift` biên dịch được; merged manifest sau `flutter build` | **Chưa chạy** |
| Khai báo Google Play, App Review | Không thuộc phạm vi lab |

Mức theo curriculum: **Kiểm chứng trên thiết bị (TB)**. Phần logic được kiểm tự động; phần cần hệ điều hành thật bạn phải tự chạy theo mục "Chạy trên thiết bị".

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/driver_location_policy.dart` | `DriverLocationPolicy`: chỉ `accepted`, `arriving`, `pickedUp`, `inTransit` và tài xế online mới được gửi vị trí nền |
| `lib/location_sample.dart`, `location_buffer.dart` | Mẫu vị trí (UTC, JSON) và bộ đệm giới hạn 120 mẫu / 10 phút |
| `lib/buffer_store.dart` | `BufferStore`: kho trong bộ nhớ (test) và file JSON ghi nguyên tử (chia sẻ giữa isolate) |
| `lib/location_uploader.dart` | Gửi theo lô, chỉ xoá mẫu sau khi gửi xong, không gửi trùng khi hai lần flush chồng nhau |
| `lib/location_tracker.dart` | `LocationTracker`, cấu hình nền theo nền tảng, lớp bọc `Geolocator.getPositionStream` |
| `lib/driver_location_controller.dart` | Bật hoặc tắt luồng vị trí khi trạng thái chuyến hoặc online đổi |
| `lib/background_tasks.dart` | Tên task, kế hoạch (chu kỳ tối thiểu 15 phút), `runBackgroundTask`, `WorkmanagerScheduler` |
| `lib/background_dispatcher.dart` | `callbackDispatcher` cấp cao nhất với `@pragma('vm:entry-point')` |
| `lib/driver_app.dart`, `main.dart` | Màn hình thử: chọn trạng thái chuyến và online, xem quyết định của chính sách và bật luồng vị trí thật; `main` khởi tạo `workmanager` và nối controller |
| `native-config/driver/` | Manifest Android (foreground service loại `location`), `Info.plist` (nền `location`, `fetch`, identifier của BGTaskScheduler, chỉ quyền "khi đang dùng"), `AppDelegate.swift`, cờ `BYPASS_PERMISSION_LOCATION_ALWAYS` cho CocoaPods (`Podfile.fragment.rb`) và Swift Package Manager (`build-spm.fragment.sh`) |
| `native-config/customer/` | Manifest và `Info.plist` tối giản, cùng hai cách đặt cờ `BYPASS_PERMISSION_LOCATION_ALWAYS` |

## Kiểm chứng tự động (không cần thiết bị)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/38-background-work
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (106 test). Test không dùng mạng; `buffer_store_test.dart` ghi file vào thư mục tạm và chạy `Isolate.run`.

## Chạy trên thiết bị (tự thử)

Thư mục lab không chứa `android/` và `ios/`. Sinh runner một lần:

```bash
flutter create --platforms=ios,android --project-name toi_noi_38 --org vn.toinoi .
```

Sau đó chép cấu hình của **app tài xế** (đây là lab mô phỏng app tài xế; muốn thử app khách thì chép thư mục `customer/`):

- `native-config/driver/android/app/src/main/AndroidManifest.xml` đè lên `android/app/src/main/AndroidManifest.xml` (so sánh bằng `diff` trước, vì template của Flutter có thể đổi).
- Các khoá trong `native-config/driver/ios/Runner/Info.plist.fragment.xml` thêm vào `ios/Runner/Info.plist`.
- `native-config/driver/ios/Runner/AppDelegate.swift` đè lên `ios/Runner/AppDelegate.swift`.

Bài học khi thử trên máy thật:

- Bấm **Xin quyền vị trí** trước. Android 14 trở lên: chọn "Tài xế đã nhận" khi màn hình đang hiện (tài xế online), rồi chuyển sang app khác. Trong Logcat không được có dòng "Foreground service started from background can not have location/camera/microphone access". Thông báo "Tới Nơi đang chia sẻ vị trí" phải hiện; từ chối quyền thông báo thì kiểm Task Manager của foreground service.
- Android: `Workmanager().getWorkInfo('vn.toinoi.driver.purge-expired')` trả trạng thái do WorkManager quản lý (ENQUEUED, RUNNING...); thử gọi sau khi `main()` đăng ký task. Chu kỳ tối thiểu giữa hai lần chạy là 15 phút.
- iOS: `Workmanager().printScheduledTasks()` in các task đang chờ của BGTaskScheduler (iOS 13 trở lên). Thời điểm chạy do iOS quyết định, có thể rất lâu.
- iOS: theo Apple, app có quyền "khi đang dùng" được hệ thống tự đổi giao diện thanh trạng thái (xanh dương) khi dùng vị trí nền; kiểm lại trên máy thật.
- iOS: dự án dùng Swift Package Manager (mặc định trên Flutter stable gần đây) thì đặt cờ bằng `BYPASS_PERMISSION_LOCATION_ALWAYS=1 flutter build ios`; dự án CocoaPods thì dùng `Podfile.fragment.rb`.

## Phiên bản đã kiểm

Flutter 3.47.4 / Dart 3.13.3 (stable). Package chính: `workmanager` 0.10.10 (`^0.10.10`), `geolocator` 14.1.2 (`^14.1.2`), `path_provider` 2.1.6, `xml` 7.1.0 (dev), `flutter_lints` 6.0.0. Khớp bảng §4 của khung (workmanager 0.10, geolocator 14.1). `pubspec.lock` không commit.

## Phạm vi chưa làm

- Chưa chạy trên thiết bị hay giả lập: mọi điều ở bảng "Mức kiểm chứng" ghi **Chưa chạy**.
- `LoggingLocationSink` chỉ ghi log. Gửi thật qua `toi_noi_api` cần token từ secure storage trong isolate nền; thuộc bài 47.
- Xin quyền vị trí và quyền thông báo lúc chạy (bài 35 và 37), màn hình giải thích quyền (prominent disclosure) và nút mở cài đặt.
- Hai isolate (giao diện và task nền) có thể cùng đọc một file bộ đệm nên một mẫu có thể lên server hai lần (lab có `dedupeKey` nhưng không có server khử trùng), và hai lần ghi đồng thời thì bản sau cùng thắng, bản kia mất (không hợp nhất).
- Khai báo Google Play (biểu mẫu, video), mô tả App Store Connect.
- Merged manifest: service loại `location` của `geolocator_android` được trộn vào mọi app dùng plugin, kể cả app khách; chưa kiểm ảnh hưởng tới khai báo Play Console của app khách.
