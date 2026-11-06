# Lab 32: lint, analyzer, dart format và analyzer plugin

Lab của bài **FL-32** trong loạt **Flutter qua Tới Nơi**: [Lint, analyzer, dart format và analyzer plugin: chất lượng code có kiểm chứng](../../../api/seed/content/flutter-lint-analyzer-format-analyzer-plugin-chat-luong-code.md).

Một app báo giá nhỏ (ba lớp `domain`, `data`, `presentation` như bài 21) làm nền để kiểm chứng bộ công cụ chất lượng: `analysis_options.yaml` với `flutter_lints` và ba cờ strict, plugin `riverpod_lint`, plugin tự viết `toi_noi_lints` (rule `no_data_import_in_presentation`: file dưới `lib/presentation/` không được import hay export file dưới `lib/data/`), `dart format`, `dart fix`, `// ignore` có lý do, một script pre-commit và ngân sách cảnh báo bằng 0. Bảng giá (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền 0 đến 100 km) là minh hoạ; mọi con số cước do `calculateFare` của `toi_noi_core` tính.

**Lệch so với khung (§4 của master curriculum)**: khung ghi `custom_lint`. Gói đó phát hành lần cuối 0.8.1 (09/2025), buộc `analyzer ^8.0.0`, trong khi bản analyzer hiện hành là 14.7.0; `riverpod_lint` đã bỏ `custom_lint` từ 3.1.0 (12/2025). Lab dùng **analyzer plugin chính thức** (`plugins:` trong `analysis_options.yaml`, có từ Dart 3.10) cho cả `riverpod_lint` lẫn lint tự viết.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `analysis_options.yaml` | `flutter_lints` + `strict-casts`, `strict-inference`, `strict-raw-types` + 9 rule thêm (gồm `document_ignores`, `unnecessary_ignore`) + khối `plugins:` (`riverpod_lint`, `toi_noi_lints`) |
| `lib/domain/`, `lib/data/`, `lib/presentation/`, `lib/composition.dart`, `lib/main.dart` | App báo giá tối thiểu: cổng `QuoteRepository` (domain), `FareTableRepository` (data), provider và màn hình (presentation), composition root nối ba lớp |
| `lint_plugin/` | Package `toi_noi_lints`: `lib/main.dart` (biến `plugin`), `lib/src/no_data_import_in_presentation.dart` (rule), `lib/src/layers.dart`; test bằng `analyzer_testing` |
| `fixtures/strict_before/`, `fixtures/strict_after/` | Cùng một việc (đọc cước từ JSON) viết sai và đúng dưới ba cờ strict |
| `fixtures/layers/`, `fixtures/layers_off/` | Ba lớp có một file vi phạm; `layers` bật rule, `layers_off` khai báo plugin nhưng không bật rule |
| `fixtures/fixable/`, `fixtures/ignores/` | Mẫu cho `dart fix` và cho `// ignore` có/không có lý do |
| `tool/pre-commit` | Script `sh` kiểm format và phân tích với ngân sách 0 cho file `.dart` đang staged (không tự cài) |
| `test/quote_screen_test.dart` | 6 test app: màn hình với thực thi thật và với repository giả, biên 100 km, ngoài miền là lỗi |
| `test/strict_analysis_test.dart` | 6 test: ba cờ bắt đúng 2 lỗi và 3 cảnh báo, mã sửa sạch, không cờ thì im lặng, từng cờ một |
| `test/dart_tools_test.dart` | 8 test: `dart format` theo language version, `dart fix`, `document_ignores` và `unnecessary_ignore` |
| `test/layer_plugin_test.dart` | 6 test: plugin chạy trong `dart analyze` thật, ignore có tiền tố, rule tắt mặc định, bẫy chỉ định thư mục `lib`, mã thoát 4 khi mất pub.dev |
| `test/pre_commit_test.dart` | 15 test script pre-commit trong repo git tạm, gồm `git commit` thật: file đổi tên, tên có dấu, lỗi cú pháp, worktree liên kết |

## Kiểm chứng (không cần simulator; cần pub.dev)

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/32-lint-analyzer
flutter pub get
dart format --output=none --set-exit-if-changed .
dart analyze --fatal-infos
flutter test

cd lint_plugin
dart pub get
dart analyze --fatal-infos
dart test
```

Kỳ vọng: `dart analyze` báo `No issues found!`; `flutter test` báo `All tests passed!` (41 test); trong `lint_plugin`, `dart test` báo 11 test xanh. Mức kiểm chứng: **Kiểm chứng tự động**.

**Dùng `dart analyze`, không phải `flutter analyze`, khi có plugin.** Với Flutter 3.47.4, `flutter analyze` vẫn báo lỗi của analyzer nhưng không in chẩn đoán của plugin (đã thử: file vi phạm `no_data_import_in_presentation` lọt qua `flutter analyze`, còn `dart analyze` báo). Chạy cả hai thì `dart analyze --fatal-infos` mới là cổng chặn.

Lưu ý khi chạy:

- Lần đầu analysis server biên dịch plugin (nó dựng một package `plugin_entrypoint` trong `~/.dartServer/.plugin_manager/` rồi biên dịch AOT), nên lần phân tích đầu hoặc đầu sau khi sửa plugin có thể mất vài chục giây tới vài phút trên máy tải nặng. Test dùng timeout 5 phút.
- Mỗi lần phân tích có plugin, analysis server chạy lại bước resolve phụ thuộc, nên cần với tới pub.dev: với `PUB_HOSTED_URL` trỏ vào một cổng chết, `dart analyze` thoát 4 kèm stack trace dù cache đã ấm (có test). Pre-commit và CI có plugin cũng cần mạng. `dart test` trong `lint_plugin` chỉ cần `dart pub get`.
- Bẫy: `dart analyze --fatal-infos lib` (chỉ định thư mục con) thoát 0 và không báo chẩn đoán của plugin, còn không đối số, `.` hoặc đường dẫn một file thì báo. Luôn chạy ở thư mục package.
- Test `strict_analysis_test`, `dart_tools_test` và `layer_plugin_test` gọi `dart` thật qua `Process`; `dart` phải có trên `PATH` (đi kèm Flutter).

## Chạy trên máy (tự thử)

Thư mục lab không chứa `android/` và `ios/`. Sinh runner một lần:

```bash
cd examples/toi-noi-flutter/32-lint-analyzer
flutter create --platforms=ios,android --project-name toi_noi_32 --org vn.toinoi .
flutter run -d <id>
```

Dùng thử pre-commit mà không cài vào repo: `git add` một file `.dart` trong lab rồi chạy `sh tool/pre-commit`. Muốn cài trong bản sao riêng của bạn, tạo `.git/hooks/pre-commit` (đừng dùng symlink: script tìm thư mục lab từ vị trí của chính nó) với nội dung:

```sh
#!/bin/sh
exec sh "$(git rev-parse --show-toplevel)/examples/toi-noi-flutter/32-lint-analyzer/tool/pre-commit"
```

rồi `chmod +x .git/hooks/pre-commit`. Lab này không cài hook nào, và không sửa hook của repository.

## Phiên bản đã kiểm

Flutter 3.47.4 stable, Dart 3.13.3, `flutter_riverpod` 3.5.0 (ràng buộc `^3.4.3`), `flutter_lints` 6.0.0, `riverpod_lint` 3.2.0 (plugin, ghim đúng `3.2.0` trong `analysis_options.yaml` vì plugin không có lockfile), `analysis_server_plugin` 0.3.26, `analyzer` 14.7.0, `analyzer_testing` 0.4.5, `test_reflective_loader` 0.2.3, `path` 1.9.1, `toi_noi_core` 0.1.0 (path), Material 3.

## Chưa làm

- Chưa chạy trên iOS Simulator, Android emulator hay thiết bị thật (không có SDK Android và Xcode đầy đủ trên máy kiểm).
- Plugin chỉ có một rule và không có quick fix; `dart fix --apply` không sửa được chẩn đoán của plugin. Quản lý cấu hình theo từng lớp (danh sách lớp trong file cấu hình thay vì hằng trong rule) chưa làm.
- Rule kiểm theo tên thư mục `presentation`/`data` dưới `lib/` của một package; không hiểu monorepo nhiều package cùng một lớp, và không kiểm `part` hay import trong `test/`.
- `riverpod_lint` chỉ được thử tay (một `Notifier` có field công khai làm `avoid_public_notifier_properties` báo info); không có test tự động cho nó.
- `analyzer_testing` ở 0.4.x và `analysis_server_plugin` ở 0.3.x còn là API trước 1.0: kiểm lại khi nâng Dart. Với plugin viết cho `analyzer` 14 và Dart 3.13, nâng SDK có thể buộc nâng plugin.
- Chưa có CI chạy các lệnh trên (bài 33) và chưa có `very_good_analysis` hay bộ lint nghiêm hơn.
