# Lab bài 05: Future, async/await và Stream

Lab của bài [Future, async/await và Stream: gọi báo giá và lắng nghe vị trí tài xế](../../../api/seed/content/dart-future-async-await-stream-bao-gia-va-vi-tri.md) trong loạt **Flutter qua Tới Nơi** (FL-05). Dart thuần, chạy bằng `dart`, không cần Flutter, simulator hay thiết bị.

## Nội dung

| File | Vai trò |
|---|---|
| `lib/quote_service.dart` | `QuoteService.fetchFareVnd` (độ trễ mô phỏng, tiêm qua constructor), `calculateFareVnd`, `QuoteException`, `fetchWithTimeout`, `fetchFaresInParallel` (`Future.wait`), `fetchFromFastest` (`Future.any`) |
| `lib/driver_location.dart` | `Position`, `distanceMeters` (equirectangular), `DriverLocationFeed.positions` (single-subscription hoặc broadcast), `movedMoreThan`, `onlyMoved` |
| `lib/event_loop_demo.dart` | `traceEventLoop` (sync, microtask, event), `traceQueues` (đủ năm cách xếp việc) và `traceAwaitOrder` trả `List<String>` để test |
| `lib/heavy.dart` | `sumUpTo`, `sumInIsolate` (`Isolate.run`), `measureTicks` |
| `bin/main.dart` | Chạy báo giá, timeout, nghe 5 vị trí rồi cancel, in thứ tự event loop, so sánh main isolate với `Isolate.run` |
| `test/` | 34 test; dùng `package:fake_async` nên không chờ thời gian thật |

## Chạy

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/05-async-stream
dart pub get
dart run bin/main.dart
```

Chương trình chạy xong trong vài giây. Hai dòng cuối (số lần timer chạy khi tính trên main isolate và trên isolate khác) phụ thuộc máy; chỉ cần nhận ra main isolate gần như 0 lần còn `Isolate.run` nhiều lần.

## Kiểm chứng

```bash
dart analyze
dart test
```

Mức: **Kiểm chứng tự động** (không cần thiết bị). Test chứng minh:

- `fetchFareVnd(5)` trả 25.500 đúng sau 300 ms ảo (`fake_async`), lỗi ngoài miền đến qua Future dưới dạng `QuoteException`.
- `fetchWithTimeout` ném `TimeoutException` sau 100 ms ảo và việc phía service vẫn chạy tiếp (timeout không huỷ).
- `Future.wait` giữ thứ tự đầu vào, tốn 300 ms ảo cho 3 quãng đường; mặc định chờ hết rồi mới báo lỗi xảy ra sớm nhất, `eagerError: true` báo ngay ở 100 ms; `Future.any` lấy nguồn nhanh nhất.
- `movedMoreThan`/`onlyMoved` lọc đúng bằng `expectLater(stream, emitsInOrder([...]))`.
- Stream: single-subscription ném `StateError` khi nghe lần hai; broadcast cho nhiều người nghe, người nghe muộn bỏ lỡ sự kiện; `cancel` và `await for` + `break` không để lại timer; `pause`/`resume` dừng và tiếp tục nguồn.
- Thứ tự event loop `['sync', 'microtask', 'event']` và thứ tự đủ năm cách xếp việc (`scheduleMicrotask`, `Future.microtask`, `Timer.run`, `Future`); `sumInIsolate` đúng và ném lại lỗi cho người gọi.

## Phiên bản đã kiểm

Dart 3.13.3 (Flutter 3.47.4 stable), `test` ^1.33.0, `fake_async` ^1.3.3, `lints` ^6.1.0.

## Phạm vi chưa làm

- Không có mạng thật: `QuoteService` chỉ mô phỏng độ trễ bằng `Future.delayed`; chưa có retry, backoff, huỷ request thật (bài 19 và 20).
- Vị trí tài xế là dữ liệu giả phát lại từ danh sách, chưa có GPS, WebSocket hay xử lý mất kết nối (bài 35 và 36).
- Chưa dùng `Isolate.spawn`, cổng (`SendPort`) hay isolate sống lâu; `CancelableOperation` của `package:async` nằm ngoài phạm vi.
- Bảng giá và độ trễ là số minh hoạ, không phải của sản phẩm thật.
