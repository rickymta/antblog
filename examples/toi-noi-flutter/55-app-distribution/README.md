# Lab 55: phân phối nội bộ qua Firebase App Distribution

Lab của bài **FL-55** trong loạt **Flutter qua Tới Nơi**: [Firebase App Distribution: phân phối nội bộ cho tester iOS và Android](../../../api/seed/content/flutter-firebase-app-distribution-phan-phoi-noi-bo-tester.md).

Lab có hai phần. `native-config/` là bộ file mẫu đưa APK và IPA ad-hoc của flavor `staging` (bài 54) tới nhóm tester `qa-noi-bo`: đoạn dán hai job (`notes`, `distribute`) vào cuối workflow của bài 54, đoạn dựng APK, hai lane fastlane, cấu hình Gradle cho SDK feedback và file cấu hình Firebase mẫu. Ta không sửa lab 54: đoạn dán nằm ở đây và được test ghép vào workflow của lab 54 trong bộ nhớ. Phần còn lại là project Dart thuần (`toi_noi_55`): `ReleaseNotes` sinh release notes tiếng Việt từ danh sách commit, `DistributionConfig` kiểm file cấu hình, và các test đọc, rồi chạy thật các script `run` của workflow với lệnh giả.

Mọi số liệu và tên miền là minh hoạ (`toinoi.example`). App ID Firebase trong file mẫu là giá trị giả (`1:000000000000:...:FAKE-...-REPLACE-ME`), cùng giá trị với file `.example` của bài 50.

## Mức kiểm chứng: đọc kỹ (Kiểm chứng cấu hình)

Máy viết lab **không** có tài khoản Firebase, không có `firebase` CLI, không có Ruby >= 3.2 (hệ thống là 2.6.10; plugin fastlane yêu cầu Ruby >= 3.2), không có Android SDK và Xcode chưa đầy đủ. Ta không cài thêm gì. Vì vậy **chưa có lần phân phối thật nào**.

| Phần | Đã làm ở đây? |
|---|---|
| `dart analyze --fatal-infos` và `dart format --set-exit-if-changed` | Có: 0 issue |
| `ReleaseNotes`: lọc commit nội bộ, scope app kia, làm sạch, bỏ trùng, cắt độ dài theo ký tự Unicode (emoji, chuỗi NFD), tiếng Việt, ca biên | Có (test) |
| `DistributionConfig`: chỉ flavor `staging`, App ID đúng dạng và giả, Application ID khớp bài 50, tham số của `firebase appdistribution:distribute` | Có (test) |
| Đoạn dán là YAML hợp lệ, ghép được vào workflow bài 54 (job `needs` đều có thật, tên artifact tải về khớp tên đã upload, trigger không đổi), và **bộ audit workflow của lab 54 (`toi_noi_54`) không tìm thấy gì** trên bản đã ghép; ca sai (thêm `pull_request`, secret ở cấp job, bỏ kiểm rỗng) bị audit bắt | Có (test tĩnh) |
| Job có secret: environment `staging`, secret chỉ trong `env` của bước `run`, không chạy mã của repository (`dart`, `flutter`, script trong repo), không `FIREBASE_TOKEN`, `login:ci`, `--token`, bước dọn `if: always()` | Có (test tĩnh) |
| **Chạy thật các script `run`**: bước tạo release notes (khoảng commit từ tag `staging-v*` trước đó: ca không có tag, ca commit nằm trên tag, ca merge; build number và version sai bị từ chối) và bước phân phối (ghi và xoá file khoá, quyền `600`, không lộ secret ra log, dừng khi thiếu file) | Có (test); `dart` và `firebase` là **lệnh giả**, nên chứng minh hành vi script, không chứng minh Firebase hay GitHub chạy nó |
| Fastfile: lane, tham số (`download_udids` dùng `project_number`), chỉ đọc mục `staging`, truyền tường minh file service account; `ruby -c` | Cú pháp: chạy `ruby -c` bằng Ruby 2.6.10 (test tự bỏ qua nếu máy không có `ruby`). **Chưa chạy lane** |
| Quét `native-config/` để chặn khoá riêng, token, API key, email ngoài tên miền ví dụ | Có (test) |
| Cú pháp bằng `actionlint` | **Chưa**: máy không có `actionlint` |
| Workflow ghép chạy trên GitHub, `needs` giữa các job, `actions/download-artifact` tải theo `name`, environment giới hạn tag | **Chưa** |
| `firebase appdistribution:distribute` thật, service account thật, nhóm tester thật, mail mời, cài trên máy | **Chưa**: cần tài khoản Firebase của bạn |
| Đăng ký UDID, thêm vào profile, build lại IPA | **Chưa**: cần tài khoản Apple Developer và macOS |
| In-app feedback SDK (Android), Crashlytics từ bản nội bộ | **Chưa**: chỉ kiểm cấu hình Gradle bằng đọc file; không build |
| `flutter build apk --flavor staging` và đường dẫn `app-staging-release.apk` | **Chưa build**: đường dẫn suy ra từ mã nguồn công cụ Flutter 3.47.4 (`gradle.dart`, hàm `listApkPaths`) |

Mức theo curriculum: **Kiểm chứng cấu hình (CH)**.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `native-config/.github/workflows/release-staging.distribute-jobs.example.yml` | Đoạn dán cuối mục `jobs:` của `release-staging.yml` (bài 54): job `notes` (không secret, chạy `dart run` của repo) và job `distribute` (environment `staging`, có secret, không chạy mã repo) |
| `native-config/.github/workflows/build-apk-steps.example.yml` | Hai bước (và upload symbols) dán vào job `android` của bài 54 để có APK; bài 54 chỉ dựng AAB |
| `native-config/app-distribution.example.json` | App ID Firebase (giả), Application ID, alias nhóm `qa-noi-bo`; chỉ có `staging` |
| `native-config/{ios,android}/fastlane/Distribute.fastfile`, `Pluginfile` | Lane `distribute_staging` (cả hai nền tảng), `download_udids` (iOS); plugin `firebase_app_distribution` |
| `native-config/android/app-distribution.gradle.kts.example` | SDK App Distribution: bản api cho mọi variant, bản đầy đủ chỉ cho `staging` |
| `lib/src/release_notes.dart` | `Commit`, `classifyCommit`, `ReleaseNotes`, `truncateAtWord` |
| `lib/src/distribution_config.dart` | `DistributionConfig`, `DistributionTarget`, `cliArguments` |
| `lib/src/cli.dart`, `bin/release_notes.dart` | Đọc `git log` từ stdin, in release notes |
| `test/release_notes_test.dart` | 33 test |
| `test/distribution_config_test.dart` | 17 test |
| `test/cli_test.dart` | 7 test (kể cả chạy tiến trình thật) |
| `test/workflow_jobs_test.dart` | 28 test tĩnh trên đoạn dán, gồm audit của lab 54 trên bản đã ghép |
| `test/native_config_test.dart` | 21 test: đoạn dựng APK, fastlane, Gradle, quét bí mật |
| `test/workflow_scripts_test.dart` | 17 test chạy script thật trong thư mục tạm |

## Kiểm chứng tự động

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/55-app-distribution
dart pub get
dart format --output=none --set-exit-if-changed .
dart analyze --fatal-infos
dart test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (123 test, khoảng 20 giây vì có test tạo repo git tạm). Cần `bash`, `git` và `jq` cho nhóm chạy script (không có thì nhóm đó tự bỏ qua). Test cần repository đầy đủ vì đọc file và dùng bộ audit của lab 50 và lab 54. Không test nào dùng mạng, thiết bị hay đồng hồ thật.

Thử tạo release notes từ log của chính repository:

```bash
git log -n 30 --no-merges --pretty=format:'%h%x09%s' | \
  dart run bin/release_notes.dart --app customer --version 1.4.0 --build 42
```

Notes ra stdout, một dòng tóm tắt (số dòng, số commit bị bỏ) ra stderr. `--flavor dev` hoặc `prod` thoát với mã 64: chỉ `staging` được phân phối.

## Dùng cho repository ứng dụng thật (tự làm, chưa kiểm)

Giả định giống lab 54: gốc repository là workspace `toi-noi-app` (có `apps/customer`), runner `android/` và `ios/` đã commit, workflow `release-staging.yml` của bài 54 đã chạy được, IPA ad-hoc ký theo bài 52.

1. **Firebase**: trong Firebase console, đăng ký app Android `vn.toinoi.customer.staging` và app iOS cùng bundle ID (bài 50), mở App Distribution và bấm Get started cho từng app. Tạo nhóm: `firebase appdistribution:group:create "QA nội bộ" qa-noi-bo`, thêm tester: `firebase appdistribution:testers:add --group-alias=qa-noi-bo mai@toinoi.example`.
2. **Service account**: trong Google Cloud console, tạo service account, gán vai trò **Firebase App Distribution Admin**, tạo khoá JSON; dán nguyên nội dung vào environment secret `FIREBASE_SERVICE_ACCOUNT_JSON` của environment `staging`. Khoá này cấp quyền quản trị App Distribution: không commit, không gửi qua chat.
3. **Cấu hình**: chép `native-config/app-distribution.example.json` thành `app-distribution.json` ở gốc repository ứng dụng và điền App ID thật (App ID không phải bí mật). File này khác với `google-services.json`; `.gitignore` của lab chặn bản không có đuôi `.example` trong `native-config/`.
4. **APK**: dán các bước trong `build-apk-steps.example.yml` vào job `android` của `release-staging.yml`, ngay sau bước "Build AAB".
5. **Công cụ release notes**: chép `lib/`, `bin/` và `pubspec.yaml` của lab thành `tool/release_notes/` trong repository ứng dụng. `ReleaseNotes` chỉ dùng `AppKind` của lab 53; trong repository thật hãy thay bằng enum của app (hoặc chép enum đó vào) và bỏ `toi_noi_53` khỏi `pubspec.yaml` tương ứng.
6. **Hai job**: dán `release-staging.distribute-jobs.example.yml` vào cuối mục `jobs:` của `release-staging.yml`, sau job `ios`. Lý do đặt chung workflow thay vì workflow riêng chạy bằng `workflow_run`: job có secret dùng artifact của chính run này, chạy cùng ref với bản đã dựng nên environment `staging` (giới hạn deployment theo tag `staging-v*`, bài 54) áp dụng đúng, và không phải checkout hay tải mã nào khác. Chạy tay (`workflow_dispatch`) phải chọn tag; chạy lại phân phối bằng Re-run failed jobs. Tag `staging-v1.4.0` kích hoạt cả chuỗi.
7. **iOS**: lần đầu mỗi tester phải mở email mời bằng Safari trên iPhone, cài Firebase profile và đăng ký thiết bị. Sau đó làm theo vòng UDID trong bài: nhập UDID vào Apple Developer, thêm vào profile ad-hoc, build lại, phân phối lại.
8. **fastlane** (tuỳ chọn, thay cho CLI): `bundle exec fastlane add_plugin firebase_app_distribution` rồi chép `Distribute.fastfile` vào `ios/fastlane/` và `android/fastlane/`, thêm `import("Distribute.fastfile")` vào Fastfile của bài 53. Cần Ruby >= 3.2.

Đường dẫn trong workflow (`apps/customer`, `app-distribution.json`, `tool/release_notes`) tính từ gốc repository ứng dụng.

## Phiên bản đã kiểm

Flutter 3.47.4 stable / Dart 3.13.3 (lab không phụ thuộc Flutter, chỉ dùng `dart`), `yaml` 3.1.4, `args` 2.7, `test` 1.33, `lints` 6.1; `toi_noi_53` và `toi_noi_core` bằng path. Kiểm ngày 07/11/2026 (11/2026):

- `firebase-tools` **15.35.0** (npm `latest`, phát hành 04/11/2026; yêu cầu Node 20, 22 hoặc 24), ghim trong workflow bằng biến `FIREBASE_CLI_VERSION`.
- `fastlane-plugin-firebase_app_distribution` **1.0.0** (rubygems, 04/03/2026; Ruby >= 3.2, fastlane >= 2.232.0); `fastlane` 2.241.2 (rubygems, 26/10/2026).
- GitHub Actions (API `releases/latest`): `actions/checkout` v7.0.1, `actions/download-artifact` v8.0.2, `actions/setup-node` v7.1.0, `actions/upload-artifact` v7.0.2, `subosito/flutter-action` v2.23.0. Tag chưa bất biến; ghim SHA đầy đủ chặt hơn.
- Maven (Google): `com.google.firebase:firebase-appdistribution` và `-api` **16.0.0-beta21** (bản beta, tài liệu ghi SDK Android là beta).
- Giới hạn App Distribution (trang troubleshooting và manage-testers, 11/2026): 500 tester mỗi project, 200 mỗi nhóm; release giữ 150 ngày; tối đa 1.000 release mỗi app; file nhị phân tối đa 2.048 MiB; lời mời hết hạn sau 30 ngày. Giới hạn Apple (developer.apple.com, 11/2026): 100 thiết bị mỗi dòng sản phẩm mỗi năm thành viên.
- Plugin cộng đồng `firebase_app_distribution` trên pub.dev (1.3.4, publisher `helloprimary.care`) **không phải** FlutterFire chính thức và chỉ làm cập nhật trong app: lab không dùng.

## Chưa làm

- Mọi điều trong bảng "Chưa" ở đầu README.
- Job `distribute` vẫn checkout để đọc `app-distribution.json`; ta chỉ chứng minh nó không chạy mã repo bằng kiểm tra tĩnh, không chạy trên GitHub.
- AAB: workflow chỉ phân phối APK. Phân phối AAB cần liên kết Firebase với Google Play (app đã đăng ký và xuất bản trên một track); mô tả trong bài, không có file mẫu.
- Gọi `showFeedbackNotification()` trong `MainActivity` (cần source set riêng cho `staging`) và chuỗi thông báo; chỉ có phần Gradle.
- Tự động hoá vòng UDID bằng Cloud Functions (cần gói Blaze); chưa dùng cảnh báo App Distribution.
- Artifact chỉ giữ 14 ngày (`retention-days` của bài 54): phân phối lại sau thời hạn đó phải build lại.
- Khoá JSON của service account là thông tin dài hạn, một đánh đổi. Workload Identity Federation (OIDC) với Application Default Credentials là hướng cần thử, **chưa kiểm** cách Firebase CLI nhận thông tin đăng nhập kiểu này.
- App tài xế: file cấu hình có mục `driver` nhưng workflow chỉ phân phối `customer`.
- Enterprise distribution (Apple Developer Enterprise Program): Tới Nơi không dùng.
- Crashlytics: chỉ nhắc, không có code (bài 48).
