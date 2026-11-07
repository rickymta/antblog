# Lab 59: vận hành sau phát hành, force update, remote config và quyết định halt rollout

Lab của bài **FL-59** trong loạt **Flutter qua Tới Nơi**: [Vận hành sau phát hành: force update, remote config, Shorebird và rollback](../../../api/seed/content/flutter-van-hanh-sau-phat-hanh-force-update-remote-config-shorebird-rollback.md). Đây là bài kết Phần 7.

Project Flutter `toi_noi_59` gồm năm khối logic độc lập, đều nằm sau interface tự viết và test bằng bản giả:

1. **Kiểm phiên bản tối thiểu**: `MinVersionPolicy` so `x.y.z+build` với chính sách của server, ra `ok`, `recommend` hoặc `force`. `UpdateGate` **fail open** (server lỗi, hết 3 giây, JSON hỏng hay cấu hình tự mâu thuẫn thì không chặn), `ReminderThrottle` giới hạn tần suất nhắc.
2. **Màn hình chặn**: `ForceUpdateScreen` (không có nút đóng, mở đúng link cửa hàng đã kiểm; `PopScope(canPop: false)` có tác dụng khi push như route) và `UpdateGuard` (đặt ở `MaterialApp.builder`; khi force thì gỡ hẳn Navigator, nên nút Back do hệ thống xử lý).
3. **Feature flag**: `FlagDef`, `Flags`, `FeatureFlags` với mặc định an toàn do app sở hữu, cache, kill switch áp ngay và cờ thường áp ở lần mở app sau. `firebase_remote_config` chỉ có mặt ở `FirebaseRemoteFlagSource`.
4. **Quyết định rollout**: `RolloutGuard` đổi số crash-free thành `wait`, `proceed`, `pause` hoặc `halt` (mức giảm cần 2.000 phiên; sàn tuyệt đối 97% xét từ 300 phiên) cùng việc cụ thể trên Play hoặc App Store, và soạn thông báo.
5. **Bộ lọc trước patch**: `PatchPreflight` phân loại danh sách file thay đổi (Shorebird là lựa chọn không bắt buộc; bộ lọc này không thay công cụ).

## Mức kiểm chứng: đọc kỹ (Kiểm chứng cấu hình)

Máy viết lab **không** có tài khoản Google Play Console, App Store Connect, Firebase hay Shorebird, không có Android SDK, không có Xcode đầy đủ và không có thiết bị.

| Phần | Đã làm ở đây? |
|---|---|
| `flutter analyze` toàn bộ lab, kể cả `FirebaseRemoteFlagSource` và `UrlLauncherOpener` | Có: kiểm kiểu và tên API của `firebase_remote_config` 7.0.1 và `url_launcher` 6.3.3, không phải biên dịch |
| `flutter test`: so phiên bản, chính sách, fail open, hết giờ chờ (đồng hồ giả), giới hạn nhắc, màn hình chặn và `PopScope`, cờ, kill switch, `RolloutGuard`, `PatchPreflight`, đọc `native-config/` | Có, 161 test |
| `FirebaseRemoteFlagSource` chạy với Firebase project thật, fetch thật, throttling thật | **Chưa chạy** |
| `UrlLauncherOpener` mở App Store hoặc Google Play thật trên thiết bị | **Chưa chạy** |
| Nút Back thật của Android và cử chỉ vuốt về của iOS trên màn hình chặn | **Chưa chạy**: test dùng `handlePopRoute` của binding và kênh `SystemNavigator.pop` giả |
| Endpoint `GET /app-config/min-version` thật | **Chưa có**: `PolicySource` chỉ có bản giả |
| `shorebird init`, `shorebird release`, `shorebird patch`, rollback patch | **Chưa chạy**: `shorebird.yaml.example` chỉ là mẫu với UUID giả |
| Halt rollout trên Play Console, pause phased release trên App Store Connect | **Chưa chạy**: cần tài khoản; các bước nêu theo tài liệu chính thức |
| Số crash-free thật từ Crashlytics | **Chưa có**: số trong test và bài viết là minh hoạ |

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/app_version.dart` | `AppVersion`: đọc và so sánh `x.y.z+build` |
| `lib/min_version_policy.dart` | `UpdateDecision` (sealed), `PlatformRule`, `VersionPolicyConfig.fromJson`, `MinVersionPolicy.evaluate` |
| `lib/store_link.dart`, `lib/store_platform.dart` | Chỉ nhận link `https` của đúng cửa hàng, nếu không dùng link dựng sẵn |
| `lib/update_gate.dart` | `PolicySource` và `UpdateGate` (fail open, thời gian chờ) |
| `lib/reminder_throttle.dart` | `ReminderStore` và `ReminderThrottle` |
| `lib/force_update_screen.dart`, `lib/update_guard.dart`, `lib/url_opener.dart` | Giao diện và ranh giới mở link |
| `lib/url_launcher_opener.dart` | Lớp bọc mỏng `url_launcher` |
| `lib/feature_flags.dart` | `FlagDef`, `Flags`, `RemoteFlagSource`, `FlagCache`, `FeatureFlags` |
| `lib/firebase_flag_source.dart` | Lớp bọc mỏng `firebase_remote_config` |
| `lib/rollout_guard.dart` | `RolloutStats`, `RolloutGuard.decide`, `RolloutGuard.notice` |
| `lib/patch_preflight.dart` | `PatchPreflight.classify` |
| `native-config/` | `min-version.example.json` (phản hồi mẫu của server), `remote-config-template.example.json` (mẫu template Remote Config), `shorebird.yaml.example` |
| `test/` | `support.dart` (bản giả) và 10 file test; `native_config_test.dart` kiểm các file mẫu khớp với code |

## Chạy và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/59-post-release-ops
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (161 test). Test đọc `native-config/` bằng đường dẫn tương đối nên phải chạy từ thư mục lab. Không test nào dùng mạng, Firebase, thiết bị hay đồng hồ thật.

## Dùng với app thật (tự làm, chưa kiểm)

1. Sinh runner một lần: `flutter create --platforms=ios,android --project-name toi_noi_59 --org vn.toinoi .` (thư mục `android/`, `ios/` không commit).
2. Khởi tạo Firebase theo bài 37 (`Firebase.initializeApp`), tạo `FirebaseRemoteFlagSource(FirebaseRemoteConfig.instance)`, gọi `configure()` một lần. Đặt tham số trong console theo `native-config/remote-config-template.example.json`.
3. Viết `PolicySource` thật gọi endpoint của BFF; phản hồi theo `native-config/min-version.example.json`. Viết `ReminderStore` và `FlagCache` bằng `shared_preferences` (bài 22).
4. Đặt `UpdateGuard` ở `MaterialApp.builder` (bao cả Navigator; đặt ở `home:` thì route push lúc chờ chính sách nằm trên màn hình chặn, test `update_guard_test.dart` có ca đối chứng), và gọi `FeatureFlags.load()` trước `runApp`, `refresh()` sau khi app đã hiện.
5. Chỉ khi quyết định dùng Shorebird: `shorebird init` rồi so file sinh ra với `shorebird.yaml.example`.

## Phiên bản đã kiểm

Flutter 3.47.4 stable / Dart 3.13.3. `firebase_remote_config` 7.0.1 (ràng buộc `^7.0.1`), `url_launcher` 6.3.3, `yaml` 3.1.4, `fake_async` 1.3.3, `flutter_lints` 6.0.0 (`pubspec.lock` không commit). Không lệch bảng §4 của khung (bảng chỉ ghi `firebase_messaging`, không ghi `firebase_remote_config`).

## Chưa làm

- Chưa lưu lại chính sách đọc được lần cuối để dùng khi server lỗi: hiện server lỗi luôn cho vào app, kể cả khi lần trước đã biết bản này bị force.
- Chưa có `onConfigUpdated` (cập nhật thời gian thực của Remote Config) cho kill switch; hiện kill switch chỉ nhận qua `refresh()`.
- `FlagCache` và `ReminderStore` chỉ có bản trong bộ nhớ (test); chưa có bản `shared_preferences`.
- Chưa có kiểm tra chữ ký hay xác thực cho phản hồi chính sách ngoài HTTPS; giả định BFF tin cậy.
- Màn hình chỉ có tiếng Việt, chưa qua `intl` (bài 14).
- `RolloutGuard` chưa đọc số liệu tự động từ Crashlytics hay BigQuery; ngưỡng là số minh hoạ. Chưa có bộ lọc theo kênh Play (track) hay theo model máy.
- `PatchPreflight` chỉ phân loại theo đường dẫn; chưa gọi `shorebird patch`. Mẫu thiết lập `auto_update` chưa có.
