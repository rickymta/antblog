# Lab 57: Google Play, internal/closed/open/production và staged rollout

Lab của bài **FL-57** trong loạt **Flutter qua Tới Nơi**: [Google Play: internal, closed, open testing, production và staged rollout](../../../api/seed/content/flutter-google-play-internal-closed-open-production-staged-rollout.md).

Lab là một project Dart thuần (`toi_noi_57`), không có runner Android và không dùng `toi_noi_core` (bài không đụng domain chuyến). Nó gồm:

- `lib/rollout_plan.dart`: `RolloutPlan` (bước phần trăm, điều kiện dừng theo crash-free, halt kể cả bản đã 100%, resume ở mức chọn, `fixForward`).
- `lib/track_promotion.dart`: `TrackPromotion` (máy trạng thái bốn track) và `preflight` (các điều chặn trước khi promote).
- `lib/manifest_permissions.dart`: đọc `<uses-permission>` từ manifest.
- `lib/data_safety.dart`: `DataSafetyDraft`, đối chiếu bản khai với quyền trong manifest.
- `native-config/AndroidManifest.xml`: manifest mẫu của app tài xế (quyền, không có `ACCESS_BACKGROUND_LOCATION`).
- `native-config/fastlane/Fastfile.promote.example`: lane `promote` (supply), `import` vào Fastfile Android của [lab 53](../53-fastlane/).
- `bin/check.dart`: in quyền, đối chiếu Data safety và một lịch rollout minh hoạ.

Mọi con số về ngưỡng crash-free, cỡ mẫu và bước phần trăm là **minh hoạ** của Tới Nơi. Số tester, số ngày, target API lấy từ trang trợ giúp Play Console (kiểm tại 11/2026), có thể đã đổi khi bạn đọc.

## Mức kiểm chứng: cấu hình (CH), đọc kỹ

Máy viết lab này **không** có tài khoản Google Play Console, không có Android SDK, không cài fastlane (Ruby hệ thống là 2.6.10).

| Phần | Đã làm ở đây? |
|---|---|
| `dart analyze`: 0 issue | Có |
| `dart test`: 71 test (rollout, promote, Data safety, lane `promote`) | Có |
| Ngưỡng dừng và tiến của rollout, ca biên (bằng ngưỡng, thiếu mẫu, versionCode bằng/thấp hơn) | Có (test) |
| Bảng đường đi giữa bốn track: kiểm cả 16 cặp (từ, tới) | Có (test) |
| Điều kiện closed test (12 tester, 14 ngày), mốc 13/11/2023, target API 36 | Có (test) theo trang trợ giúp 11/2026; **không** kiểm với Play thật |
| Đọc quyền manifest mẫu, bỏ chú thích, `tools:node="remove"`, đối chiếu Data safety | Có (test) |
| Lane `promote`: cú pháp (`ruby -c`) và logic, chạy Ruby thật với bản giả của `UI`, `lane`, `upload_to_play_store` | Có (test; tự bỏ qua nếu không có `ruby`) |
| Tên action và tham số supply đúng với bản fastlane đã cài | **Chưa**: không cài fastlane; đối chiếu trang docs.fastlane.tools và mã nguồn supply (11/2026) |
| Tên track Play API (`alpha` cho closed, `internal`): trang Google ghi internal là `qa` và closed tạo tay có tên tuỳ ý, supply lọc theo tên chính xác | **Chưa kiểm**: chạy `tracks.list` cho package của bạn trước lần chạy đầu |
| Manifest đã gộp với manifest của plugin (thứ Play nhìn thấy) | **Chưa**: cần Gradle và Android SDK; lab chỉ đọc manifest nguồn |
| Pre-launch report, form Data safety, khai báo foreground service và vị trí nền trên Console | **Chưa**: cần tài khoản |
| Promote thật giữa track, staged rollout thật, halt thật | **Chưa**: cần tài khoản và bản build đã ký |
| Số crash-free thật từ Crashlytics hoặc Play Console | **Chưa**: `RolloutObservation` nhận số do ta đưa vào |

Test chứng minh logic và file cấu hình **có** các tính chất ở trên; chúng không chứng minh Google Play **chấp nhận** chúng.

## Chạy và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/57-google-play
dart pub get
dart analyze
dart test
dart run bin/check.dart
```

Kỳ vọng: `No issues found!`, `All tests passed!` (71 test), và `bin/check.dart` in sáu quyền của manifest mẫu, "Data safety: khớp." và bốn dòng rollout (giữ, tiến, giữ, dừng). Test đọc `native-config/` theo đường dẫn tương đối nên phải chạy từ thư mục lab. Test không dùng mạng.

## Dùng lane promote cho app thật (tự thử)

Đặt `native-config/fastlane/Fastfile.promote.example` thành `android/fastlane/Promote.fastfile` và thêm `import "Promote.fastfile"` lên đầu `android/fastlane/Fastfile` của lab 53 (docs.fastlane.tools khuyên đặt `import` phía trên các lane; đường dẫn tương đối của `import` chưa được kiểm). Lane dùng lại `env!` và `pick_target` của Fastfile đó. Không dán thẳng vào Fastfile của lab 53, vì audit của lab 53 cấm `track_promote_to` và các chuỗi `"production"`, `"beta"`, `"alpha"` trong file ấy; chỉ đặt file trong app của bạn, đừng đặt vào `native-config` của lab 53 (test danh sách file của lab đó sẽ đỏ); ngược lại audit đó chỉ đọc Fastfile chính, nên test của lab 57 mới là thứ canh lane `promote`.

```bash
bundle exec fastlane android promote app:customer flavor:staging from:internal to:closed version_code:43
bundle exec fastlane android promote app:customer flavor:prod from:closed to:production version_code:43 rollout_percent:1
```

Cần Ruby >= 3.2 (theo fastlane 2.241 ở lab 53) và biến `PLAY_JSON_KEY_DATA`. Chạy lần đầu trên track internal hoặc closed, từng lane một, trước khi đưa lên CI.

## Phiên bản đã kiểm

Flutter 3.47.4 / Dart 3.13.3 (lab chỉ dùng `dart`). `lints` ^6.1.0, `test` ^1.33.0. Ruby hệ thống 2.6.10 chỉ dùng cho `ruby -c` và harness của test. `flutter.targetSdkVersion` trong `flutter_tools` 3.47.4 là 36 (`gradle/src/main/kotlin/FlutterExtension.kt`), khớp mốc target API 36 của trang trợ giúp.

## Chưa làm

- Mọi điều trong bảng "Chưa" ở trên.
- Lane halt hoặc resume rollout bằng fastlane: docs.fastlane.tools liệt kê `halted` là giá trị hợp lệ của `release_status` nhưng không có ví dụ, nên lab không viết lane chưa chắc chạy. Halt làm trên Console. `RolloutPlan.halt()` mô hình hoá cả halt bản đã 100% (trang "Halting a fully rolled-out release"), trừ lần phát hành đầu của track; internal không mô hình hoá.
- Lần phát hành production đầu tiên (Play không cho staged rollout với lần xuất bản đầu): làm tay trên Console.
- Form Data safety ở dạng CSV import, Permissions Declaration Form và video minh hoạ: cần tài khoản.
- Đo crash-free tự động và nối với `RolloutObservation` (bài 48 và bài 59 mô tả hướng).
