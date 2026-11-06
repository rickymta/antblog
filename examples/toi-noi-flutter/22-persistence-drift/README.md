# Lab 22: lưu trữ cục bộ cho Tới Nơi

Lab của bài **FL-22** trong loạt **Flutter qua Tới Nơi**: [Lưu trữ cục bộ: drift, shared_preferences và secure storage](../../../api/seed/content/flutter-local-persistence-drift-secure-storage-lich-su-offline.md).

Ba loại dữ liệu, ba kho: lịch sử chuyến trong SQLite bằng **drift** (bảng, DAO, stream query, migration schema 1 lên 2), tuỳ chọn sáng/tối bằng **`SharedPreferencesAsync`**, token đăng nhập sau interface **`TokenStore`** (bản `flutter_secure_storage` và bản giả). Dùng [`toi_noi_core`](../packages/toi_noi_core) (`Trip`, `Money`, `calculateFare`) và [`toi_noi_ui`](../packages/toi_noi_ui). Bảng giá là minh hoạ; mọi con số cước do `calculateFare` tính.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/db/trips_table.dart` | Bảng `Trips` (schema 2): `fareVnd` và `distanceMeters` là `int`, `status` và `serviceKind` lưu tên enum, `requestedAtMs` là UTC mili giây |
| `lib/db/app_database.dart` | `AppDatabase`, `schemaVersion` 2, `stepByStep` với `from1To2`, mở file qua `drift_flutter` |
| `lib/db/trip_dao.dart` | `TripDao`: `upsertAll`, `watchRecent` (stream query), `findById`, `totalFareVnd`, `deleteRequestedBefore`, `deleteAll` |
| `lib/db/app_database.g.dart`, `trip_dao.g.dart`, `app_database.steps.dart` | **Mã sinh, được commit** (xem dưới) |
| `lib/trip_history_repository.dart` | `TripHistoryRepository` (interface), bản drift, hai extension đổi `TripRow` và `Trip` |
| `lib/preferences/app_preferences.dart` | `AppPreferences` trên `SharedPreferencesAsync` |
| `lib/auth/token_store.dart` | `AuthTokens`, `TokenStore`, `SecureTokenStore`, `InMemoryTokenStore` |
| `lib/providers.dart` | Provider Riverpod 3 nối ba kho với giao diện |
| `lib/history_screen.dart`, `settings_screen.dart`, `app.dart`, `main.dart` | Màn hình lịch sử (stream query), cài đặt (sáng/tối, phiên đăng nhập), gốc app |
| `drift_schemas/app_database/drift_schema_v1.json`, `_v2.json` | Ảnh chụp schema từng phiên bản (do `drift_dev make-migrations` sinh), làm chuẩn cho test migration |
| `test/drift/app_database/` | Test migration do drift sinh (`migration_test.dart`, đã điền dữ liệu thật) và `generated/schema*.dart` |
| `test/db/trip_dao_test.dart` | 13 test: upsert, CHECK, tên enum, thứ tự và giới hạn, stream phát lại, `customStatement` không báo stream, `customInsert(updates:)` và `notifyUpdates` báo được, tổng cước, xoá theo mốc |
| `test/trip_history_repository_test.dart` | 10 test: đổi qua lại `Trip` và dòng, mét và đồng, UTC, cắt dưới mili giây, dòng hỏng |
| `test/migration_from_old_file_test.dart` | 3 test: database schema 1 dựng tay được nâng lên 2, ghi sau nâng cấp, app bản cũ mở database mới hơn |
| `test/app_preferences_test.dart` | 5 test: mặc định, ghi/đọc, khoá, giá trị lạ, chưa gắn nền tảng |
| `test/token_store_test.dart` | 18 test: hợp đồng chung cho hai `TokenStore`, JSON hỏng, kho lỗi, `MissingPluginException`, `toString` không lộ token |
| `test/app_test.dart` | 6 test widget: lịch sử rỗng và có dữ liệu, thêm chuyến cập nhật màn hình, dòng hỏng, sáng/tối còn sau khi mở lại, đăng nhập/đăng xuất |

## Mã sinh được commit

`*.g.dart`, `app_database.steps.dart` và `test/drift/app_database/generated/` **có trong git**, nên `flutter test` chạy xanh ngay sau `flutter pub get`, không cần `build_runner`. Chỉ khi sửa bảng hoặc DAO mới phải sinh lại:

```bash
dart run build_runner build
# sau khi tăng schemaVersion:
dart run drift_dev make-migrations
```

`.gitignore` của lab và của repository không loại `*.g.dart`. Nếu sửa bảng mà quên sinh lại, analyzer báo lỗi ngay ở `app_database.g.dart`.

## Kiểm chứng (không cần simulator)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/22-persistence-drift
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (57 test). Mức kiểm chứng: **Kiểm chứng tự động**. Test không gọi mạng, không gọi plugin thật (shared_preferences và flutter_secure_storage dùng bản nền tảng trong bộ nhớ do chính gói cung cấp) và không phụ thuộc đồng hồ thật.

### SQLite của test đến từ đâu

`drift_flutter` kéo theo `sqlite3` 3.x. Gói này dùng **build hook** của Dart: khi `.dart_tool` chưa có bản tải sẵn (mỗi job CI sạch đều vậy), hook tải thư viện SQLite biên dịch sẵn (ví dụ `libsqlite3.arm64.macos.dylib`, `libsqlite3.x64.linux.so`) từ GitHub Releases của gói, kiểm sha256 và đặt vào `.dart_tool/hooks_runner/shared/sqlite3/`. Vì vậy:

- Không cần cài `libsqlite3` hay `apt install` gì, kể cả trên CI ubuntu; nhưng **khi `.dart_tool` chưa có bản tải sẵn thì cần truy cập mạng tới github.com** (mỗi job CI sạch đều tải lại).
- Đã kiểm trên macOS (tải về `.dart_tool/hooks_runner/shared/sqlite3/build/download-*`). Linux chưa chạy ở đây: theo `asset_hashes.dart` của sqlite3 3.9.0 có sẵn bản `x64.linux` và `arm64.linux`.

## Chạy trên máy (tự thử)

Thư mục lab không chứa `android/` và `ios/`. Sinh runner một lần rồi chạy:

```bash
flutter create --platforms=ios,android --project-name toi_noi_22 --org vn.toinoi .
flutter run -d <id>
```

Bấm "Thêm chuyến mẫu" vài lần, tắt hẳn app rồi mở lại: lịch sử vẫn còn (file `toi_noi.sqlite` trong thư mục application support). Vào Cài đặt chọn "Tối", tắt và mở lại: giao diện vẫn tối. "Đăng nhập thử" lưu hai chuỗi giả vào Keychain (iOS) hoặc kho mã hoá bằng khoá Android Keystore (Android).

Trên Android, README của `flutter_secure_storage` (mục "Disabling Auto Backup") khuyên tắt Auto Backup bằng `android:allowBackup="false"` trong `AndroidManifest.xml` của runner; lab không commit `android/` nên bước này do người chạy tự làm.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `drift` 2.35.3, `drift_flutter` 0.3.1, `drift_dev` 2.35.2, `build_runner` 2.17.1, `sqlite3` 3.9.0 (kéo theo, dùng build hook), `shared_preferences` 2.5.6, `flutter_secure_storage` 11.2.0, `path_provider` 2.1.6, `flutter_riverpod` 3.5.0, `flutter_lints` 6.0.0, `toi_noi_core` 0.1.0 và `toi_noi_ui` 0.1.0 (path), Material 3. `shared_preferences_platform_interface` và `flutter_secure_storage_platform_interface` chỉ là dev_dependency để test thay nền tảng.

## Chưa làm

- Chưa chạy trên iOS Simulator, Android emulator hay thiết bị thật: Keychain, Android Keystore, isolate nền của `drift_flutter` và đường dẫn file thật chưa được kiểm trong lab này. Test dùng database trong bộ nhớ và bản nền tảng giả.
- Chưa có web (drift trên web cần WASM và worker riêng), chưa mã hoá file SQLite (SQLCipher).
- Chưa xử lý hạ phiên bản: app bản cũ mở database mới hơn bị `StateError` (từ `stepByStep`; test `migration_from_old_file_test.dart` ghi lại hành vi này).
- Chưa đồng bộ với server, chưa có bảng outbox (bài 46), chưa có index và khoá ngoại.
- `AuthTokens` chỉ có hai chuỗi; OTP, refresh và interceptor thuộc bài 24.
