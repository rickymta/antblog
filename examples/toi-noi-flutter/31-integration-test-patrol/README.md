# Lab 31: integration test và Patrol cho luồng đặt chuyến

Lab của bài **FL-31** trong loạt **Flutter qua Tới Nơi**: [Integration test và patrol: luồng đặt chuyến trên simulator/emulator](../../../api/seed/content/flutter-integration-test-patrol-luong-dat-chuyen-tren-thiet-bi.md).

Một app khách rất nhỏ (báo giá, đặt chuyến, theo dõi chuyến, xin quyền thông báo) và **một** hàm kịch bản `runBookingFlow(WidgetTester)` chạy được bằng ba cách: `flutter test` (adapter giả, đồng hồ giả), `integration_test` và Patrol (thiết bị, server thật). Bảng giá minh hoạ do `calculateFare` của [`toi_noi_core`](../packages/toi_noi_core) tính (5 km ra 25.500 VND); client HTTP là [`toi_noi_api`](../packages/toi_noi_api); server là [`tools/fake_api`](../tools/fake_api).

## Mức kiểm chứng: đọc kỹ

Máy viết lab này **không** có Android SDK và Xcode đầy đủ (template iOS của Flutter 3.47.4 dùng Swift Package Manager, nên không thiếu CocoaPods) nên **không chạy được integration test trên thiết bị**. Cụ thể:

| Phần | Đã chạy ở đây? |
|---|---|
| `flutter analyze` cho toàn bộ lab, kể cả `integration_test/`, `patrol_test/`, `test_driver/` | Có: kiểm kiểu và tên API của `patrol` 4.11.0 |
| `flutter test`: hàm `runBookingFlow` với adapter giả, giao diện, lỗi mạng, cấu hình native | Có, 31 test |
| `integration_test/booking_flow_test.dart` trên simulator hoặc emulator | **Chưa chạy** |
| `patrol_test/booking_native_test.dart` (hộp thoại quyền, đưa app xuống nền) | **Chưa chạy** |
| File trong `native-config/` build được bằng Gradle và Xcode | **Chưa chạy**: test Dart chỉ kiểm giá trị trong file |

Mức theo curriculum: **Kiểm chứng trên thiết bị (TB)**. Phần logic của kịch bản được kiểm tự động; phần chỉ có trên thiết bị (hộp thoại của hệ điều hành, đồng hồ thật, HTTP thật qua `10.0.2.2`, bản build có `PatrolJUnitRunner`) cần bạn chạy theo hướng dẫn dưới đây.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/` | App: `bootstrap.dart` (`buildApp`), `booking_controller.dart`, `booking_screen.dart`, `trip_screen.dart`, `providers.dart` (client, polling, quyền), `notification_permission.dart` (ranh giới tới hệ điều hành), `api_base_url.dart` (`10.0.2.2` hay `localhost`), `keys.dart` |
| `test_support/booking_flow.dart` | `runBookingFlow` và các bước của nó, `pumpUntilFound` |
| `test_support/driver_simulator.dart` | "Tài xế" nhận chuyến qua API (thay app tài xế) |
| `test_support/harness.dart`, `fake_notification_permission.dart` | Server giả trong tiến trình và quyền giả cho `flutter test` |
| `test/` | `flutter test`: kịch bản đầy đủ, ca lỗi, `resolveApiBaseUrl`, `DriverSimulator` qua HTTP thật trên loopback, `native_config_test.dart` |
| `integration_test/booking_flow_test.dart` | `integration_test` thuần, server thật, quyền giả |
| `test_driver/integration_test.dart` | Driver cho `flutter drive`, lưu ảnh chụp vào `build/screenshots/` |
| `patrol_test/booking_native_test.dart` | Hai test Patrol: bấm Allow ở hộp thoại thông báo; nhận chuyến lúc app chạy nền |
| `native-config/` | File cấu hình đang dạy: Gradle, hai AndroidManifest, `network_security_config.xml`, `MainActivityTest.java`, `RunnerUITests.m`, đoạn ATS của `Info.plist` |
| `tool/apply-native-config.mjs` | Chép `native-config/` vào runner do `flutter create` sinh |

## Kiểm chứng tự động (không cần thiết bị)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/31-integration-test-patrol
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (31 test). Test không dùng mạng ngoài; `real_http_test.dart` chỉ mở một socket trên loopback.

## Chạy trên thiết bị (tự thử)

Thư mục lab không chứa `android/` và `ios/`. Sinh runner một lần rồi chép cấu hình:

```bash
flutter create --platforms=ios,android --project-name toi_noi_31 --org vn.toinoi .
node tool/apply-native-config.mjs
```

`flutter create .` còn sinh `test/widget_test.dart` (kiểm bộ đếm của `MyApp`, app này không có `MyApp`) làm `flutter analyze` báo lỗi; `tool/apply-native-config.mjs` xoá file đó nếu vẫn là bản template, hoặc tự xoá. Tên project và `--org` phải giữ nguyên để package Android là `vn.toinoi.toi_noi_31` (khớp khối `patrol:` trong `pubspec.yaml`). Bản template của Flutter 3.47.4 là bản mà các file Gradle và manifest trong `native-config/` dựa vào; nếu Flutter đổi template, hãy so sánh bằng `diff` thay vì ghi đè.

Bật fake API trên máy dev (cửa sổ khác):

```bash
cd examples/toi-noi-flutter/tools/fake_api
dart pub get
dart run bin/server.dart          # nghe ở 127.0.0.1:8787
```

Android emulator gọi máy dev qua `10.0.2.2` (app tự chọn, xem `api_base_url.dart`), iOS Simulator gọi `localhost`. Thiết bị thật phải truyền địa chỉ LAN và server phải nghe ngoài loopback: `dart run bin/server.dart --host 0.0.0.0`, rồi `--dart-define=API_BASE_URL=http://<ip-máy-dev>:8787`.

`dio` mặc định gửi qua `dart:io`; theo source engine (`switch_defs.h`: "By default, dart:io allows all socket connections", `FlutterLoader.java` không bật `--disallow-insecure-connections`, `FlutterDartProject.mm` đặt `may_insecurely_connect_to_all_domains = true`) và trang network policy của Flutter, chính sách chặn HTTP của Android (từ API 28) và iOS (ATS) không áp cho socket của Dart, nên lab này không cần mở cleartext để chạy. Chính sách đó chỉ chặn tầng nền tảng (`HttpURLConnection`, `URLSession`, WebView, adapter native như `cronet_http`, `cupertino_http`). `native-config/` giữ `network_security_config.xml` (chỉ bản debug, chỉ `10.0.2.2`) và đoạn ATS như lớp phòng khi đổi adapter; với thiết bị thật dùng adapter native thì thêm địa chỉ LAN vào đó. Kết luận này từ source và tài liệu, chưa chạy trên thiết bị.

**integration_test thuần** (quyền thông báo giả):

```bash
flutter devices
flutter test integration_test/booking_flow_test.dart -d <id-thiết-bị>
# lưu ảnh chụp vào build/screenshots/:
flutter drive --driver=test_driver/integration_test.dart --target=integration_test/booking_flow_test.dart -d <id-thiết-bị>
```

**Patrol** (bấm được hộp thoại thật):

```bash
flutter pub global activate patrol_cli
patrol doctor
patrol test --target patrol_test/booking_native_test.dart --device <id-thiết-bị>
```

Dùng emulator Android 13 (API 33) trở lên nếu muốn thấy hộp thoại quyền thông báo: dưới API 33 không có hộp thoại này, và test được viết để chạy tiếp khi không có hộp thoại. Ảnh chụp của Patrol (`takeNativeScreenshot`) chỉ có trên Android và được kéo về `patrol_test/screenshots/`.

### Bước iOS làm bằng Xcode

`project.pbxproj` không chép tự động được. Làm theo [tài liệu cài đặt Patrol](https://patrol.leancode.co/documentation) (mục iOS setup), dưới đây là các bước đã dịch:

1. Mở `ios/Runner.xcworkspace`. **File > New > Target > UI Testing Bundle**, Product Name `RunnerUITests`, **Organization Identifier giống của `Runner`** (ở lab này là `vn.toinoi`, khớp `bundle_id` trong `pubspec.yaml`), Target to be Tested là `Runner`, ngôn ngữ Objective-C.
2. Xoá `RunnerUITestsLaunchTests.m` (Move to Trash). Thay nội dung `RunnerUITests.m` bằng `native-config/ios/RunnerUITests/RunnerUITests.m` (chạy `node tool/apply-native-config.mjs` sau khi có target cũng chép được file này).
3. Build Settings của `RunnerUITests`: iOS Deployment Target bằng của `Runner`; **User Script Sandboxing = No**. Mỗi build configuration (Debug, Profile, Release) của `RunnerUITests` phải chọn cùng Configuration Set với `Runner`.
4. Dự án dùng Swift Package Manager (template Flutter 3.47.4): thêm `FlutterGeneratedPluginSwiftPackage` vào `RunnerUITests > General > Frameworks and Libraries`, rồi chạy `flutter build ios --config-only patrol_test/booking_native_test.dart`. (Chỉ khi dự án dùng CocoaPods mới cần thêm target `RunnerUITests` vào `ios/Podfile` với `inherit! :complete` và chạy `pod install --repo-update`.)
5. `RunnerUITests > Build Phases`: thêm hai Run Script Phase, đặt tên và xếp thứ tự theo tài liệu. `xcode_backend build` chứa `/bin/sh "$FLUTTER_ROOT/packages/flutter_tools/bin/xcode_backend.sh" build`; `xcode_backend embed_and_thin` chứa `/bin/sh "$FLUTTER_ROOT/packages/flutter_tools/bin/xcode_backend.sh" embed_and_thin`.
6. **Tắt "Execute in parallel"** của `RunnerUITests` trong mọi scheme (tab Test của scheme; nếu scheme dùng test plan thì bỏ chọn trong file `.xctestplan`). Xcode bật sẵn và theo tài liệu Patrol điều này làm hỏng Patrol.
7. Đoạn `native-config/ios/Runner/Info.plist.ats-fragment.xml` là lớp phòng cho adapter native (`dio` mặc định dùng `dart:io` nên không cần); dán vào `Info.plist` nếu muốn.

Các bước này **chưa được chạy** ở đây.

## Phiên bản đã kiểm

Flutter 3.47.4 / Dart 3.13.3. `flutter_riverpod` 3.5.0, `permission_handler` 13.0.2, `patrol` 4.11.0 (kéo theo `patrol_finders` 3.7.0), `integration_test` đi kèm SDK, `dio` 5.11.1, `shelf` 1.4.3, `yaml` 3.1.4. `patrol_cli` 4.9.0 mới nhất lúc viết (chưa cài ở máy này; chỉ đọc mã nguồn và tài liệu).

## Chưa làm

- Chạy trên simulator, emulator hoặc máy thật (xem bảng đầu README).
- Thông báo thật: app chưa phát notification, nên `openNotifications`, `getNotifications`, `tapOnNotificationBySelector` của Patrol chưa được dùng. Bài 37 làm phần này.
- Chạy trên CI hay device farm (Firebase Test Lab, BrowserStack); chạy song song nhiều thiết bị; flavor (`patrol: flavor:`), chỉ bàn ở bài 25 và 50.
- Theo dõi chuyến sau `accepted` (arriving, pickedUp...): polling dừng khi có tài xế; realtime là bài 36.
- macOS, web: không thuộc phạm vi loạt bài.
