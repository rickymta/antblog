# Lab 51: ký ứng dụng Android cho Tới Nơi

Lab của bài **FL-51** trong loạt **Flutter qua Tới Nơi**: [Ký ứng dụng Android: keystore, Play App Signing, AAB và versionCode](../../../api/seed/content/flutter-ky-ung-dung-android-keystore-play-app-signing-aab.md).

Lab là một project Dart thuần (`toi_noi_51`), không có runner Android. Nó gồm hai phần:

- **Cấu hình native** trong `native-config/`: `build.gradle.kts` (bản `android/app/build.gradle.kts` có ký bản release) và `key.properties.example`. Test Dart đọc hai file này để kiểm khoá và giá trị quan trọng.
- **Logic và công cụ** trong `lib/`: tính `versionCode` tăng đơn điệu từ `version: x.y.z+build`, đọc `key.properties` như `java.util.Properties`, kiểm văn bản Gradle, và bọc mỏng `keytool`/`jarsigner`/`jar` của JDK để sinh keystore **TEST** trong thư mục tạm.

Phiên bản đã kiểm: Flutter 3.47.4, Dart 3.13.3, OpenJDK/Oracle JDK 21 (`keytool`, `jarsigner`, `jar`).

## Mức kiểm chứng: cấu hình (CH), đọc kỹ

| Phần | Đã làm ở đây? |
|---|---|
| Logic `versionCode` (parse, tăng đơn điệu, cộng hệ số ABI, giới hạn 2.100.000.000) | Có (test) |
| Đọc `key.properties` đúng luật `java.util.Properties`, kể cả lỗi `\u` của đường dẫn Windows | Có (test; kết quả Java đối chiếu bằng cách chạy tay `java.util.Properties` trên JDK 21) |
| `key.properties.example` đủ bốn khoá, chỉ chứa chỗ giữ chỗ | Có (test) |
| Văn bản `build.gradle.kts` có `signingConfigs`, không lui về khoá debug, không gõ cứng mật khẩu hay `versionCode`, có bước dừng sớm | Có (test đọc văn bản và test "mẫu cố ý hỏng") |
| `.gitignore` chặn `key.properties`, `*.jks`, `*.keystore` | Có (test + `git check-ignore`; phần `git check-ignore` bị bỏ qua nếu lab nằm ngoài git repository) |
| Sinh upload keystore TEST, đọc vân tay SHA-256, cỡ khoá RSA, hạn chứng chỉ, xuất PEM | Có (test chạy `keytool` thật; bỏ qua có lý do nếu không có JDK) |
| Ký và kiểm một **AAB giả** bằng `jarsigner`, đối chiếu người ký bằng `keytool -printcert -jarfile` | Có (test chạy JDK thật) |
| Gradle thật chạy `build.gradle.kts` (task `verifyReleaseSigning`, thông báo lỗi khi thiếu `key.properties`, thứ tự với bước biên dịch) | **Một phần, do reviewer**: Gradle 9.3.1 và AGP 9.1.0 biên dịch được script rồi dừng ở "SDK location not found", nên chưa có task graph thật. Máy này không có Android SDK, chưa chạy `flutter build`. Thứ tự cổng so với bước biên dịch chưa quan sát được; `--configuration-cache` chưa thử |
| `flutter build appbundle` ra AAB thật và `apksigner verify --print-certs` trên APK thật | **Chưa**: cần Android SDK (build-tools có `apksigner`) |
| Cài app đã ký lên thiết bị, tải lên Play Console, bật Play App Signing, reset upload key | **Chưa**: cần tài khoản Google Play Console của bạn |
| Bản `profile` và `--split-per-abi` qua Gradle | **Chưa**: chỉ có phép tính `versionCode` tách ABI, đối chiếu mã nguồn Flutter, chưa chạy build |

Test chứng minh cấu hình **có** các tính chất cần thiết; chúng không chứng minh Gradle **chấp nhận** cấu hình đó.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `native-config/build.gradle.kts` | Sao chép thành `android/app/build.gradle.kts` của app khách |
| `native-config/key.properties.example` | Sao chép thành `android/key.properties` rồi điền giá trị thật (file thật không commit) |
| `lib/app_version.dart` | `AppVersion.parse`/`fromPubspec`, `ensureIncreasesFrom`, `splitVersionCode(Abi)`, `VersionCodeRegression` |
| `lib/version_check_cli.dart`, `bin/check_version_code.dart` | Lệnh kiểm trước khi build: build có lớn hơn bản đã phát hành không |
| `lib/key_properties.dart` | `KeyProperties.parse`, `missingKeys` |
| `lib/gradle_signing_audit.dart` | `auditGradleSigning`: danh sách vấn đề trong văn bản Gradle |
| `lib/jdk_tools.dart` | `TestKeystore.generate/inspect/exportCertificatePem/signBundle`, `createStandInBundle`, `verifyBundle`, `bundleSignerSha256` |
| `bin/demo_signing.dart` | Chạy trọn quy trình trên keystore TEST trong thư mục tạm và in vân tay SHA-256 |
| `test/` | 70 test: `app_version` 26, `key_properties` 12, `native_config` 9, `gitignore` 11, `jdk_signing` 12 |

## Chạy và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/51-android-signing
dart pub get
dart analyze
dart test
dart run bin/demo_signing.dart
dart run bin/check_version_code.dart pubspec.yaml --previous 0
```

Kỳ vọng: `No issues found!`, `All tests passed!` (70 test). Không có JDK thì 12 test của `jdk_signing_test.dart` bị bỏ qua kèm lý do (57 test chạy). `bin/demo_signing.dart` in vân tay SHA-256 của upload key TEST; vân tay đổi mỗi lần chạy vì khoá sinh mới.

Keystore TEST luôn nằm trong thư mục tạm của hệ điều hành, mật khẩu ngẫu nhiên sinh lúc chạy, truyền cho `keytool` qua biến môi trường, và bị xoá khi xong. Không file `.jks`, `.keystore` hay `key.properties` thật nào vào repository.

## Dùng trong app thật

```bash
flutter create --platforms=android --org vn.toinoi --project-name toi_noi_customer .
cp native-config/build.gradle.kts android/app/build.gradle.kts
cp native-config/key.properties.example android/key.properties   # rồi điền giá trị thật
flutter build appbundle
```

`flutter create` sinh `android/` (không commit trong lab này). `android/.gitignore` của template Flutter 3.47.4 đã có `key.properties`, `**/*.keystore`, `**/*.jks`. Khi dùng cho cả ba flavor, thêm `productFlavors` của bài 50 vào cùng khối `android {}`.
