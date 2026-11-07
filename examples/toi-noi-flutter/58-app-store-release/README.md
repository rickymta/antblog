# Lab 58: App Store production: metadata, privacy label, App Review và phased release

Lab của bài **FL-58** trong loạt **Flutter qua Tới Nơi**: [App Store production: metadata, privacy labels, App Review và phased release](../../../api/seed/content/flutter-app-store-production-metadata-privacy-review-phased-release.md).

Lab là một project Dart thuần (`toi_noi_58`) cộng với bộ file phát hành mẫu của hai app Tới Nơi (khách và tài xế). Phần Dart đọc các file đó và báo lỗi trước khi ta bấm **Submit for Review**: giới hạn metadata của App Store Connect, kích thước ảnh chụp, `PrivacyInfo.xcprivacy`, privacy label so với manifest và với mô tả quyền trong `Info.plist`, và kế hoạch phased release 7 ngày. Mọi số liệu, tên miền (`toinoi.example`), email, số điện thoại và mã OTP demo (`demo_password.txt`) là hư cấu. Với app thật, mã OTP demo phải ngẫu nhiên, lấy từ secret hoặc ENV, không nằm trong git; server cần giới hạn số lần thử, cảnh báo khi tài khoản demo đăng nhập và đổi mã sau mỗi lần duyệt. Mã `123456` của `tools/fake_api` (bài 24) chỉ dành cho fake API. Các giới hạn của Apple ghi "kiểm tại 11/2026" trong code và trong bài; Apple đổi chúng theo thời gian, hãy đối chiếu lại trang chính thức trước khi phát hành.

## Mức kiểm chứng: đọc kỹ

Máy viết lab này **không** có tài khoản Apple Developer, không có App Store Connect, không có fastlane, và Xcode chưa đầy đủ. Mức theo curriculum: **Kiểm chứng cấu hình (CH)**.

| Phần | Đã làm ở đây? |
|---|---|
| `dart analyze`: 0 issue | Có |
| `dart test`: 135 test (giới hạn metadata ở biên, đếm byte và ký tự, ảnh chụp, manifest, label, phased release, bố cục thư mục) | Có |
| Đọc `PrivacyInfo.xcprivacy`, `Info.plist` và `privacy-label.json` thật của hai app, đối chiếu chéo | Có (test Dart) |
| Kiểm kích thước và kênh alpha của ảnh chụp bằng PNG tạo trong bộ nhớ | Có (test Dart); repo **không chứa ảnh chụp thật** |
| Mỗi quy tắc có ca sai: sửa đúng một chỗ thì checklist phải đỏ đúng mã lỗi | Có |
| Cú pháp Ruby của `Fastfile.store.example` bằng `ruby -c` (Ruby 2.6.10 của hệ thống) và luật: lane có `automatic_release` hoặc `phased_release` không được bỏ qua metadata, cả hai lane tắt `precheck_include_in_app_purchases` | Có; test tự bỏ qua `ruby -c` nếu máy không có `ruby`. Chỉ so chuỗi trong file, **không** chạy lane |
| Tên action và tham số của `upload_to_app_store` | Chỉ đối chiếu docs.fastlane.tools (11/2026), **chưa** với bản cài thật |
| Xcode: thêm `PrivacyInfo.xcprivacy` vào target, **Product > Archive > Generate Privacy Report** | **Chưa chạy** |
| Tạo bản ghi app, nhập privacy label, tải ảnh, gửi duyệt, phased release thật | **Chưa chạy**: cần tài khoản |
| URL trong metadata (support, privacy) mở được | **Chưa kiểm**: lab không dùng mạng, chỉ kiểm cú pháp URL |
| Plugin thật (Firebase, geolocator...) đã đủ privacy manifest sau khi build | **Chưa kiểm**: cần Xcode và build iOS |

Test chứng minh file **có** các tính chất ở trên; chúng không chứng minh App Review sẽ chấp nhận.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/src/store_limits.dart` | `StoreLimits` (tên, subtitle, keywords, mô tả...), `ScreenshotSizes`, `RequiredReasonApi`, `ManifestVocabulary`; mỗi hằng ghi nguồn |
| `lib/src/plist.dart` | `parsePlist`: đọc XML plist, không phụ thuộc package ngoài |
| `lib/src/privacy_manifest.dart` | `PrivacyManifest`, `InfoPlist`, `PrivacyLabel` |
| `lib/src/metadata.dart` | `AppMetadata.read`: đọc thư mục `metadata/` theo bố cục của `fastlane deliver` |
| `lib/src/screenshot.dart` | `PngInfo.parse`: rộng, cao, alpha từ phần đầu file PNG |
| `lib/src/release_checklist.dart` | `ReleaseChecklist`, `checkScreenshots`, `Finding` |
| `lib/src/phased_release.dart` | `PhasedReleasePlan`, `RolloutThresholds`, `RolloutAction` |
| `bin/check.dart` | In kết quả checklist của một app, mã thoát 1 nếu có lỗi |
| `native-config/<app>/ios/PrivacyInfo.xcprivacy`, `Info.plist` | Manifest của app và các khoá mô tả quyền (đoạn cần ghép vào `ios/Runner/`) |
| `native-config/ios/fastlane/Fastfile.store.example` | Hai lane `listing` và `submit` gọi `upload_to_app_store` (deliver) |
| `store/<app>/metadata/` | `vi/*.txt`, `review_information/*.txt`, `copyright.txt`, `primary_category.txt` |
| `store/<app>/privacy-label.json` | Câu trả lời sẽ nhập vào mục App Privacy (ghi trong git để đối chiếu) |
| `test/` | `metadata_test`, `manifest_test`, `label_test`, `screenshot_test`, `phased_release_test`, `fastfile_test`, `layout_test`, `plist_test` |

## Kiểm chứng tự động

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/58-app-store-release
dart pub get
dart analyze
dart test
dart run bin/check.dart customer
dart run bin/check.dart driver --update
```

Kỳ vọng: `No issues found!`, `All tests passed!` (135 test). Lệnh đầu in đúng một phát hiện `screenshot-iphone-required-size` vì app chưa có ảnh; lệnh sau (`--update` tính như bản cập nhật) thêm `release-notes-missing`. Đặt ảnh PNG vào `store/<app>/screenshots/vi/` rồi chạy lại để kiểm cỡ ảnh; `fastlane deliver` khuyên không commit ảnh vào git, nên repo này không có ảnh. Test chạy từ thư mục lab vì đọc file theo đường dẫn tương đối; không dùng mạng.

## Dùng bộ file này cho app thật (tự thử)

1. Chép `native-config/<app>/ios/PrivacyInfo.xcprivacy` thành `ios/Runner/PrivacyInfo.xcprivacy` và thêm vào target Runner trong Xcode (File > Add Files, tick target). Ghép các khoá của `Info.plist` vào `ios/Runner/Info.plist`.
2. Sinh runner một lần nếu chưa có: `flutter create --platforms=ios,android --project-name toi_noi_app --org vn.toinoi .`
3. Chép `store/<app>/` vào gốc app, sửa nội dung cho đúng sản phẩm của bạn, chạy `dart run bin/check.dart <app>` cho tới khi hết lỗi.
4. Chép `Fastfile.store.example` thành `ios/fastlane/Fastfile` (hoặc ghép lane vào Fastfile của bài 53), điền biến `ASC_KEY_ID`, `ASC_ISSUER_ID`, `ASC_KEY_CONTENT`, rồi chạy `bundle exec fastlane ios listing app:customer` trước khi dùng `submit`.
5. Privacy label không có API trong `deliver` (docs ghi riêng một action `upload_app_privacy_details_to_app_store`, đăng nhập bằng Apple ID); lab nhập tay trong App Store Connect rồi đối chiếu với `privacy-label.json`.

## Phiên bản đã kiểm

Flutter 3.47.4 / Dart 3.13.3 (lab không phụ thuộc Flutter, chỉ dùng `dart`). `lints` 6.x, `test` 1.x, và `toi_noi_core` bằng path dependency. Ruby hệ thống 2.6.10 chỉ dùng cho `ruby -c`. Nguồn các giới hạn: App Store Connect Help (App information, Platform version information, Screenshot specifications, Release a version update in phases), App Review Guidelines (cập nhật 16/10/2026), tài liệu `NSPrivacyAccessedAPITypes` và `NSPrivacyCollectedDataTypes` của Apple, docs.fastlane.tools (deliver); tất cả kiểm tại 11/2026. Template Flutter 3.47.4 đặt `IPHONEOS_DEPLOYMENT_TARGET = 15.0` và `TARGETED_DEVICE_FAMILY = "1,2"` (hỗ trợ cả iPad), đọc từ `flutter_tools/templates/app/ios.tmpl`.

## Chưa làm

- Chạy bất kỳ bước nào cần tài khoản hoặc Xcode, và mọi điều trong bảng "Chưa chạy" ở đầu README.
- Ảnh chụp thật, app preview video, ảnh bìa; cỡ ảnh iPad chỉ kiểm khi tham số `supportsIpad` là true (cả hai app mẫu là iPhone-only).
- `Info.plist` và `.xcodeproj` đầy đủ: lab chỉ giữ các khoá đang dạy; `IPHONEOS_DEPLOYMENT_TARGET` là tham số của checklist.
- Xếp hạng độ tuổi (bảng câu hỏi nằm trên App Store Connect, không có file trong repo), Digital Services Act, thuế và thanh toán của Apple.
- Kiểm manifest của plugin sau khi build (đọc trong `.app` hoặc báo cáo của Xcode); lab chỉ kiểm manifest của chính app.
- Localization ngoài `vi`; metadata tiếng Anh.
- Gửi biểu mẫu expedited review (làm trên trang liên hệ của Apple, không có API).
