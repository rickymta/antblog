# Lab 41: hiệu năng, rebuild, isolate và DevTools

Lab của bài **FL-41** trong loạt **Flutter qua Tới Nơi**: [Hiệu năng: rebuild, const, RepaintBoundary, isolate và DevTools](../../../api/seed/content/flutter-hieu-nang-rebuild-isolate-devtools.md).

**Nói thẳng về phạm vi đo.** `flutter test` chạy ở debug, trên máy phát triển, không có GPU và không có pha raster thật. Lab này **không đo được mili giây khung hình**. Thứ nó đo được và kiểm bằng test là các con số không phụ thuộc máy: số lần `build`, số lần `paint`, số dòng danh sách được tạo và build, số byte ảnh giải mã. Số mili giây thật cần `flutter run --profile` trên máy thật và DevTools; lab không có số đo nào như vậy. Mọi con số về thời gian trong bài và README (ví dụ benchmark Dart thuần) là minh hoạ, chạy một lần trên máy phát triển, không phải số đo trên điện thoại.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/build_log.dart` | `BuildLog` + `BuildProbe` (đếm build theo nhãn), `RebuildRecorder` (dùng `debugOnRebuildDirtyWidget`) |
| `lib/rebuild_screens.dart` | `NaiveTripScreen` (setState ở gốc), `SplitTripScreen` (tách widget), `ChildParamTripScreen` (tham số `child` của builder), `QuoteCard`, `EtaText` |
| `lib/trip_live.dart` | `TripLive`, `TripLiveNotifier`, bốn nhãn cước: xem cả state, `select`, `select` trả `List` (lỗi), `select` trả record |
| `lib/paint_panels.dart` | `PaintLog`, `CountingPainter`, `LiveMapPanel` (có/không `RepaintBoundary`) |
| `lib/trip_list.dart` | `TripHistoryList` với ba cách: `Column`, `ListView(children:)`, `ListView.builder`; `scrollCacheExtent` |
| `lib/avatar.dart` | `decodeSizeFor`, `avatarProvider` (`ResizeImage`), `DriverAvatar` |
| `lib/trip_json.dart` | `generateTripsJson`, `parseTrips`, `parseTripsInIsolate` (`Isolate.run`) |
| `lib/frame_budget.dart` | `frameBudget(hz)`, `JankMeter` (phân loại `FrameTiming` theo ngân sách) |
| `lib/history_page.dart`, `lib/main.dart` | Trang thử tay để quan sát bằng DevTools ở profile mode |
| `tool/benchmark_isolate.dart` | Benchmark Dart thuần: mốc "không làm gì", parse đồng bộ, `Isolate.run`, nhiều lần chạy (minh hoạ) |
| `tool/benchmark_copy.dart` | Benchmark Dart thuần: chi phí sao chép đối số, cạm bẫy chung scope, trả kết quả lớn (minh hoạ) |
| `test/` | 77 test, xem bên dưới |

Test: `rebuild_test.dart` (15), `select_test.dart` (10), `repaint_test.dart` (6), `lazy_list_test.dart` (12), `image_cache_test.dart` (9), `isolate_test.dart` (14), `frame_budget_test.dart` (9), `history_page_test.dart` (2). Cước demo luôn tính bằng `calculateFare` (5 km là 25.500 VND, 2,001 km là 12.005 VND, 100 km là 453.000 VND); bảng giá là số minh hoạ.

## Kiểm chứng (không cần simulator, không cần mạng)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/41-performance
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (77 test). Mức kiểm chứng: **Kiểm chứng tự động**. Chỉ các test dùng isolate thật (`isolate_test.dart`, `history_page_test.dart`) chạy theo đồng hồ thật; chúng chờ theo sự kiện, không ngủ cố định.

Đã kiểm với Flutter 3.47.4 / Dart 3.13.3: `flutter_riverpod` 3.5.0 (ràng buộc `^3.4.3`), `flutter_lints` 6.0.0.

## Benchmark Dart thuần (minh hoạ)

```bash
dart run tool/benchmark_isolate.dart 50000 7   # số chuyến, số lần chạy
dart run tool/benchmark_copy.dart 7            # số lần chạy
```

Hai script chạy trên Dart VM (JIT) của máy phát triển, in khoảng giá trị của nhiều lần chạy cùng mốc "không làm gì" (nhiễu của chính `Timer.periodic` 2 ms). Kết quả đổi mạnh theo tải máy, nên chỉ dùng để thấy hình dạng, không làm mốc hiệu năng. Một lần chạy khi máy đang tải nặng (load average 60 đến 97), khoảng cách lớn nhất giữa hai nhịp Timer, tính bằng ms:

| Trường hợp | Khoảng giá trị |
|---|---|
| không làm gì (nhiễu nền) | 3 đến 14 |
| parse 50.000 chuyến, đồng bộ | 117 đến 159 |
| parse 50.000 chuyến, `Isolate.run` | 8 đến 21 |
| đối số `List<Map>` 1 triệu phần tử | 852 đến 2.570 |
| đối số `String` 50 MB | 2 đến 7 |
| closure chỉ dùng `String`, cùng scope với closure bắt `List` lớn | 1.025 đến 2.163 |
| trả 1 triệu `Map` bằng `Isolate.exit` | 50 đến 91 |

`Isolate.run` nằm sát vùng nhiễu nên không kết luận được nó đưa độ chặn về 0; nguyên nhân main vẫn dừng khi trả kết quả lớn chưa xác định.

## Đo thật trên máy thật (chưa làm ở đây)

```bash
flutter create --platforms=ios,android --project-name toi_noi_41 --org vn.toinoi .
flutter run --profile        # trên thiết bị thật, không dùng simulator hay debug
```

Rồi mở DevTools, tab **Performance**: xem biểu đồ khung hình, khung đỏ là jank, bật **Track Widget Builds / Layouts / Paints** để xem sự kiện theo từng widget. Thư mục lab không chứa `android/`, `ios/`.

## Chưa làm

- Mọi số đo mili giây trên thiết bị, jank ở profile mode, thời gian raster, shader jank (bài 61), thời gian khởi động và kích thước app (bài 62).
- Tìm nguyên nhân main isolate vẫn dừng khi `Isolate.run` trả kết quả lớn (benchmark ghi nhận hiện tượng, chưa xác định nguyên nhân).
- Isolate sống lâu (`Isolate.spawn`, `ReceivePort`) và `TransferableTypedData`.
- Impeller, `saveLayer`, `Opacity` và clip: chỉ nhắc trong bài, lab không có test.
