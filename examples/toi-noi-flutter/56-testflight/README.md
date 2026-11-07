# Lab 56: TestFlight, Beta App Review và export compliance

Lab của bài **FL-56** trong loạt **Flutter qua Tới Nơi**: [TestFlight: internal và external testing, Beta App Review và export compliance cho Tới Nơi](../../../api/seed/content/flutter-testflight-internal-external-testing-beta-review.md).

Lab có hai phần: bộ file cấu hình mẫu trong `native-config/` (Info.plist của app khách và app tài xế, entitlements, ExportOptions cho staging lên TestFlight, một lane `pilot` cho nhóm external, mẫu Test Information) và một package Dart thuần kiểm các file đó cùng kế hoạch nhóm tester. Không có widget và không có plugin, nên lab dùng `dart test`, không cần Flutter.

## Mức kiểm chứng: đọc kỹ

Bài thuộc mức **Kiểm chứng cấu hình (CH)**. TestFlight cần tài khoản Apple Developer Program trả phí, máy macOS và Xcode; máy viết lab không có tài khoản, không build được IPA, và **không tải lên gì cho Apple**.

| Phần | Đã chạy ở đây? |
|---|---|
| `dart analyze` toàn bộ lab | Có |
| `dart test`: `BetaChecklist` (vị trí, push, mã hoá, version và build), `TesterGroups`, hạn 90 ngày, giới hạn gửi Beta App Review, trạng thái build, `TestInformation`, `buildWhatToTest`, công cụ dòng lệnh | Có |
| Đọc file thật trong `native-config/` và kiểm khoá, giá trị, thứ tự dừng sớm của lane | Có |
| `ruby -c` trên `Fastfile.beta-external.example` | Có, bằng Ruby 2.6.10 của hệ thống (test tự bỏ qua nếu máy không có `ruby`) |
| Quét `native-config/` để chặn khoá riêng, chứng chỉ, JWT | Có |
| Con số và quy tắc của Apple (100 internal, 10.000 external, 90 ngày, sáu build mỗi 24 giờ, mười hai trạng thái build) | Đối chiếu với trang chính thức của developer.apple.com, kiểm tại 11/2026; **chưa** kiểm bằng tài khoản thật |
| Tên tham số `upload_to_testflight` | Đối chiếu với docs.fastlane.tools; **chưa** cài fastlane hay chạy lane |
| `flutter build ipa`, `xcrun altool`, Transporter, Xcode Organizer | **Chưa chạy** |
| Tạo nhóm, mời tester, Beta App Review, trả lời export compliance, nhận feedback và crash | **Chưa chạy**: cần tài khoản của bạn |

Các giá trị trong `native-config/` là giữ chỗ: Team ID `ABCDE12345`, tên miền `toinoi.example`, thông tin liên hệ và mật khẩu demo ở dạng `<...>`.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `native-config/customer/ios/Info.plist`, `native-config/driver/ios/Info.plist` | Các khoá bài kiểm: quyền vị trí, `UIBackgroundModes`, `ITSAppUsesNonExemptEncryption`, `CFBundleVersion` |
| `native-config/ios/Runner.entitlements.example` | `aps-environment` |
| `native-config/ios/ExportOptions-staging-testflight.plist.example` | `app-store-connect` cho bundle ID staging, profile loại App Store |
| `native-config/ios/fastlane/Fastfile.beta-external.example` | Lane bổ sung, đặt thành `BetaExternal.fastfile` rồi `import` vào Fastfile của bài 53 (không dán thẳng, vì audit lab 53 cấm `distribute_external`); đầu file liệt kê tám biến `BETA_*` cần thêm |
| `native-config/test-information.json.example` | Mẫu Test Information, chưa điền |
| `lib/src/beta_checklist.dart` | `BetaChecklist`, `CheckIssue`, đọc và so build number |
| `lib/src/tester_groups.dart` | `TesterGroups`, `TesterGroup`, `PublicLink`, kế hoạch của app khách và app tài xế |
| `lib/src/beta_review.dart` | `expectedReview`, `checkBetaSubmission`, `BuildExpiry` |
| `lib/src/build_status.dart` | `BuildStatus`: mười hai trạng thái theo tên Apple |
| `lib/src/test_information.dart`, `what_to_test.dart` | `TestInformation`, `buildWhatToTest` (dùng `calculateFare` của `toi_noi_core`) |
| `lib/src/plist.dart` | Bản sao bộ đọc plist của lab 52 (mỗi lab tự đứng được) |
| `lib/src/cli.dart`, `bin/beta_check.dart` | Công cụ dòng lệnh |
| `test/` | 128 test |

Quyết định của lab, không phải luật Apple: build number phải **tăng** so với bản đã tải lên (Apple chỉ đòi khác nhau); ngưỡng "sắp hết hạn" là 14 ngày; hạn 90 ngày tính từ lúc tải lên (trang Apple không nói mốc bắt đầu); con số 100 internal được đếm trên tổng số người khác nhau của một app (trang Apple không nói rõ là mỗi nhóm hay cả app); `ITSAppUsesNonExemptEncryption` chỉ được kiểm tính nhất quán, lab **không** xác định app có được miễn hay không và không phải tư vấn pháp lý.

## Kiểm chứng tự động

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/56-testflight
dart pub get
dart analyze
dart test
```

Kỳ vọng: `No issues found!` và `All tests passed!`. Không test nào dùng mạng, đồng hồ thật hay thiết bị (thời điểm được truyền vào).

## Dùng công cụ dòng lệnh

```bash
dart run bin/beta_check.dart customer native-config/customer/ios/Info.plist \
  --build-name=1.4.0 --build-number=43 --last-build=42 \
  --entitlements=native-config/ios/Runner.entitlements.example
```

Mã thoát: 0 không có lỗi (cảnh báo vẫn in), 1 có lỗi, 2 dùng sai hoặc không đọc được file. `--build-name` và `--build-number` là giá trị của `$(FLUTTER_BUILD_NAME)` và `$(FLUTTER_BUILD_NUMBER)` trong Info.plist; trên máy thật chúng đến từ `flutter build ipa --build-name ... --build-number ...`.

## Dùng cấu hình trên máy macOS (tự thử)

Cần: máy macOS với Xcode, tài khoản Apple Developer Program, app đã có flavor (bài 50), chứng chỉ và profile (bài 52, 53).

1. Sinh runner một lần: `flutter create --platforms=ios,android --project-name toi_noi_56 --org vn.toinoi .`
2. Chép các khoá của `native-config/<app>/ios/Info.plist` vào `ios/Runner/Info.plist` (đừng thay cả file), `Runner.entitlements.example` thành `ios/Runner/Runner.entitlements`.
3. Tạo profile App Store cho `vn.toinoi.customer.staging`, đặt tên đúng như trong `ExportOptions-staging-testflight.plist.example`, thay `ABCDE12345` bằng Team ID thật.
4. Build và tải lên: `flutter build ipa --release --flavor staging --build-name 1.4.0 --build-number 43 --export-options-plist=...`, rồi Transporter, `xcrun altool --upload-app` hoặc lane `beta` của bài 53. Các lệnh này chưa chạy ở đây.
5. Trong App Store Connect: điền Test Information, tạo nhóm internal trước rồi nhóm external, thêm build vào nhóm. Điền thông tin liên hệ và tài khoản demo trực tiếp trên App Store Connect hoặc qua biến môi trường của CI, không đưa vào repo.

## Phiên bản đã kiểm

Flutter 3.47.4 / Dart 3.13.3. `xml` 7.1.0, `lints` 6.1.0, `test` 1.33.0, `toi_noi_core` (path). Chính sách và con số của Apple đối chiếu với trang chính thức tháng 11/2026.

## Chưa làm

- Tải lên TestFlight bằng bất kỳ cách nào; chạy `flutter build ipa`, `altool`, `pilot`, Transporter, Xcode Organizer.
- Tạo nhóm, mời tester, gửi Beta App Review, nhận trạng thái build thật, feedback và crash.
- Đối chiếu danh sách trường Test Information và review information riêng của TestFlight trên form App Store Connect (trang tài liệu đọc được chỉ nêu đầy đủ các trường của bản App Store).
- Quyết định câu trả lời export compliance cho Tới Nơi: lab chỉ dùng `false` như giá trị mẫu; chủ sản phẩm hoặc người phụ trách pháp lý phải xác nhận.
- Gọi App Store Connect API để lấy trạng thái build thật: `BuildStatus` và `TesterGroups` chỉ mô hình hoá, không gọi API.
- Kiểm bản đã biên dịch: ví dụ cờ `BYPASS_PERMISSION_LOCATION_ALWAYS` của `geolocator_apple` (bài 35) không thấy được từ Info.plist.
