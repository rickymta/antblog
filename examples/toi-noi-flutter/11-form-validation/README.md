# Lab 11: Form, TextFormField và validation

Lab của bài **FL-11** trong loạt **Flutter qua Tới Nơi**: [Form, TextFormField và validation: nhập điểm đón, điểm đến](../../../api/seed/content/flutter-form-validation-focus-nhap-diem-don-diem-den.md).

Màn hình đặt xe có một form ba ô: điểm đón, điểm đến và khoảng cách km (nhập tay để tạm thay cho bản đồ ở bài sau). Form chấp nhận dấu phẩy thập phân (`5,5`), kiểm miền 0 đến `FarePolicy.maxKm` km, đưa focus qua các ô bằng `TextInputAction.next`, và khi hợp lệ gọi callback với dữ liệu đã chuẩn hoá. Bảng giá của `toi_noi_core` là số minh hoạ, không phải bảng giá thực tế.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/validators.dart` | Hàm thuần Dart: `normalizeAddress`, `validatePickup`, `validateDestination`, `parseDistanceKm`, `validateDistance` |
| `lib/trip_request_data.dart` | `TripRequestData`: dữ liệu đã chuẩn hoá, `estimatedFare` lấy từ `calculateFare` của `toi_noi_core` |
| `lib/trip_request_form.dart` | `TripRequestForm`: `Form`, ba `TextFormField`, controller, `FocusNode`, `dispose` |
| `lib/request_screen.dart` | `RequestScreen`: `Scaffold` + `SingleChildScrollView`, hiện `SnackBar` khi gửi |
| `lib/main.dart` | `main()` và `ToiNoiApp` |
| `test/validators_test.dart` | Unit test thuần Dart cho các hàm kiểm và `TripRequestData` |
| `test/trip_request_form_test.dart` | Widget test: lỗi, focus, dữ liệu chuẩn hoá, dispose, bàn phím, cạm bẫy ListView |

## Kiểm chứng (không cần simulator)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/11-form-validation
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (82 test).

## Chạy trên máy (tự thử)

Lab không commit thư mục nền tảng. Sinh runner một lần rồi chạy:

```bash
flutter create --platforms=ios,android --project-name toi_noi_11 --org vn.toinoi .
flutter run -d <id>
```

`android/` và `ios/` đã nằm trong `.gitignore` của lab.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_lints` 6.0.0, Material 3, `toi_noi_core` (path, `../packages/toi_noi_core`). `AutovalidateMode.onUserInteractionIfError` có trong source Flutter 3.47.4.

## Chưa làm

- Chưa chạy trên iOS Simulator, Android emulator hay thiết bị thật: máy kiểm chưa có Android SDK và Xcode đầy đủ. Bàn phím ảo được mô phỏng bằng `tester.view.viewInsets`, không phải bàn phím thật; hành vi của từng bàn phím Android và phím Return trên bàn phím số chưa kiểm. Theo kiến thức về iOS (chưa thử trên thiết bị), `numberWithOptions(decimal: true)` ứng với bàn phím decimal pad của iOS, bàn phím này không có phím Return, nên trên iPhone người dùng bấm nút "Tìm tài xế" thay vì Done.
- Chưa có chọn điểm trên bản đồ, gợi ý địa chỉ, autofill, định dạng số theo locale bằng `intl` (bài 14).
- `1.500` được đọc là 1,5 km, không phải một nghìn rưỡi: form không hiểu dấu chấm là phân cách hàng nghìn. Với miền 0 đến 100 km, ta chấp nhận cách đọc này.
- So sánh điểm đón và điểm đến chỉ bỏ qua hoa thường và khoảng trắng, chưa chuẩn hoá Unicode (NFC/NFD) hay bỏ dấu.
- Kiểm phía server (xem bài ASP.NET Core 10 về validation): form không thay thế việc server tự kiểm lại.
