# Lab 42: bảo mật mobile cho app khách và app tài xế

Lab của bài **FL-42** trong loạt **Flutter qua Tới Nơi**: [Bảo mật mobile: secure storage, pinning, obfuscation và OWASP MASVS](../../../api/seed/content/flutter-bao-mat-mobile-secure-storage-pinning-obfuscation-masvs.md). Đây là bài kết Phần 5.

Lab là project **Dart thuần** (không có Flutter, không cần thư mục `android/` hay `ios/`, không cần `flutter create`). Toàn bộ là phòng thủ cho **chính app của ta**. Dữ liệu, số điện thoại, toạ độ và mọi giá trị trong test đều giả; không có khoá, token hay chứng chỉ thật.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/src/redactor.dart` | `Redactor`: che số điện thoại, JWT, Bearer, trường nhạy cảm theo tên, token dạng chuỗi dài; làm tròn toạ độ về 2 chữ số thập phân |
| `lib/src/der.dart` | Lấy khối `SubjectPublicKeyInfo` từ chứng chỉ X.509 DER |
| `lib/src/cert_pin.dart` | `CertPin` (SPKI hoặc hash chứng chỉ) và `PinPolicy` (tối thiểu 2 ghim mỗi host, kiểm hạn, host lạ bị từ chối) |
| `lib/src/pinned_adapter.dart` | `createPinnedAdapter` (ghim ở bước bắt tay TLS) và `createCheckAfterSendAdapter` (`validateCertificate`, giữ để so sánh) cho `dio` |
| `lib/src/obfuscation_map.dart` | `ObfuscationMap`: đọc file của `--save-obfuscation-map` và dịch stack trace |
| `lib/src/storage_policy.dart` | Dữ liệu nào được lưu ở đâu (RAM, Keychain/Keystore, database, preferences, hoặc không được giữ) |
| `lib/src/integrity_policy.dart` | Tín hiệu root/jailbreak là gợi ý: app phản ứng bằng báo server hoặc đòi xác thực lại, không chặn |
| `lib/src/threat_model.dart` | Mô hình đe doạ app khách và app tài xế: tài sản, kẻ tấn công, biện pháp, nơi thực thi |
| `lib/src/masvs_checklist.dart` | 24 control OWASP MASVS v2 và mức đã làm của Tới Nơi |
| `tool/obfuscation_target.dart` | Chương trình nhỏ để đo `--obfuscate` |
| `native-config/android/` | Mảnh `AndroidManifest` tắt backup và luật `data_extraction_rules.xml` loại hai tệp của secure storage (`FlutterSecureStorage.xml`, `FlutterSecureKeyStorage.xml`) |
| `test/support/` | Sinh CA và chứng chỉ **chỉ dùng cho test** bằng `openssl`, server HTTPS trên localhost |

## Chạy và kiểm chứng

Cần Dart 3.13 (đi cùng Flutter 3.47 stable) và lệnh `openssl` trên PATH (macOS, Linux có sẵn). Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/42-mobile-security
dart pub get
dart analyze
dart test
```

Kỳ vọng: `No issues found!` và `All tests passed!` với **86 test** (`redactor` 31, `policy` 23, `cert_pin` 11, `pinned_adapter` 13, `obfuscation` 8). Chạy hết khoảng 10 đến 35 giây, phần lớn là ba lần biên dịch AOT của `obfuscation_test.dart`. Mức kiểm chứng: **Kiểm chứng tự động** cho toàn bộ logic. Phần cần thiết bị được nêu ở mục "Chưa làm".

Đã kiểm với Flutter 3.47.4, Dart 3.13.3, `dio` 5.11.1, `crypto` 3.0.8, `test` 1.33.0, `lints` 6.1.0, LibreSSL 3.3.6 (macOS).

## Về chứng chỉ trong test

`test/support/test_certs.dart` sinh lúc chạy một CA và các chứng chỉ lá cho `localhost`, vào thư mục tạm và xoá sau khi xong. Chúng là **chứng chỉ test**: khoá riêng của chúng không phải bí mật, chỉ sống trong một lần chạy và chỉ hợp lệ cho `localhost` và `127.0.0.1`. Không có file PEM nào được commit.

Test còn dựng chứng chỉ có SAN lạ, chuỗi lá + trung gian, và một `HttpClient` tin CA test cộng `badCertificateCallback` để chứng minh callback không chạy (không ra mạng ngoài). Lưu ý khi tự sinh chứng chỉ: `openssl x509 -req` của LibreSSL ký bằng SHA-1 nếu không có `-sha256`, và client Dart từ chối chữ ký đó; chứng chỉ lá còn cần `authorityKeyIdentifier` để chuỗi qua được bước kiểm tra. Hai chi tiết này đã được thử, không đoán.

## Về obfuscation

`obfuscation_test.dart` biên dịch `tool/obfuscation_target.dart` bằng `dart compile aot-snapshot --extra-gen-snapshot-options=...` rồi chạy bằng `dartaotruntime`. Đó là bộ biên dịch AOT của Dart mà `flutter build --obfuscate` dùng, nhưng **không phải** một lần `flutter build`. Test dùng `--obfuscate --strip`: `flutter_tools` mặc định cũng strip snapshot (đọc ở `flutter_tools/lib/src/base/build.dart`: iOS strip sau khi xuất dSYM, Android để AGP lo, nền tảng khác thêm `--strip`). Chưa thử `--split-debug-info` và `flutter symbolize`: `dart compile` trên macOS sinh snapshot Mach-O mà `flutter symbolize` không đọc được (đã thử, báo "Cannot locate isolate instructions section"). Phần dịch stack trace được kiểm bằng bản đồ `--save-obfuscation-map`.

## Chưa làm

- Chưa chạy trên iOS Simulator, Android emulator hay thiết bị thật: Keychain, Keystore, backup Android, ATS và kho CA thật của hệ điều hành chưa được kiểm.
- Chưa có bộ phát hiện root/jailbreak, Play Integrity hay App Attest: `IntegritySignal` chỉ mô hình hoá cách app dùng tín hiệu.
- Chưa có hash ghim thật: Tới Nơi không có server thật. Ghim chỉ được kiểm với chứng chỉ test trên localhost.
- Chuỗi lá + chứng chỉ trung gian chỉ kiểm trên macOS (callback chỉ nhận lá, nên không ghim được trung gian); chưa kiểm trên Linux.
- Chưa đo trên `flutter build apk/ipa --obfuscate --split-debug-info`.
- `native-config/android` chưa được dán vào một runner thật; chỉ có test đọc file để kiểm giá trị.
- Chưa có khoá gắn sinh trắc học, xác thực cục bộ, che màn hình ở app switcher.
