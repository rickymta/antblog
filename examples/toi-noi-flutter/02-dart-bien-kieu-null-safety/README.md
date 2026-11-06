# Lab 02: biến, kiểu dữ liệu và null safety khi tính cước Tới Nơi

Lab đi kèm bài 02 của lộ trình **Flutter qua Tới Nơi**: [Dart cơ bản: biến, kiểu dữ liệu và null safety khi tính cước Tới Nơi](/articles/dart-bien-kieu-du-lieu-null-safety-tinh-cuoc).

Đây là project Dart console (`dart create -t console`, package `toi_noi_02`), chưa dùng Flutter.

## Nội dung

| File | Vai trò |
|---|---|
| `lib/fare.dart` | `calculateFareVnd(double)` theo bảng giá minh hoạ (đổi km sang mét nguyên rồi tính bằng `int`), `formatVnd(int)` chèn dấu chấm nghìn |
| `lib/quote.dart` | `Quote` với `String? driverName` (nullable), `late final fareText`, `describe()` dùng `?.` và `??` |
| `bin/main.dart` | In báo giá 5 km có tài xế và một báo giá chưa có tài xế |
| `test/fare_test.dart` | Ca biên của cước: 0, 1, 2, 2.001, 2.0004, 2.0006, 2.3, 5, 100, -1, 100.5, NaN, Infinity; `formatVnd` |
| `test/quote_test.dart` | null và chuỗi rỗng, `withDriver`, `late` |
| `test/basics_test.dart` | Thí nghiệm kiểm chứng khẳng định trong bài: `/` và `~/`, `round()` trên double và NaN, `!`, `dynamic` |

Bảng giá chỉ để minh hoạ: 12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền hợp lệ 0 đến 100 km, ngoài miền ném `ArgumentError`, `round()` chỉ dùng ở biên km sang mét (độ phân giải 1 m), phần lẻ đồng làm tròn nửa lên bằng số nguyên. Không phải bảng giá thực tế của sản phẩm.

## Chạy

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/02-dart-bien-kieu-null-safety
dart pub get
dart run bin/main.dart
```

Kết quả mong đợi:

```text
Đã có tài xế : 5.0 km | 25.500 VND | Châu
Chưa có tài xế: 2.3 km | 13.350 VND | chưa có tài xế
Sau khi nhận  : 2.3 km | 13.350 VND | An
5 / 2 = 2.5, 5 ~/ 2 = 2
453.000 VND
```

## Kiểm chứng

```bash
dart analyze
dart test
```

Kết quả đã kiểm: `dart analyze` không có issue, `dart test` chạy 40 test xanh.

Mức kiểm chứng: **kiểm chứng tự động**. Không cần simulator, emulator hay thiết bị.

## Phiên bản đã kiểm

Flutter 3.47.4 stable / Dart 3.13.3 (lab chỉ dùng Dart SDK), `lints` 6.1.0, `test` 1.33.0.

## Chưa làm

- Chưa có kiểu `Money` và `Trip` (bài 03 và bài 06).
- Chưa định dạng tiền bằng `intl` theo locale (bài về localization).
- `calculateFareVnd` vẫn nhận `double` km cho dễ học; khi lưu và truyền dữ liệu, các bài sau dùng `int` mét và bỏ bước đổi km sang mét.
- Chưa có kiểu `Distance` bọc mét nguyên.
- Chưa có phụ phí, giảm giá hay bảng giá theo thời điểm.
