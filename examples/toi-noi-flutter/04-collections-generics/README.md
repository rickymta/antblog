# Lab 04 · Collections, generics và extension

Lab của **bài 04** loạt Flutter qua Tới Nơi: [Collections, generics và extension: quản lý danh sách chuyến và tài xế](../../../api/seed/content/dart-collections-generics-extension-danh-sach-chuyen.md). Dart console thuần, package `toi_noi_04`, không cần Flutter SDK, simulator hay thiết bị.

## Nội dung

| File | Vai trò |
|---|---|
| `lib/model.dart` | Bản rút gọn của mô hình chung: `ServiceKind`, `TripStatus` (8 giá trị, `canCancel`, `isTerminal`), `Trip` bất biến, `Driver` |
| `lib/trip_queries.dart` | `extension TripIterableX on Iterable<Trip>`: `totalFareVnd` (fold), `byStatus` (groupBy), `active` (lười), `sortedByFareDesc` (ổn định), `priciest` (reduce), `allSettled` (every); bản `groupBy` viết tay |
| `lib/paged.dart` | `Paged<T>` (`hasNext`, `map<R>`), `paginate<T>`, `highestBy<T, K extends Comparable<K>>` |
| `lib/driver_lookup.dart` | `Map<String, Driver>` theo id, `Set<String>` thành phố online, `findDriver`, `any`, `firstWhere` + `orElse` |
| `lib/lazy_demo.dart` | `QueryProbe` đếm side effect của `where` + `map`; `ConcurrentModificationError` và cách sửa |
| `lib/literals.dart` | collection-if, collection-for, spread, `const` Set |
| `lib/summary_short.dart`, `lib/summary_long.dart` | Hai extension cùng tên member (và hàm `pluralTrips`) để minh hoạ xung đột và `hide` |
| `bin/main.dart` | Dựng 8 chuyến hư cấu và in tổng cước, nhóm theo trạng thái, trang 1 và 2, tài xế online |

## Chạy

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/04-collections-generics
dart pub get
dart run bin/main.dart
```

## Kiểm chứng

```bash
dart analyze
dart test
```

Hoặc chạy cả sáu lab Phần 1: `node examples/toi-noi-flutter/verify-01-06.mjs`.

Các test chứng minh:

- `trip_queries_test.dart`: tổng cước, nhóm theo trạng thái, `active`, sắp xếp ổn định trên 200 chuyến có cước trùng nhau.
- `paged_test.dart`: `hasNext`, trang cuối lẻ, trang vượt cuối, `map` giữ thông tin phân trang.
- `lazy_demo_test.dart`: bộ đếm bằng 0 trước khi duyệt, bằng n sau `toList()`, duyệt hai lần đếm hai lần, `first` dừng sớm; lỗi thường gặp (`ConcurrentModificationError`, `List.from`, `==` của List).
- `lookup_literals_test.dart`: Set loại trùng, Map thiếu key trả null, `firstWhere` với `orElse`, collection literal.
- `extension_conflict_test.dart`, `extension_hide_test.dart`: xung đột tên member giữa hai extension và hai cách giải.

## Đã kiểm

- Flutter 3.47.4 stable / Dart 3.13.3 (`dart analyze` 0 issue, `dart test` 46 test xanh).
- `package:collection` 1.19.1, `package:lints` 6.1.0, `package:test` 1.33.0.

## Chưa làm

- Chưa có `TripEvent`, `calculateFareVnd` hay `Money`: bài 04 chỉ cần dữ liệu chuyến, cước nhập sẵn (số liệu minh hoạ, không phải bảng giá thật).
- `Paged<T>` mới phân trang trong bộ nhớ theo số trang; phân trang theo cursor và gọi API thật để dành cho phần dữ liệu và mạng.
- Chưa đo hiệu năng collection với dữ liệu lớn.
