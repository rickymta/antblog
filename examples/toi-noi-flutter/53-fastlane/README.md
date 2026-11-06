# Lab 53: fastlane cho Flutter: match, gym, pilot, supply

Lab của bài **FL-53** trong loạt **Flutter qua Tới Nơi**: [fastlane cho Flutter: match, gym, pilot, supply tự động hoá build](../../../api/seed/content/flutter-fastlane-match-gym-pilot-supply-tu-dong-hoa-build.md).

Lab gồm hai phần. `native-config/` là bộ file fastlane của Tới Nơi (`Fastfile` cho iOS và Android, `Appfile`, `Matchfile`, `Gemfile`, `.env.example`, mẫu `.gitignore`) để chép vào thư mục `ios/` và `android/` của app. Phần còn lại là một project Dart thuần (`toi_noi_53`): `lib/release_plan.dart` chọn lane theo flavor, `lib/native_config_audit.dart` đọc các file trong `native-config/` và đối chiếu với kế hoạch đó, còn `test/` chạy cả hai. Mọi số liệu và tên miền là minh hoạ (`toinoi.example`).

## Mức kiểm chứng: đọc kỹ

Máy viết lab này **không** có tài khoản Apple Developer hay Google Play Console, không có fastlane (`which fastlane` trống), và Ruby hệ thống là 2.6.10 trong khi `fastlane` 2.241.2 yêu cầu Ruby từ 3.2.0 (rubygems.org, kiểm tại 11/2026). Ta không cài thêm gì. Vì vậy **không lane nào đã được chạy**.

| Phần | Đã làm ở đây? |
|---|---|
| `dart analyze` | Có: 0 issue |
| `dart test`: chọn lane theo flavor, kế hoạch lệnh, 75 test | Có |
| Đọc `native-config/`: đủ lane, bundle ID khớp curriculum §3.2, track Play là `internal`, `match` chỉ đọc trong lane `beta`, bí mật chỉ đến từ ENV, không có chuỗi giống bí mật | Có (test Dart) |
| Ca sai: sửa đúng một chỗ của file thật thì audit phải đỏ | Có (test Dart) |
| Cú pháp Ruby bằng `ruby -c` (Ruby 2.6.10 của hệ thống) | Có: 6 file `Syntax OK` (Fastfile, Appfile, Matchfile, hai Gemfile); test tự bỏ qua nếu máy không có `ruby` |
| `bundle install`, `bundle exec fastlane ...` | **Chưa chạy**: không có Ruby >= 3.2 và không cài mới |
| Tên action và tham số fastlane đúng với phiên bản đã cài | **Chưa chạy**: đối chiếu với trang docs.fastlane.tools (11/2026), chưa với bản cài thật |
| `match` đọc hoặc ghi kho chứng chỉ, `setup_ci` tạo keychain, `update_code_signing_settings` sửa project | **Chưa chạy** |
| `flutter build ipa` / `flutter build appbundle` với flavor, `gym` xuất IPA | **Chưa chạy**: không Xcode đầy đủ, không Android SDK |
| Upload lên TestFlight, Google Play | **Chưa chạy**: không tài khoản |

Mức theo curriculum: **Kiểm chứng cấu hình (CH)**. Test chứng minh file **có** các tính chất ở trên; chúng không chứng minh lane **chạy được**. Lần chạy đầu tiên phải là trên máy bạn, từng lane một, trước khi đưa lên CI (bài 54).

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/release_plan.dart` | `Flavor`, `AppKind`, `applicationId`, `LaneSpec`, `lanesFor`, `requiredEnv`, `missingEnv`, `planRelease` |
| `lib/native_config_audit.dart` | `auditNativeConfig`: trả danh sách `Problem`; các hàm đọc `laneBody`, `callBody`, `appIdsIn`, `envReadIn`... |
| `bin/plan.dart` | In lệnh fastlane cho một lần phát hành, không chạy gì |
| `native-config/ios/fastlane/{Fastfile,Appfile,Matchfile}`, `native-config/ios/Gemfile` | Lane `beta` (match chỉ đọc, `flutter build ipa`, `gym`, `upload_to_testflight`) và lane quản trị `sync_certs` |
| `native-config/android/fastlane/Fastfile`, `native-config/android/Gemfile` | Lane `internal` (`flutter build appbundle`, `upload_to_play_store` track `internal`) |
| `native-config/.env.example` | Tên biến môi trường, mọi giá trị để trống |
| `native-config/{ios,android}/gitignore.fragment.example` | Dòng cần thêm vào `ios/.gitignore`, `android/.gitignore` (chặn `.env`) |
| `test/release_plan_test.dart` | 15 test: logic chọn lane, biến thiếu, build number biên |
| `test/native_config_test.dart` | 28 test trên file thật, kể cả `ruby -c` |
| `test/audit_rules_test.dart` | 32 test ca sai và trình phân tích |

## Kiểm chứng tự động

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/53-fastlane
dart pub get
dart analyze
dart test
dart run bin/plan.dart staging customer 42
```

Kỳ vọng: `No issues found!`, `All tests passed!` (75 test) và hai dòng `bundle exec fastlane ...`. Test đọc `native-config/` theo đường dẫn tương đối nên phải chạy từ thư mục lab. Test không dùng mạng.

## Dùng bộ file này cho app thật (tự thử)

Chép `native-config/ios/` vào `ios/` và `native-config/android/` vào `android/` của app (mỗi thư mục có `Gemfile` và `fastlane/`), cần Ruby >= 3.2 và Xcode đầy đủ cho iOS:

```bash
cd ios && bundle install
bundle exec fastlane ios sync_certs app:customer flavor:staging build_number:1   # quản trị viên, một lần
bundle exec fastlane ios beta app:customer flavor:staging build_number:42
cd ../android && bundle install
bundle exec fastlane android internal app:customer flavor:staging build_number:42
```

Thêm dòng trong `gitignore.fragment.example` vào `ios/.gitignore` và `android/.gitignore`, rồi chép `native-config/.env.example` thành `ios/.env` và `android/.env` và điền: fastlane chỉ tìm `.env` trong `fastlane/` và thư mục cha của nó, nên `.env` ở gốc app không được nạp. Sau `bundle install` lần đầu, **commit `Gemfile.lock`** (lab không có vì không chạy được `bundle`) để mọi máy và CI dùng cùng một bản. Lane giả định bài 25 (`config/<flavor>.json`), bài 50 (flavor Gradle và scheme Xcode), bài 51 (ký AAB bằng upload keystore) và bài 52 (Xcode project ký thủ công, `Release-<flavor>.xcconfig`) đã làm. Xcconfig của bài 52 cho staging trỏ profile ad-hoc, không tải lên App Store Connect được, nên lane `beta` gọi `update_code_signing_settings` để ép `Release-<flavor>` ký bằng profile của `match`; lệnh này sửa `ios/Runner.xcodeproj` trong thư mục làm việc, trên máy cá nhân hãy `git checkout` file đó sau khi chạy. `gym` truyền `scheme: <flavor>` vì project có ba scheme (bài 50). Trên CI, xác thực vào repo `match` bằng `MATCH_GIT_BASIC_AUTHORIZATION` (HTTPS) hoặc `MATCH_GIT_PRIVATE_KEY` (deploy key). Đường dẫn trong Fastfile theo tài liệu fastlane: mã Fastfile và `sh` chạy trong `fastlane/`, còn tham số đường dẫn của action tính từ thư mục chứa `fastlane/`.

## Phiên bản đã kiểm

Flutter 3.47.4 / Dart 3.13.3 (lab không phụ thuộc Flutter, chỉ dùng `dart`). `lints` 6.x, `test` 1.x. Ruby hệ thống 2.6.10 chỉ dùng cho `ruby -c`. `Gemfile` ghim `fastlane ~> 2.241.2`, chỉ nhận bản vá (2.241.2 là bản mới nhất ghi trên rubygems.org tại 11/2026). Đường dẫn AAB theo flavor lấy từ mã nguồn `flutter_tools` 3.47.4 (`build/app/outputs/bundle/<flavor>Release/app-<flavor>-release.aab`).

## Chưa làm

- Chạy bất kỳ lane nào, và mọi điều trong bảng "Chưa chạy" ở đầu README.
- `ExportOptions.plist` và cấu hình Xcode ký thủ công (bài 52); keystore và `key.properties` (bài 51).
- `Gemfile.lock`: chưa sinh được (không có Ruby >= 3.2); bạn tạo bằng `bundle install` rồi commit.
- Số build tự tăng từ CI (`run_number`): lane nhận `build_number` làm tham số (bài 54).
- Phát cho tester ngoài (`distribute_external`), export compliance, Beta App Review (bài 56); promote giữa track Play (bài 57); metadata, ảnh chụp và gửi App Review (bài 58).
- Lưu `match` trên S3: chỉ có dòng ví dụ trong `Matchfile`, ở dạng chú thích.
