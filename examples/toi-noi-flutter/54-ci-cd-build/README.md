# Lab 54: workflow phát hành staging của Tới Nơi, kiểm bằng test Dart

Lab của bài **FL-54** trong loạt **Flutter qua Tới Nơi**: [CI/CD build: GitHub Actions macOS/ubuntu, secrets, build number và artifact](../../../api/seed/content/flutter-ci-cd-github-actions-macos-build-ios-android-secrets.md).

Lab có một **workflow mẫu** (`native-config/.github/workflows/release-staging.yml`) và một project Dart thuần (`toi_noi_54`) kiểm nó. Workflow dựng AAB trên ubuntu và IPA trên macOS cho flavor `staging` của app khách (`apps/customer` trong [`toi-noi-app`](../toi-noi-app)), lấy build number từ `github.run_number`, giải mã keystore, certificate và provisioning profile từ secret, rồi upload artifact.

Workflow nằm trong `native-config/`, **không** ở `.github/workflows/` thật của repository này: repository không có secret, và ta không muốn một workflow đòi secret chạy trong CI của blog. Muốn dùng, chép vào `.github/workflows/` của repository ứng dụng.

## Mức kiểm chứng: đọc kỹ (Kiểm chứng cấu hình)

| Phần | Đã làm ở đây? |
|---|---|
| Workflow mẫu là YAML hợp lệ, đọc được bằng package `yaml` | Có (test) |
| Tính chất của workflow: ghim Flutter và action, quyền tối thiểu, chỉ chạy tay hoặc theo tag, secret chỉ ở `env` của bước `run`, không echo secret, có bước dọn keychain `if: always()`, build number một nguồn, lệnh build đúng flavor, artifact | Có (test tĩnh, tái dùng `WorkflowAudit` của lab 33) |
| **Chạy thật các script `run`** (giải mã keystore, ghi `key.properties`, cài keychain, dọn, tính build number) bằng bash trên máy tác giả, với secret **giả** và lệnh `security` **giả** | Có (test); chứng minh hành vi script, không chứng minh GitHub chạy nó |
| Cú pháp bằng `actionlint` | **Chưa**: máy kiểm không có `actionlint`, ta không cài mới. Nếu bạn có: `actionlint examples/toi-noi-flutter/54-ci-cd-build/native-config/.github/workflows/release-staging.yml` |
| Workflow chạy trên GitHub (runner thật, environment thật, artifact thật) | **Chưa**: mẫu không nằm ở `.github/workflows/` và repository này không có secret |
| `flutter build appbundle` / `flutter build ipa --flavor staging` | **Chưa**: máy kiểm không có Android SDK, Xcode chưa đầy đủ. Không có runner `android/`, `ios/` trong `toi-noi-app` (không commit) nên lệnh build không chạy được ở đây |
| Lệnh `security` thật, keychain thật, ký thật | **Chưa**: test dùng `security` giả |
| Environment `staging` có reviewer, secret thật | **Chưa**: cấu hình trong Settings của GitHub, không nằm trong file YAML nên test không kiểm được |
| Thời gian chạy và chi phí phút thật | **Chưa đo**: số phút trong mô hình chi phí là giả định minh hoạ; chỉ đơn giá là số tra từ trang chính thức |

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `native-config/.github/workflows/release-staging.yml` | Workflow mẫu: job `prepare` (build number), `android` (ubuntu), `ios` (macOS) |
| `lib/release_audit.dart` | `ReleaseWorkflow`: bọc `WorkflowAudit` (lab 33, path dependency) và thêm kiểm tra riêng: trigger, phạm vi secret, rò secret ra log, dọn keychain/keystore, build number, lệnh build, artifact, vệ sinh |
| `lib/build_number.dart` | `computeBuildNumber` (cùng phép tính với script), `isStrictlyIncreasing`, trần `versionCode` 2.100.000.000 |
| `lib/cost_model.dart` | Chi phí một lần phát hành theo đơn giá phút của GitHub và Codemagic (milli-USD, số nguyên) |
| `test/sample_workflow_test.dart` | 28 test tĩnh trên workflow mẫu |
| `test/audit_rules_test.dart` | 52 test ca sai: mỗi kiểm tra đỏ khi mẫu bị sửa đúng một chỗ |
| `test/step_scripts_test.dart` | 22 test chạy script thật bằng bash trong thư mục tạm |
| `test/models_test.dart` | 16 test cho build number và chi phí |
| `test/support.dart` | `mutated`, sandbox, `security` giả, chạy script bằng `bash -eo pipefail` (chặt hơn mặc định `bash -e {0}` của GitHub) |

## Chạy và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/54-ci-cd-build
dart pub get
dart format --output=none --set-exit-if-changed .
dart analyze --fatal-infos
dart test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (118 test). Cần `bash`, `base64`, `openssl` (có sẵn trên macOS và Linux); không có `bash` thì nhóm chạy script tự bỏ qua. Test cần repository đầy đủ vì đọc workflow của lab 33, bài 33, `toi-noi-app/apps/customer/config/staging.json` và `pubspec.yaml`. Test không dùng mạng, không đụng keychain thật, không cần thiết bị.

## Dùng workflow mẫu cho repository thật (tự làm, chưa kiểm)

Giả định repository ứng dụng có gốc là workspace `toi-noi-app` (có `apps/customer`), runner `android/` và `ios/` đã commit, flavor `staging` đã cấu hình theo bài 50, ký theo bài 51 và 52 (`android/key.properties` đọc bởi `signingConfigs`; `apps/customer/ios/ExportOptions-adhoc.plist` chép từ `ExportOptions-adhoc.plist.example` của lab 52, profile tên `ToiNoi Customer Staging AdHoc`). Các giả định này nằm ngoài lab 54.

1. Chép file vào `.github/workflows/release-staging.yml`. Với `workflow_dispatch`, theo tài liệu GitHub, file phải nằm trên nhánh mặc định thì nút Run workflow mới xuất hiện.
2. Settings, Environments, tạo `staging`: giới hạn deployment tags theo `staging-v*`, thêm required reviewers và bật prevent self-review. **Giới hạn gói** (docs.github.com, "Deployments and environments", kiểm tại 11/2026): với GitHub Free, Pro và Team, required reviewers chỉ có ở repository công khai; repository riêng cần Enterprise. Giới hạn deployment branches và tags dùng được ở repository riêng với Pro và Team. Không có Enterprise thì chỉ còn luật tag, không có người duyệt trước khi secret được nạp: khi đó ai có quyền đẩy tag `staging-v*` đều kích hoạt được job ký, nên hạn chế quyền tạo tag. Environment secret vẫn dùng được ở các gói đó (tài liệu: Pro, Team hoặc Enterprise).
3. Thêm **environment secrets** (không phải repository secrets):

| Secret | Nội dung |
|---|---|
| `ANDROID_KEYSTORE_BASE64` | upload keystore `.jks` mã hoá base64 một dòng |
| `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD` | thông tin của upload keystore |
| `IOS_CERTIFICATE_P12_BASE64` | certificate distribution xuất `.p12`, base64 |
| `IOS_CERTIFICATE_PASSWORD` | mật khẩu `.p12` |
| `IOS_PROVISIONING_PROFILE_BASE64` | profile ad-hoc của bundle ID staging, base64 |

   Mã hoá file: macOS `base64 -i file -o file.b64`, Linux `base64 -w 0 file > file.b64`; đặt secret bằng `gh secret set TÊN --env staging < file.b64`. Mỗi secret tối đa 48 KB.
4. Tuỳ chọn: biến repository `BUILD_NUMBER_OFFSET` (số nguyên không âm) khi store đã nhận build có số cao hơn `run_number`. Chỉ tăng, không bao giờ giảm.
5. Chạy tay (Actions, Run workflow) hoặc `git tag staging-v0.1.0 && git push origin staging-v0.1.0`. Khi đã giới hạn deployment theo tag, chạy tay từ nhánh sẽ bị environment từ chối: ở ô "Use workflow from" hãy chọn tag `staging-v*`.

Đường dẫn trong workflow (`apps/customer`, `config/staging.json`, `ios/ExportOptions-adhoc.plist`) tính từ gốc repository ứng dụng; nếu đặt workflow trong repository của blog, thêm tiền tố `examples/toi-noi-flutter/toi-noi-app/`.

## Phiên bản đã kiểm

Flutter 3.47.4 stable / Dart 3.13.3, `yaml` 3.1.4, `test` 1.33 (ràng buộc `^1.33.0`), `lints` 6.1.0; `toi_noi_33` bằng path.

Phiên bản action kiểm bằng API GitHub (`api.github.com/repos/<repo>/releases/latest`; máy kiểm không có `gh`, gọi bằng `curl`, cùng dữ liệu) ngày 07/11/2026: `actions/checkout` v7.0.1, `subosito/flutter-action` v2.23.0, `actions/upload-artifact` v7.0.2 (ba bản này lab 33 đã ghim), `actions/setup-java` v6.2.0 (phát hành 04/11/2026; tag trỏ tới một commit có thật). Tag chưa bất biến; ghim SHA đầy đủ chặt hơn. Nhãn runner: `ubuntu-24.04` và `macos-26` (arm64, 3 nhân); readme image kiểm tại 11/2026 ghi macOS 26 có Xcode 26.6 mặc định ở `/Applications/Xcode_26.6.app` và Ubuntu 24.04 có Java 17 mặc định cùng Android SDK. Kiểm lại khi dùng.

## Chưa làm

- Workflow chưa chạy trên GitHub; chưa chạy `actionlint`; chưa build AAB/IPA; chưa dùng `security` thật.
- Chưa có job test/analyze trước khi build: giả định tag chỉ đặt trên commit đã qua workflow CI của bài 33.
- Chưa upload lên store hay Firebase App Distribution (bài 55 đến 57); artifact chỉ nằm trên GitHub.
- Chưa dùng OIDC (không `id-token`): workflow không gọi nhà cung cấp cloud. Khi upload lên Google Cloud/Firebase ở bài sau, OIDC là lựa chọn thay secret dài hạn.
- `key.properties` được đọc như `java.util.Properties` (lab 51): mật khẩu chứa dấu `\` bị diễn giải là ký tự thoát và script chưa escape. Test chỉ kiểm phía shell (ký tự đặc biệt không bị shell diễn giải), không kiểm Gradle đọc lại.
- Profile được chép vào cả `~/Library/MobileDevice/Provisioning Profiles` (đường cũ) và `~/Library/Developer/Xcode/UserData/Provisioning Profiles` (đường Flutter 3.47.4 đọc trong `code_signing.dart`, theo bài 52); chưa kiểm Xcode trên runner thật đọc đường nào.
- Chưa ghim action bằng SHA đầy đủ; test chấp nhận cả tag ba số lẫn SHA.
- Chưa dùng `fastlane match` (bài 53): mẫu đặt certificate và profile trực tiếp trong secret để thấy cơ chế.
- Chưa so thời gian chạy giữa GitHub và Codemagic; mô hình chi phí chỉ so đơn giá.
- Chưa kiểm lệnh `flutter build ipa` có xuất đúng IPA ad-hoc với ExportOptions của bài 52; đường dẫn artifact `build/app/outputs/bundle/stagingRelease/*.aab` và `build/ios/ipa/*.ipa` suy ra từ source công cụ Flutter 3.47.4 (`gradle.dart`, `application_package.dart`), không từ một lần build.
