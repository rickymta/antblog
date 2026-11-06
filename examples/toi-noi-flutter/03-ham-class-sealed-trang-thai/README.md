# Lab 03: hàm, class, enum và sealed class

Lab của bài 03 trong loạt **Flutter qua Tới Nơi**: [Hàm, class, enum và sealed class: mô hình trạng thái chuyến bằng Dart](../../../api/seed/content/dart-ham-class-enum-sealed-pattern-matching-trang-thai-chuyen.md). Project Dart thuần (package `toi_noi_03`), không cần Flutter UI, simulator hay thiết bị.

## Nội dung

| File | Vai trò |
|---|---|
| `lib/functions_tour.dart` | hàm: tham số vị trí, đặt tên, `required`, mặc định, hàm mũi tên, hàm là giá trị (`typedef TripFilter`) |
| `lib/service_kind.dart` | `enum ServiceKind { ride, food, parcel }` có `label` tiếng Việt |
| `lib/trip_status.dart` | enhanced enum `TripStatus` tám trạng thái, `label`, `canCancel`, `isTerminal` |
| `lib/trip_event.dart` | `sealed class TripEvent` với bảy biến thể, `enum CancelledBy`, hàm `describeEvent` (switch vét cạn, không có `_`) |
| `lib/quote.dart` | hàm tự do `quoteFor` trả record đặt tên `({int fareVnd, double distanceKm})` |
| `lib/trip.dart` | class `Trip` bất biến, `apply(TripEvent)` bằng switch trên record `(status, event)` |
| `bin/main.dart` | chạy một chuyến hoàn thành và một chuyến bị huỷ ở `arriving` |
| `test/trip_test.dart` | bảng 8 trạng thái x 8 sự kiện mẫu (64 ca), hàm, enum, bất biến, `==`/`hashCode`, record (96 test) |

Bảng giá trong `quote.dart` (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền 0 đến 100 km) là số minh hoạ, không phải bảng giá thực tế.

## Chạy

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/03-ham-class-sealed-trang-thai
dart pub get
dart run bin/main.dart
```

## Kiểm chứng

```bash
dart analyze
dart test
```

Mức: **Kiểm chứng tự động** (`dart analyze` 0 issue, `dart test` xanh). Phiên bản đã kiểm: Flutter 3.47.4 / Dart 3.13.3, `lints` 6.x, `test` 1.x.

## Chưa làm

- Chưa lưu chuyến vào database và chưa đồng bộ với server; `Trip` chỉ sống trong bộ nhớ.
- `Trip` không ghi lại ai đã huỷ (`CancelledBy`); thông tin này thuộc nhật ký sự kiện, sẽ xuất hiện ở bài máy trạng thái trên client.
- Chưa có `toi_noi_core`: code gom vào package ở bài 06.
