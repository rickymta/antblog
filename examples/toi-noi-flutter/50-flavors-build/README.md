# Lab 50: flavors khi build, ID, icon, tên app và Firebase theo môi trường

Lab của bài **FL-50** trong loạt **Flutter qua Tới Nơi**: [Flavors khi build: bundle ID, application ID, icon, tên app và Firebase theo môi trường](../../../api/seed/content/flutter-flavors-bundle-id-icon-firebase-theo-moi-truong-build.md).

Bài 25 lo lớp Dart (`--dart-define-from-file`). Lab này lo lớp native: Gradle `productFlavors`, Xcode scheme, configuration và xcconfig, icon và tên app khác nhau, file Firebase chọn theo flavor. Vì repo không commit runner, các file đang dạy nằm trong `native-config/` và được kiểm bằng script Node và test Dart đọc file.

## Mức kiểm chứng: đọc kỹ

Mức theo curriculum: **Kiểm chứng cấu hình (CH)**. Máy viết lab không có Android SDK, Xcode chưa dùng được cho `xcodebuild`, không có Firebase project và không có thiết bị.

| Phần | Đã chạy ở đây? |
|---|---|
| `flutter analyze`, `flutter test` (131 test), `dart format` | Có |
| `node tool/verify-flavors.mjs` trên `native-config/`, `config/`, `flavors.json` | Có |
| `verify-flavors.mjs` trên bản sao bị làm hỏng từng chỗ (20 ca) | Có, trong test |
| Script Run Script `copy-google-service-info.sh` với `PROJECT_DIR` giả | Có, trong test (phép so `BUNDLE_ID` cần `/usr/libexec/PlistBuddy`, có trên macOS) |
| `tool/check-runner.mjs` trên `project.pbxproj` dựng trong test (chỉ xét khối của target Runner, nhận ra bằng `INFOPLIST_FILE = Runner/Info.plist`) | Có. Cũng thử tay trên pbxproj do `flutter create` sinh: thiếu chín cấu hình, và nhân bản khối Debug của target thì báo hai khoá ghi đè ở configuration đó |
| Sinh runner bằng `flutter create`, chép `native-config/customer` vào, `flutter analyze` | Có (trong thư mục tạm), sạch |
| `plutil -lint` các plist, `sh -n` script | Có, đạt |
| `flutter build apk --flavor staging --dart-define-from-file=config/staging.json` | Đã thử: dừng ngay với `No Android SDK found`, chưa vào Gradle |
| Biên dịch `build.gradle.kts`, `settings.gradle.kts` bằng Gradle | **Chưa chạy**: cú pháp chỉ đối chiếu với template Flutter 3.47.4 (AGP 9.1.0) và tài liệu |
| Tạo scheme, thêm file reference, tạo configuration và Run Script phase trong Xcode, `xcodebuild`, `flutter build ios/ipa --flavor` | **Chưa chạy** (các bước Xcode viết theo tài liệu và hiểu biết về Xcode) |
| Cài ba bản cạnh nhau trên thiết bị, nhận Firebase thật | **Chưa chạy** |

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `flavors.json` | "Hợp đồng": sáu ID theo §3.2 của curriculum và tên hiển thị của hai app |
| `native-config/customer/android/` | `settings.gradle.kts` (thêm plugin google-services), `app/build.gradle.kts` (flavors), `AndroidManifest.xml` (nhãn và icon theo tài nguyên), `src/<flavor>/res/values/strings.xml`, icon `mipmap-*` (dev, staging ở `src/<flavor>`, prod ở `src/main`), `src/<flavor>/google-services.json.example` |
| `native-config/customer/ios/` | `Flutter/Flavors/<flavor>.xcconfig`, chín `Flutter/<Mode>-<flavor>.xcconfig`, `Debug.xcconfig`/`Release.xcconfig` gốc, `Runner/Info.plist`, ba bộ `Assets.xcassets/AppIcon*.appiconset`, `Runner/Firebase/<flavor>/GoogleService-Info.plist.example`, `scripts/copy-google-service-info.sh` |
| `native-config/driver/` | Cây đầy đủ của app tài xế, sinh từ cây app khách bằng `tool/derive-driver-config.mjs` (đổi ID gốc, tên hiển thị, id Firebase giả); icon hình vuông do `make-placeholder-icons.mjs` sinh |
| `config/dev.json`, `staging.json`, `prod.json` | Bản sao của lab 25, để `--dart-define-from-file` đi cùng `--flavor` |
| `lib/flavor_check.dart`, `lib/flavor_screen.dart`, `lib/main.dart` | `checkFlavors` so `appFlavor` với `FLAVOR`; màn hình chẩn đoán cho biết bản cài đang là flavor nào |
| `tool/verify-flavors.mjs` | Kiểm toàn bộ `native-config/` với hợp đồng; `--root <thư mục>` để kiểm một bản sao |
| `tool/check-runner.mjs` | Kiểm `project.pbxproj` sau khi tạo cấu hình trong Xcode |
| `tool/flavor-command.mjs` | Ghép `--flavor X` và `--dart-define-from-file=config/X.json` từ một tên; `--dry-run` chỉ in lệnh |
| `tool/derive-driver-config.mjs` | Sinh `native-config/driver` từ `native-config/customer` (ghi đè file text, bỏ qua PNG) |
| `tool/make-placeholder-icons.mjs` | Sinh icon giữ chỗ cho cả hai app (nền màu theo flavor; tròn cho app khách, vuông cho app tài xế); không phải icon thiết kế |
| `test/` | 131 test: hợp đồng, Android, iOS, Firebase và `.gitignore`, icon, script chép plist, `verify-flavors`, `check-runner`, `flavor_check` |

## Kiểm chứng tự động

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/50-flavors-build
flutter pub get
flutter analyze
flutter test
node tool/verify-flavors.mjs
```

Kỳ vọng: `No issues found!`, `All tests passed!` (131 test) và `OK: 3 flavor x 2 app đều khớp hợp đồng`. Không test nào dùng mạng hay thiết bị. Test cần `node` (và `git`, `sh`) trên PATH; thiếu thì nhóm tương ứng tự bỏ qua.

Lệnh build theo flavor (in ra, chưa chạy ở đây):

```bash
node tool/flavor-command.mjs run dev --dry-run
node tool/flavor-command.mjs apk staging --dry-run
node tool/flavor-command.mjs appbundle prod --dry-run
```

## Chạy trên máy (tự thử)

Cần Android SDK (Android) hoặc Xcode (iOS), và tài khoản Firebase của bạn nếu muốn thử Firebase.

1. Sinh runner một lần. Tên dự án `customer` làm `namespace` và `applicationId` gốc thành `vn.toinoi.customer`:

   ```bash
   flutter create --platforms=ios,android --project-name customer --org vn.toinoi .
   rm test/widget_test.dart
   ```

   `flutter create` giữ nguyên `pubspec.yaml`, `analysis_options.yaml` và `lib/`; file `test/widget_test.dart` nó sinh ra phải xoá.
2. Chép cấu hình native lên runner. Xoá trước các PNG cũ trong `ios/Runner/Assets.xcassets/AppIcon.appiconset` (`Contents.json` mới chỉ nhắc `icon-1024.png`, PNG cũ sẽ nằm thừa), rồi: `cp -R native-config/customer/android/. android/` và `cp -R native-config/customer/ios/. ios/`.
3. Android: đổi các `google-services.json.example` thành `google-services.json` với file thật tải từ Firebase console cho từng app (giá trị trong `.example` là giả), đặt ở `android/app/src/<flavor>/`.
4. iOS, làm trong Xcode theo [docs.flutter.dev/deployment/flavors-ios](https://docs.flutter.dev/deployment/flavors-ios) (các bước dưới chưa chạy thử trong Xcode):
   - Tạo ba scheme `dev`, `staging`, `prod` và tích Shared.
   - **File > Add Files to "Runner"**: thêm chín file `Flutter/<Mode>-<flavor>.xcconfig`, bỏ chọn mọi target. Template không có synchronized group (không có `PBXFileSystemSynchronizedRootGroup` trong `project.pbxproj`), nên chép file ra đĩa chưa tạo file reference để chọn ở bước gắn.
   - Nhân bản Debug, Profile, Release thành chín cấu hình `<Mode>-<flavor>` (phần flavor viết thường); với target Runner gắn mỗi cấu hình vào file xcconfig cùng tên.
   - Xoá `PRODUCT_BUNDLE_IDENTIFIER` và `ASSETCATALOG_COMPILER_APPICON_NAME` ở Build Settings của target Runner (nếu còn, chúng thắng xcconfig).
   - Gán scheme với cấu hình (`Debug-<flavor>` cho Run, Test, Analyze; `Profile-<flavor>`; `Release-<flavor>` cho Archive).
   - Target Runner, **Build Phases**, **+**, **New Run Script Phase**, kéo xuống dưới "Copy Bundle Resources", nội dung `sh "${SRCROOT}/scripts/copy-google-service-info.sh"`.
   - Đặt file thật `GoogleService-Info.plist` vào `ios/Runner/Firebase/<flavor>/`.
   - Dự án có Podfile thì thêm chín cấu hình vào khối `project 'Runner'` (template Flutter 3.47.4 không có Podfile).

   Sau đó chạy `node tool/check-runner.mjs ios/Runner.xcodeproj/project.pbxproj`.
5. Chạy: `node tool/flavor-command.mjs run dev`. Mở từng bản cài trên cùng máy để xem ba tên, ba icon.

### Áp dụng cho `toi-noi-app`

Workspace thật có `tool/create-runners.mjs` (đã dùng `--project-name customer` và `driver`, `--org vn.toinoi`). Chạy nó, rồi chép `native-config/customer/{android,ios}` vào `apps/customer/` và `native-config/driver/{android,ios}` vào `apps/driver/` (xoá PNG cũ trong `AppIcon.appiconset` trước). `.gitignore` của workspace đang chặn `/apps/*/android/*` và `/apps/*/ios/*`; chọn commit file cấu hình nào là việc của bài tích hợp.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_lints` 6.0.0, `xml` 7.1.0 (dev, đọc plist và `strings.xml` trong test). Template runner: AGP 9.1.0, Kotlin plugin 2.4.0, Gradle wrapper 9.3.1, `compileSdk` do `flutter.compileSdkVersion` (36), `minSdk` 24 (`flutter.minSdkVersion`). Plugin `com.google.gms.google-services` 4.5.0 là bản mới nhất trên Google Maven lúc kiểm (11/2026). Các điểm về AGP 9 (`resValues` tắt mặc định), vị trí `google-services.json`, thứ tự áp build settings của Xcode đối chiếu tài liệu chính thức tháng 11/2026.

## Chưa làm

- Mọi mục **Chưa chạy** trong bảng đầu README.
- `app_name` đặt bằng `strings.xml` mỗi flavor, không bằng `resValue`: AGP 9 tắt `resValues` mặc định (trang flavors của Flutter nói rõ). Bật lại bằng `android { buildFeatures { resValues = true } }` là khả thi nhưng ngược với khuyến nghị của tài liệu.
- Icon thích ứng (`mipmap-anydpi-v26`) và icon thiết kế thật.
- App tài xế chỉ có cây sinh từ app khách và kiểm bằng cùng script; chưa có Firebase app thật hay icon riêng được thiết kế.
- Khởi tạo Firebase trong Dart (`firebase_core`, `Firebase.initializeApp()`), FlutterFire CLI và `firebase_options.dart` theo flavor.
- Chọn file cấu hình trong CI (bài 54), ký ứng dụng (bài 51, 52), phân phối (bài 55 trở đi).
