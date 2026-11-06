# Lab 10: StatefulWidget, lifecycle và setState

Lab của bài **FL-10** trong loạt **Flutter qua Tới Nơi**: [StatefulWidget, lifecycle và setState: chọn dịch vụ và đếm ngược tìm tài xế](../../../api/seed/content/flutter-statefulwidget-lifecycle-setstate-chon-dich-vu.md).

Hai widget có state: `ServicePicker` (chọn Đặt xe, Giao đồ ăn hoặc Giao bưu kiện bằng `SegmentedButton`) và `SearchingCountdown` (đếm ngược khi tìm tài xế bằng `Timer.periodic`). `FindDriverScreen` ghép hai widget để thấy `didUpdateWidget` làm việc: đổi dịch vụ khi đang tìm thì đếm lại với thời gian của dịch vụ mới. Thời gian chờ 30, 45 và 60 giây là số minh hoạ, không phải cam kết của sản phẩm.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/service_picker.dart` | `ServicePicker`: state giữ dịch vụ đang chọn, đọc `initial` một lần ở `initState` |
| `lib/searching_countdown.dart` | `SearchingCountdown`: `initState` bật timer, `didUpdateWidget` đếm lại khi đổi `seconds`, `dispose` huỷ timer, `mounted` sau `await onCancel()` |
| `lib/find_driver_screen.dart` | `FindDriverScreen` và `searchSecondsFor(ServiceKind)` |
| `lib/main.dart` | `main()` và `ToiNoiApp` |
| `test/service_picker_test.dart` | 8 test: chọn dịch vụ, `initial`, màn hình hẹp, phạm vi rebuild |
| `test/searching_countdown_test.dart` | 18 test: đếm đúng, `didUpdateWidget`, huỷ Timer ở `dispose`, `mounted` sau `await`, hai bản sai cố ý |
| `test/find_driver_screen_test.dart` | 7 test: ghép hai widget, state sống qua lúc cha dựng lại, thoát màn hình khi đang huỷ |
| `test/keys_test.dart` | 4 test: không key, `GlobalKey`, `ValueKey` |

Package `toi_noi_core` (path dependency tới `../packages/toi_noi_core`) cung cấp `ServiceKind` và nhãn tiếng Việt.

## Kiểm chứng (không cần simulator)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/10-stateful-lifecycle
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (37 test). Test không dùng thời gian thật: `tester.pump(Duration)` đẩy đồng hồ giả của `flutter_test`, nên cả bộ chạy trong vài giây.

Khi `SearchingCountdown.dispose` quên `_timer?.cancel()`, `flutter test` đỏ với "A Timer is still pending even after the widget tree was disposed".

## Chạy trên simulator hoặc emulator (tự thử)

Lab không commit thư mục `android/` và `ios/`. Sinh runner một lần khi muốn chạy trên máy:

```bash
flutter create --platforms=ios,android --project-name toi_noi_10 --org vn.toinoi .
flutter run
```

Hai thư mục sinh ra nằm trong `.gitignore` của lab. Chạy iOS cần macOS với Xcode đầy đủ và CocoaPods, chạy Android cần Android SDK và một emulator.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_lints` 6.0.0, Material 3.

## Chưa làm

- Chưa chạy trên iOS Simulator, Android emulator hay thiết bị thật: máy kiểm chưa có Android SDK và Xcode đầy đủ. Mức kiểm chứng là tự động.
- Đếm ngược trừ một mỗi lần `Timer.periodic` chạy, không đối chiếu với đồng hồ thật. Chưa xử lý trường hợp app xuống nền rồi quay lại: cách đếm theo mốc thời điểm và `AppLifecycleListener` thuộc bài sau.
- `onCancel` lỗi chỉ báo một dòng và mở lại nút; phân loại lỗi, retry và ghi log thuộc bài về xử lý lỗi.
- Chưa có `Form` (bài 11), điều hướng (bài 12), theme riêng (bài 13), accessibility và text scale (bài 16).
