# Lab 25: cấu hình môi trường dev, staging và prod

Lab của bài **FL-25** trong loạt **Flutter qua Tới Nơi**: [Cấu hình môi trường: flavors, --dart-define-from-file và DI theo môi trường](../../../api/seed/content/flutter-flavors-dart-define-cau-hinh-moi-truong-dev-staging-prod.md).

Một app rất nhỏ chỉ để trả lời câu hỏi "bản build này đang chạy với cấu hình nào?". Ba file `config/*.json` đi vào app bằng `--dart-define-from-file`, `AppConfig` kiểm tra chúng ngay khi khởi động, `appConfigProvider` được ghi đè trong `main()`, và `apiClientProvider` (dùng `ToiNoiApiClient` của [`toi_noi_api`](../packages/toi_noi_api)) lấy địa chỉ, timeout và việc có log HTTP hay không từ đó. Bảng giá của lệnh báo giá thử (5 km ra 25.500 VND) là minh hoạ do `calculateFare` của [`toi_noi_core`](../packages/toi_noi_core) tính.

Địa chỉ trong `config/` là địa chỉ hư cấu (`*.toinoi.example`, tên miền `.example` dành riêng cho ví dụ). Không có khoá, token hay chứng chỉ nào trong repo.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `config/dev.json`, `staging.json`, `prod.json` | Năm khoá cho mỗi môi trường: `FLAVOR`, `API_BASE_URL`, `LOG_HTTP`, `CONNECT_TIMEOUT_MS`, `RECEIVE_TIMEOUT_MS` |
| `config/local.json.example` | Mẫu cho điện thoại thật gọi máy dev qua địa chỉ LAN; sao thành `config/local.json` (đã nằm trong `.gitignore`) |
| `lib/config/flavor.dart` | `enum Flavor { dev, staging, prod }` |
| `lib/config/app_config.dart` | `AppConfig` (`fromEnvironment`, `fromMap`), `ConfigException`, `isLocalHost` |
| `lib/providers.dart` | `appConfigProvider` (chưa ghi đè thì ném lỗi), `httpAdapterProvider`, `logSinkProvider`, `apiClientProvider`, `SampleQuoteController` |
| `lib/bootstrap.dart`, `lib/main.dart` | `buildRoot()` đọc cấu hình rồi chạy app hoặc màn hình lỗi; `main()` |
| `lib/app.dart`, `lib/environment_screen.dart`, `lib/config_error_app.dart` | Vỏ app (ruy-băng môi trường), màn hình chẩn đoán, màn hình báo cấu hình sai |
| `test/app_config_test.dart` | Mọi quy tắc của `AppConfig.fromMap`: thiếu khoá, biên timeout, https, host cục bộ, user:password, query, `appFlavor` lệch |
| `test/config_files_test.dart` | Mỗi file JSON: đúng bộ khoá, đúng kiểu, qua `AppConfig`, không giống bí mật; prod không có localhost |
| `test/secret_scan_test.dart` | Bộ quét bí mật bắt được các dạng phổ biến và không bao giờ in giá trị |
| `test/environment_test.dart` | Hai nhóm loại trừ nhau: chạy không define, và chạy với `--dart-define-from-file` (kể cả lời gọi không `const`) |
| `test/providers_test.dart` | `appConfigProvider` chưa ghi đè, `apiClientProvider` và luồng báo giá theo từng flavor |
| `test/app_test.dart` | `buildRoot`: ba flavor, ruy-băng, báo giá thử, màn hình lỗi cấu hình |
| `test/support/` | Đọc JSON như công cụ Flutter, bộ quét bí mật, `Harness` (fake API trong tiến trình) |
| `tool/verify-flavors.mjs` | Chạy analyze và test theo từng cách cấp cấu hình (mặc định, ba flavor, lệch flavor, khoá dành riêng) |
| `tool/embed_demo.dart` | Chương trình 3 dòng để tự kiểm giá trị define nằm trong binary |

## Kiểm chứng (không cần simulator)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/25-flavors-config
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (86 test xanh, 5 bỏ qua). Một số test của `environment_test.dart` được đánh dấu bỏ qua vì chúng chỉ chạy khi có define; đó là chủ ý.

Chạy đủ các cách cấp cấu hình (mặc định, ba flavor kèm `--flavor`, chỉ file, file lệch `--flavor`, khoá `FLUTTER_APP_FLAVOR`):

```bash
node tool/verify-flavors.mjs
```

Hoặc tự chạy một flavor:

```bash
flutter test --flavor staging --dart-define-from-file=config/staging.json
```

Mức kiểm chứng: **Kiểm chứng tự động**. Test không gọi mạng: báo giá đi qua `InProcessAdapter` vào fake API trong tiến trình.

## Chạy trên máy (tự thử)

Thư mục lab không chứa `android/` và `ios/`. Sinh runner một lần rồi chạy từng flavor:

```bash
flutter create --platforms=ios,android --project-name toi_noi_25 --org vn.toinoi .
flutter run --dart-define-from-file=config/dev.json
flutter run --dart-define-from-file=config/staging.json
```

- Chưa có flavor native (Gradle `productFlavors`, Xcode scheme), nên **không** thêm `--flavor` ở đây; `appFlavor` hiện "không đặt". Flavor native là việc của bài 50.
- `dev.json` trỏ `http://localhost:8787`. iOS Simulator gọi được máy dev qua `localhost`. Android emulator phải đổi sang `10.0.2.2` bằng cách chồng thêm một define: `flutter run --dart-define-from-file=config/dev.json --dart-define=API_BASE_URL=http://10.0.2.2:8787` (`--dart-define` luôn thắng file, đứng trước hay sau cũng vậy). Điện thoại thật dùng `config/local.json` (sao từ `local.json.example`, sửa địa chỉ LAN).
- Server giả: `dart run bin/server.dart` trong [`../tools/fake_api`](../tools/fake_api), cổng 8787.
- Đã đọc cả hai chiều lệnh nhưng **chưa chạy** trên simulator, emulator hay thiết bị (máy kiểm không có Android SDK, CocoaPods).

Tự kiểm "giá trị define nằm trong binary":

```bash
dart compile exe -DDEMO_VALUE=gia-tri-thu tool/embed_demo.dart -o /tmp/embed_demo
env -i /tmp/embed_demo                  # in value=gia-tri-thu dù không có biến môi trường
grep -a -c gia-tri-thu /tmp/embed_demo  # 1: chuỗi nằm trong chính file binary
```

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_riverpod` 3.5.0, `dio` 5.11.1, `flutter_lints` 6.0.0, `toi_noi_core`, `toi_noi_ui`, `toi_noi_api` (path) và `toi_noi_fake_api` (dev, path), Material 3.

## Chưa làm

- Chưa có Gradle `productFlavors`, `applicationIdSuffix`, Xcode scheme, file `.xcconfig`, icon và tên app theo môi trường: bài 50. ID đích theo hồ sơ là `vn.toinoi.customer.dev`, `vn.toinoi.customer.staging` và `vn.toinoi.customer`.
- Chưa có Firebase theo môi trường (`google-services.json`, `GoogleService-Info.plist`), chưa có tác vụ CI chọn file cấu hình: bài 50 và bài 54.
- `LOG_HTTP` mới điều khiển `SafeLogInterceptor`; retry, timeout theo loại lỗi là bài 20, token là bài 24.
- Chưa chạy trên iOS Simulator, Android emulator hay thiết bị thật; chưa dựng `libapp.so` hay IPA để tự soi nội dung binary (chỉ kiểm bằng `dart compile exe` và `flutter build bundle`).
