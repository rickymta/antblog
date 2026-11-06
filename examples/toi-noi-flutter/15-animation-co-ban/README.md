# Lab 15: Animation cơ bản, implicit animation và Hero khi tìm tài xế

Lab của bài **FL-15** trong loạt **Flutter qua Tới Nơi**: [Animation cơ bản: implicit animation và Hero khi tìm tài xế](../../../api/seed/content/flutter-animation-implicit-hero-tim-tai-xe.md).

Lab là màn hình tìm tài xế của app khách. Một `Trip` của `toi_noi_core` đi qua các `TripEvent` (nút "Bước tiếp theo (mô phỏng)" thay cho realtime) và màn hình phản ứng bằng bốn loại animation ngầm định, cộng một `Hero` sang màn chi tiết. Bảng giá (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt) và mọi số trong dữ liệu tài xế (đánh giá, số chuyến, số phút) là minh hoạ.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/motion.dart` | `ToiNoiMotion`: thời lượng, đường cong và `duration(context, normal)` trả `Duration.zero` khi `MediaQuery.disableAnimationsOf` |
| `lib/search_status_panel.dart` | `SearchStatusPanel`: `AnimatedSwitcher` đổi nội dung theo `TripStatus`, key theo trạng thái |
| `lib/progress_ring.dart` | `ProgressRing`: `TweenAnimationBuilder<double>` quanh `CircularProgressIndicator` |
| `lib/driver_card.dart` | `DriverCard`: `AnimatedContainer` (màu nền, viền), `AnimatedOpacity` (dòng ETA), cùng đường cong `easeInOut`, bọc `Hero` |
| `lib/driver_detail_screen.dart` | Màn chi tiết và `driverDetailRoute` (route có thời lượng theo `ToiNoiMotion`) |
| `lib/find_driver_screen.dart` | Màn tìm tài xế, `nextEventFor` chọn sự kiện kế tiếp |
| `lib/driver.dart` | `Driver` hư cấu, `driverHeroTag`, `TripStatus.progress` |
| `test/motion_test.dart` | 7 test: helper, cờ nền tảng thật, giới hạn iOS, framework rút còn 5% |
| `test/search_status_panel_test.dart` | 7 test: giữa chừng có hai nội dung, về đích, tắt animation, thiếu key |
| `test/progress_ring_test.dart` | 9 test: giá trị trung gian, đổi đích giữa chừng, huỷ lúc 50%, `onEnd`, vòng không `value` làm `pumpAndSettle` hết hạn, bảng tiến độ |
| `test/driver_card_test.dart` | 6 test: màu và viền trung gian, ETA mờ dần, semantics, tắt animation |
| `test/hero_test.dart` | 7 test: tag không trùng, tag trùng bị bắt, chuyến bay giữa chừng, tắt animation, quay lại |
| `test/find_driver_screen_test.dart` | 7 test: cả hành trình, thẻ xuất hiện, huỷ, mở chi tiết, không animation lặp vô hạn |

Tổng cộng 43 test. Test dùng `pump(Duration)` nên không phụ thuộc thời gian thật.

## Kiểm chứng (không cần simulator)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/15-animation-co-ban
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (43 test). Mức: **Kiểm chứng tự động**.

## Chạy trên máy (tự thử)

Lab chỉ commit `pubspec.yaml`, `analysis_options.yaml`, `lib/` và `test/`. Muốn chạy trên simulator hoặc emulator, sinh runner một lần:

```bash
flutter create --platforms=ios,android --project-name toi_noi_15 --org vn.toinoi .
flutter run
```

Bấm "Bước tiếp theo (mô phỏng)" để đi từ "Đã đặt" tới "Hoàn thành". Để thấy hiệu ứng giảm chuyển động: trên Android bật mục "Remove animations" trong cài đặt trợ năng (tên mục và vị trí tuỳ máy); trên iOS xem phần "Phạm vi chưa làm". `android/` và `ios/` đã nằm trong `.gitignore` của lab.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_lints` 6.0.0, Material 3, `toi_noi_core` 0.1.0 và `toi_noi_ui` 0.1.0 (path, không sửa).

## Chưa làm

- Chưa chạy trên iOS Simulator, Android emulator hay thiết bị thật: máy kiểm chưa có Android SDK và Xcode đầy đủ. Chuyển động chỉ được kiểm bằng `pump(Duration)`, chưa nhìn bằng mắt, chưa đo độ mượt (số khung hình) trên máy thật.
- Cờ `reduceMotion` riêng của iOS (`AccessibilityFeatures.reduceMotion`) chưa được đọc: `ToiNoiMotion.reduced` chỉ đọc `MediaQuery.disableAnimationsOf`. Có test ghi lại giới hạn này.
- `ToiNoiMotion` nằm trong lab, chưa chuyển sang `toi_noi_ui`.
- Không có animation tường minh (`AnimationController`), `Hero` tuỳ biến (`flightShuttleBuilder`, `createRectTween`) hay chuyển trang theo nền tảng; nội dung của Phần 8 và các bài sau.
- Chưa có `Semantics` cho vòng tiến độ ngoài nhãn mặc định, text scale lớn (bài 16).
