# Lab 40: platform channel, Pigeon và FFI cho app tài xế

Lab của bài **FL-40** trong loạt **Flutter qua Tới Nơi**: [Platform channel, Pigeon và FFI: gọi native khi plugin không đủ](../../../api/seed/content/flutter-platform-channel-pigeon-ffi-goi-native.md).

Tình huống: app tài xế đọc mức pin và chế độ tiết kiệm pin của máy để chọn khoảng cách giữa hai lần gửi vị trí (5, 10 hoặc 30 giây; số liệu minh hoạ). Cùng một hợp đồng `BatteryGateway` được làm hai cách: `MethodChannel` viết tay và Pigeon. Thêm một hàm C `toi_noi_round_fare` (làm tròn cước) gọi bằng `dart:ffi` theo hai cách: `DynamicLibrary` và build hook.

## Mức kiểm chứng: đọc kỹ

Máy viết lab này **không** có Android SDK, Xcode đầy đủ hay CocoaPods, nên **không chạy được app trên simulator, emulator hay thiết bị**.

| Phần | Đã chạy ở đây? |
|---|---|
| Dart: `BatteryGateway` qua `MethodChannel` và qua Pigeon, kiểm bằng messenger giả của `flutter_test` (hai chiều, lỗi, thiếu native) | Có |
| Mã Dart do Pigeon sinh: chạy `dart run pigeon` thật; Dart, Kotlin và Swift sinh ra từ cùng một file định nghĩa | Có |
| Hàm C qua `DynamicLibrary`: biên dịch bằng `cc` của máy trong test, gọi thật | Có trên macOS và Linux; **tự bỏ qua (skip) có lý do** khi không có `cc` |
| Hàm C qua build hook (`native/`): `dart test` chạy hook, biên dịch bằng compiler của máy | Có (macOS, arm64); cần compiler C |
| Kotlin và Swift trong `native-config/` | **Chưa biên dịch**: chỉ có test Dart đọc file để kiểm tên channel, tên method, mã lỗi, luồng gọi. Swift mới được `swiftc -parse` kiểm cú pháp, không kiểm kiểu |
| App chạy trên iOS Simulator, Android emulator, máy thật | **Chưa chạy** |
| Hook cho Android (NDK) và iOS (Xcode) | **Chưa chạy** |

Mức theo curriculum: **kiểm chứng trên thiết bị (TB)**; phần Dart và phần C trên máy host được kiểm tự động.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `pigeons/battery.dart` | Định nghĩa Pigeon (một nguồn cho Dart, Kotlin, Swift) |
| `lib/src/battery_gateway.dart` | Interface `BatteryGateway` và `BatteryUnavailableException` |
| `lib/src/method_channel_battery_gateway.dart` | Cách viết tay: `MethodChannel` hai chiều |
| `lib/src/pigeon_battery_gateway.dart`, `lib/src/generated/battery_api.g.dart` | Cách Pigeon (file `.g.dart` do pigeon sinh, **không sửa tay**) |
| `lib/src/battery_event_stream.dart` | `EventChannel` (chỉ phía Dart, chưa có native) |
| `lib/src/report_interval.dart` | Logic chọn khoảng cách gửi vị trí, kiểm bằng fake |
| `lib/src/fare_rounder.dart` | `DartFareRounder` (chuẩn) và `FfiFareRounder` (gọi C bằng `DynamicLibrary`) |
| `lib/main.dart` | Màn hình nhỏ hiện pin và tần suất (`--dart-define=BATTERY_BRIDGE=channel` để dùng MethodChannel; mặc định Pigeon) |
| `native/` | Package riêng cho hàm C: `src/toi_noi_round_fare.c`, `hook/build.dart`, `lib/toi_noi_round_fare.dart` (`@Native`) |
| `native-config/` | Kotlin và Swift cho phía native, gồm cả file `*.g.kt`, `*.g.swift` do Pigeon sinh (commit để thấy chúng) |
| `test/` | 62 test; `test/support/` có gateway giả |

## Kiểm chứng tự động

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/40-platform-channel-pigeon
flutter pub get
flutter analyze
flutter test                       # 62 test; nhóm FFI tự skip nếu không có cc

cd native
dart pub get
dart analyze
dart test                          # chạy build hook; cần compiler C của máy
```

`native/` nằm ngoài phạm vi `flutter analyze` của lab chính (xem `analysis_options.yaml`) và không phải dependency của lab chính, nên `flutter test` của lab chính không bao giờ phải biên dịch C qua hook; chỉ nhóm test `DynamicLibrary` gọi `cc`. Khi chạy `dart test` trong `native/`, macOS có thể in `SDK iphonesimulator not installed`: hook thăm dò SDK iOS, thông báo không làm test đỏ ở máy này.

Sinh lại mã Pigeon (khi sửa `pigeons/battery.dart`) và kiểm không lệch:

```bash
dart run pigeon --input pigeons/battery.dart
git diff --exit-code lib/src/generated native-config   # chỉ có nghĩa sau khi đã commit bản sinh trước
```

Không chạy `dart format` lên `lib/src/generated/`: Pigeon sinh code theo định dạng riêng, format sẽ làm lệch lần sinh sau.

## Chạy trên thiết bị (tự thử)

Lab không chứa `android/` và `ios/`. Sinh runner một lần rồi chép code native vào:

```bash
flutter create --platforms=ios,android --project-name toi_noi_40 --org vn.toinoi .
rm test/widget_test.dart            # file mẫu của flutter create, không áp dụng cho lab này
cp -R native-config/android/. android/
cp -R native-config/ios/Runner/. ios/Runner/
```

Android: `MainActivity.kt` của template bị ghi đè bằng bản có đăng ký channel. Không cần sửa `AndroidManifest.xml` (đọc pin không cần quyền; `ACTION_POWER_SAVE_MODE_CHANGED` chỉ gửi tới receiver đăng ký bằng code).

iOS: dự án Runner của template (Flutter 3.47.4) **không** tự nhận file Swift mới. Mở `ios/Runner.xcworkspace`, **File > Add Files to "Runner"**, chọn `BatteryReader.swift`, `MethodChannelBattery.swift`, `PigeonBattery.swift`, `BatteryApi.g.swift` và tick target `Runner`. `AppDelegate.swift` đã có trong dự án nên chỉ bị ghi đè. Các bước này **chưa được chạy**.

Simulator iOS không có pin: `UIDevice.batteryState` là `.unknown`, phía native trả lỗi `UNAVAILABLE` và màn hình báo "Không đọc được pin (UNAVAILABLE)". Đó là hành vi mong đợi, không phải lỗi.

## Phiên bản đã kiểm

Flutter 3.47.4 / Dart 3.13.3. `pigeon` 29.1.0 (mới nhất ngày 30/10/2026), `meta` 1.17.0, `yaml` 3.1.4, `flutter_lints` 6.0.0. Trong `native/`: `hooks` 2.2.0, `code_assets` 2.2.0, `native_toolchain_c` 0.19.6 (README của gói này ghi trạng thái thử nghiệm, nhà xuất bản `labs.dart.dev`, với tỷ lệ API và breaking change dự kiến cao hơn nhiều so với gói thường: hãy ghim phiên bản đã kiểm 0.19.6 và đọc changelog trước khi nâng), `lints` 6.1.0, `test` 1.33.0. Trình biên dịch dùng khi kiểm: Apple clang 21.0.0 (arm64, macOS). Chưa chạy trên Linux và Windows.

## Chưa làm

- Chạy trên simulator, emulator, máy thật; biên dịch Kotlin bằng Gradle, Swift bằng Xcode; build hook nhắm Android (NDK) và iOS.
- Phía native của `EventChannel` và của Pigeon `@EventChannelApi`.
- Pigeon Native Interop (FFI/JNI, ghi là thử nghiệm trong README của pigeon 29.1.0): chỉ nhắc ở bài.
- Plugin riêng đóng gói (`flutter create --template=plugin`) và `package_ffi`.
- Windows, Linux, web, macOS.
- Link hook (`hook/link.dart`) và tree-shaking symbol C.
