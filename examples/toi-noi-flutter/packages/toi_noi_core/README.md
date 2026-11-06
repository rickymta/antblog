# toi_noi_core

Thư viện domain dùng chung của **Tới Nơi** (dự án mẫu hư cấu của loạt **Flutter qua Tới Nơi**): tiền, cước, trạng thái chuyến, sự kiện và `Trip` bất biến. Là kết quả của bài 06 [Package, pubspec, lint và test: thư viện toi_noi_core kiểm chứng được](../../../../api/seed/content/dart-package-pubspec-lint-test-toi-noi-core.md); gom code từ lab bài 02 đến 05. App khách và app tài xế ở các bài sau cùng dùng package này.

Phiên bản **0.1.0**, `publish_to: none`. Bảng giá là số minh hoạ, không phải bảng giá thực tế.

## API công khai

Chỉ những gì `lib/toi_noi_core.dart` export:

| Tên | Loại | Vai trò |
|---|---|---|
| `Money` | `final class` bất biến | `final int vnd`; `const Money(vnd)` (chỉ `assert` khi bật assertions) và `Money.fromVnd(vnd)` (luôn ném `ArgumentError` nếu âm, dùng cho dữ liệu từ ngoài); `Money.zero`, `+`, `-`, `*` (nhân int), `<` `<=` `>` `>=`, `Comparable`, `==`/`hashCode`, `toString()` dạng `25.500 VND` |
| `FarePolicy` | `abstract final class` | hằng `int`: `baseFareVnd` 12000, `includedMeters` 2000, `pricePerKmVnd` 4500, `maxKm` 100 |
| `calculateFareVnd(double distanceKm)` | hàm | cước `int` đồng, tính bằng mét nguyên; ngoài miền 0..100 (kể cả NaN) ném `ArgumentError` |
| `calculateFare(double distanceKm)` | hàm | như trên nhưng trả `Money` |
| `ServiceKind` | enum | `ride`, `food`, `parcel`, có `label` |
| `TripStatus` | enum | 8 trạng thái, có `label`, `canCancel`, `isTerminal` |
| `TripEvent` | sealed class | 7 biến thể: `SearchStarted`, `DriverAccepted(driverId)`, `DriverArrived`, `PickedUp`, `TripStarted`, `Completed`, `Cancelled(by)`; kèm `enum CancelledBy { customer, driver }`; `DriverAccepted` và `Cancelled` so sánh theo giá trị |
| `Trip` | `final class` bất biến | `id`, `serviceKind`, `status`, `distanceKm`, `fare` (`Money`), `driverId?`, `requestedAt` (UTC); `Trip.request(...)`; `apply(TripEvent)` trả bản mới hoặc ném `StateError`; `==`/`hashCode` so mọi trường; `isSameTrip(other)` so theo `id`; constructor ném `ArgumentError` nếu `driverId` không khớp trạng thái (`requested`/`searching` phải null, từ `accepted` tới `completed` phải có) |
| `TripIterableX` | extension trên `Iterable<Trip>` | `totalFare` (`Money`), `byStatus`, `active` |

## Cách dùng

Trong `pubspec.yaml` của project khác (đường dẫn tương đối tới package):

```yaml
dependencies:
  toi_noi_core:
    path: ../packages/toi_noi_core
```

```dart
import 'package:toi_noi_core/toi_noi_core.dart';

void main() {
  print(calculateFare(5)); // 25.500 VND
  var trip = Trip.request(
    id: 'TN-1001',
    serviceKind: ServiceKind.ride,
    distanceKm: 5,
    requestedAt: DateTime.now().toUtc(),
  );
  trip = trip.apply(const SearchStarted());
  print(trip.status.label); // Đang tìm tài xế
}
```

Ví dụ chạy được nằm ở [`06-dung-toi-noi-core`](../../06-dung-toi-noi-core).

## Bố cục

```text
lib/
  toi_noi_core.dart        # công khai: chỉ export
  src/                     # riêng của package
    money.dart  fare.dart  service_kind.dart  trip_status.dart
    trip_event.dart  trip.dart  trip_queries.dart
test/                      # money, fare, trip, trip_event, trip_queries
analysis_options.yaml      # lints recommended + strict-casts, strict-inference, strict-raw-types
```

## Kiểm chứng

Từ thư mục này:

```bash
dart pub get
dart analyze
dart test
```

Hữu ích khi học: `dart test --reporter expanded`, `dart test --name "ca biên"`, `dart format --output=none --set-exit-if-changed .`, `dart doc`.

Mức: **Kiểm chứng tự động**. Phiên bản đã kiểm: Flutter 3.47.4 / Dart 3.13.3, `lints` 6.1, `test` 1.33, `collection` 1.19.

## Phạm vi và chưa làm

- Mới có phần domain thuần Dart; chưa có `toi_noi_api` (client, DTO) hay `toi_noi_ui`.
- `Money` chỉ có VND không âm, chưa có tiền tệ khác, chưa định dạng theo locale (bài về localization sẽ dùng `intl`).
- Không có `copyWith` cho `Trip`: mọi chuyển trạng thái đi qua `apply` để bất biến status và `driverId` luôn được kiểm (quyết định có chủ đích).
- `Trip` chưa biết ai đã huỷ (`CancelledBy` chỉ nằm trong sự kiện) và chưa serialize JSON.
- Chưa publish lên pub.dev; từ lúc code dồn vào workspace `toi-noi-app`, package sẽ khai báo `resolution: workspace`.
