# Lab 52: ký ứng dụng iOS (certificate, provisioning profile, capabilities)

Lab của bài **FL-52** trong loạt **Flutter qua Tới Nơi**: [Ký ứng dụng iOS: Apple Developer Program, certificate, provisioning profile và capabilities](../../../api/seed/content/flutter-ky-ung-dung-ios-certificate-provisioning-profile-capabilities.md).

Lab có hai phần: bộ file cấu hình ký mẫu trong `native-config/ios/` cho app khách (ba flavor `dev`, `staging`, `prod`) và một package Dart thuần đọc, kiểm các file đó cùng một provisioning profile giả dạng plist. Không có widget, không có plugin, nên lab dùng `dart test` chứ không cần Flutter.

## Mức kiểm chứng: đọc kỹ

Bài thuộc mức **Kiểm chứng cấu hình (CH)**. Ký iOS cần máy macOS, Xcode và tài khoản Apple Developer Program trả phí; máy viết lab không có chứng chỉ hay tài khoản, và **không chạy `xcodebuild archive` hay `xcodebuild -exportArchive`**.

| Phần | Đã chạy ở đây? |
|---|---|
| `dart analyze` toàn bộ lab | Có |
| `dart test`: đọc plist và xcconfig, khoá và giá trị trong `native-config/`, đối chiếu chéo xcconfig với ExportOptions và profile mẫu, hạn dùng, allowlist entitlement, CLI | Có |
| Quét `native-config/` và `samples/` để chặn khoá riêng, chứng chỉ, Team ID thật | Có |
| Tên `method` hiện hành của ExportOptions | Đối chiếu với trích dẫn `xcodebuild -help` của Apple DTS và source Flutter 3.47.4; **chưa** tự chạy `xcodebuild -help` trên máy này |
| Đọc file `.mobileprovision` thật (CMS) | **Chưa chạy**: script chỉ đọc plist đã giải bằng `security cms -D -i`, và nhận ra file CMS để nhắc lệnh đó |
| Build, ký, export IPA, cài lên thiết bị | **Chưa chạy** |
| Tạo certificate, App ID, profile trên Apple Developer | **Chưa chạy**: cần tài khoản của bạn |

Các file trong `samples/` là plist **giả** (Team ID `ABCDE12345`, UDID toàn số 0, `DeveloperCertificates` là chuỗi base64 của "FAKE"). Profile thật có thêm nhiều khoá và nằm trong lớp CMS.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `native-config/ios/Runner.entitlements.example` | `aps-environment` và associated domains `toinoi.example` |
| `native-config/ios/Info.plist.fragment.example` | `UIBackgroundModes` (Background Modes ghi vào Info.plist) |
| `native-config/ios/Signing-{dev,staging,prod}.xcconfig.example` | `DEVELOPMENT_TEAM`, `CODE_SIGN_STYLE`, `PROVISIONING_PROFILE_SPECIFIER` theo flavor |
| `native-config/ios/ExportOptions-{adhoc,appstore}.plist.example` | Tuỳ chọn cho `xcodebuild -exportArchive` (`release-testing`, `app-store-connect`) |
| `samples/*.plist` | Bốn profile giả: development, ad hoc, ad hoc cũ (thiếu push), App Store |
| `lib/src/plist.dart`, `xcconfig.dart` | Đọc plist XML và xcconfig |
| `lib/src/provisioning_profile.dart` | `ProvisioningProfile`: loại, hạn dùng, thiết bị, allowlist entitlement |
| `lib/src/export_options.dart` | `lintExportOptions`: tên method cũ, team, profile thiếu |
| `lib/src/profile_report.dart`, `bin/inspect_profile.dart` | Báo cáo và công cụ dòng lệnh |
| `test/` | 69 test |

## Kiểm chứng tự động

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/52-ios-signing
dart pub get
dart analyze
dart test
```

Kỳ vọng: `No issues found!` và `All tests passed!`. Không test nào dùng mạng, đồng hồ thật hay thiết bị (thời điểm được truyền vào).

## Dùng công cụ đọc profile

```bash
dart bin/inspect_profile.dart samples/staging-adhoc-old.plist \
  --now=2026-11-07T00:00:00Z \
  --entitlements=native-config/ios/Runner.entitlements.example \
  --device=00008030-000000000000000C
```

Mã thoát: 0 ổn, 1 có cảnh báo (sắp hết hạn, thiếu entitlement, thiếu thiết bị), 2 lỗi dùng hoặc file không đọc được. Với profile thật, giải CMS trước trên macOS:

```bash
security cms -D -i ten-profile.mobileprovision > profile.plist
dart bin/inspect_profile.dart profile.plist
```

## Dùng cấu hình trên máy macOS (tự thử)

Cần: máy macOS với Xcode, tài khoản Apple Developer Program, vai trò đủ quyền (Account Holder hoặc Admin để tạo certificate Distribution, App ID và profile), và app Flutter đã có flavor ở bài 50.

1. Sinh runner một lần: `flutter create --platforms=ios,android --project-name toi_noi_52 --org vn.toinoi .`
2. Trên Apple Developer, đăng ký App ID `vn.toinoi.customer.staging` và `vn.toinoi.customer`, bật Push Notifications và Associated Domains; tạo hai profile đặt tên đúng như trong xcconfig và ExportOptions.
3. Chép `Runner.entitlements.example` thành `ios/Runner/Runner.entitlements`, đoạn `UIBackgroundModes` vào `Info.plist`; dán xcconfig vào configuration tương ứng; thay `ABCDE12345` bằng Team ID thật.
4. `flutter build ipa --flavor staging --export-options-plist=native-config/ios/ExportOptions-adhoc.plist.example` (đổi tên bỏ đuôi `.example` nếu muốn). Lệnh này cần Xcode và chứng chỉ thật; chưa chạy ở đây.

## Phiên bản đã kiểm

Flutter 3.47.4 / Dart 3.13.3. `xml` 7.1.0, `lints` 6.1.0, `test` 1.33.0. Chính sách Apple (phí, giới hạn thiết bị, tên method) đối chiếu với trang chính thức tháng 11/2026.

## Chưa làm

- Chạy `xcodebuild archive`, `-exportArchive`, `flutter build ipa` và cài IPA lên thiết bị.
- Đọc `.mobileprovision` thật; xác thực chữ ký CMS; so khớp certificate trong `DeveloperCertificates` với keychain.
- App của tài xế (`vn.toinoi.driver.*`) có bộ cấu hình riêng, cùng cấu trúc; lab chỉ dựng app khách.
- Profile `staging` cho TestFlight (cùng bundle ID staging, method `app-store-connect`): bài 56.
- Tự động hoá bằng fastlane `match` (bài 53) và giải mã từ secrets trên CI (bài 54).
- Cài đặt thật của Xcode scheme và configuration theo flavor: bài 50.
