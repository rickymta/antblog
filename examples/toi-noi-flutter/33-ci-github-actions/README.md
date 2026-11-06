# Lab 33: kiểm tra workflow CI của Tới Nơi bằng test Dart

Lab của bài **FL-33** trong loạt **Flutter qua Tới Nơi**: [CI với GitHub Actions: analyze, test, golden và cache pub](../../../api/seed/content/flutter-ci-github-actions-analyze-test-golden-cache.md).

Lab là một project Dart thuần (`toi_noi_33`). Nó không dựng CI mới; nó **đọc workflow thật** của repository (`.github/workflows/toi-noi-flutter.yml`) bằng package `yaml` và khẳng định các tính chất bài 33 nói tới: Flutter ghim đúng 3.47.4, không có `@main`/`@master`, có `permissions` chỉ đọc, mọi lệnh `node` và `working-directory` trỏ tới file có thật, không có chuỗi giống bí mật, bộ lọc `paths` bắt workflow và mọi project dưới `examples/toi-noi-flutter`. Mỗi kiểm tra có test "ca sai": workflow thật bị sửa đúng một chỗ phải làm kiểm tra đỏ.

## Mức kiểm chứng: đọc kỹ

| Phần | Đã làm ở đây? |
|---|---|
| Workflow là YAML hợp lệ và đọc được bằng package `yaml` | Có (test) |
| Các tính chất trên của workflow thật | Có (test) |
| Lệnh trong workflow (`flutter test --tags golden`, `dart analyze --fatal-infos`, `flutter test --coverage`) chạy xanh trong lab tương ứng trên macOS của máy tác giả | Có, chạy tay từng lệnh |
| Workflow chạy trên GitHub (runner thật, cache thật, artifact thật) | **Chưa**: nhánh chưa push nên chưa có run nào |
| Job `golden-macos` xanh trên runner macOS | **Chưa chạy**; ảnh chuẩn sinh trên macOS của máy tác giả, chưa so với runner |
| Cú pháp bằng `actionlint` | **Chưa**: máy kiểm không có `actionlint`, ta không cài mới. Nếu máy bạn có, chạy `actionlint .github/workflows/toi-noi-flutter.yml` từ gốc repository |
| Thời gian chạy và chi phí phút thật | **Chưa đo**; con số trong bài là minh hoạ |

Test chứng minh workflow **có** tính chất; chúng không chứng minh workflow **chạy được** trên GitHub. Lần push đầu tiên là phép thử thật.

## Cấu trúc

| Đường dẫn | Vai trò |
|---|---|
| `lib/workflow_audit.dart` | `WorkflowAudit`: đọc YAML và có một hàm kiểm cho mỗi tính chất (`checkFlutterPin`, `checkActionRefs`, `checkPermissions`, `checkNoSecrets`, `checkReferencedFiles`, `checkPathsTrigger`, `checkHygiene`, `checkPubCacheKey`); `globToRegExp` cài bảng filter pattern của GitHub |
| `test/real_workflow_test.dart` | 24 test trên workflow thật, kể cả hai job bài 33 thêm (golden macOS, cổng analyzer lab 32, coverage lab 28) |
| `test/audit_rules_test.dart` | 31 test ca sai: từng kiểm tra đỏ khi workflow thật bị sửa một chỗ; giá trị giả của bí mật được ghép từ mảnh nên repo không chứa chuỗi giống bí mật |
| `test/support.dart` | Tìm gốc repository, `mutated` (sửa một chỗ và báo lỗi nếu chỗ đó không có), liệt kê project dưới `examples/toi-noi-flutter` |

Một số test đọc thêm file của lab 28, 29 và 32 (chỉ đọc) để kiểm lý do của các bước workflow: `goldenBaselineOs` trong lab 29, `plugins:` trong lab 32, `.gitignore` có `pubspec.lock`.

## Chạy và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/33-ci-github-actions
dart pub get
dart format --output=none --set-exit-if-changed .
dart analyze --fatal-infos
dart test
```

Kỳ vọng: `No issues found!` và `All tests passed!` (55 test). Test cần chạy trong repository đầy đủ vì đọc `.github/workflows/toi-noi-flutter.yml` và lab khác; chép lab ra ngoài repository thì `findRepoRoot` ném `StateError`. Test không dùng mạng.

Các lệnh mà workflow chạy, thử tay trong từng lab:

```bash
cd examples/toi-noi-flutter/29-golden-semantics && flutter pub get && flutter test --tags golden
cd examples/toi-noi-flutter/32-lint-analyzer && flutter pub get && dart analyze --fatal-infos
cd examples/toi-noi-flutter/28-widget-test && flutter pub get && flutter test --coverage
```

`flutter test --coverage` ghi `coverage/lcov.info` trong lab 28; lab đó không liệt kê `coverage/` trong `.gitignore`, nên xoá thư mục này trước khi commit.

## Workflow đã đổi ở bài này

| Mục | Trước | Sau |
|---|---|---|
| `subosito/flutter-action` | `@v2` (tag trôi) | `@v2.23.0` |
| `actions/setup-node` | `@v6` | `@v7.1.0` |
| `permissions` | `contents: read` | giữ nguyên |
| `concurrency` | không có | huỷ run cũ cùng ref; `main` nhóm theo SHA nên không run nào bị huỷ hay bị thay khi chờ |
| Trigger `push` | mọi nhánh | `branches: [main]`; nhánh có PR chạy qua `pull_request`, tránh hai run (macOS tốn gấp đôi) |
| `timeout-minutes` | mặc định 360 | 20 (golden), 120 (verify: lab 01–26 khoảng 20 phút ở máy tác giả có cache, chừa 2–3 lần cho runner chậm và lab thêm sau), con số minh hoạ |
| Cache pub | khoá mặc định (băm `pubspec.lock`) | khoá băm `pubspec.yaml` |
| Job mới | không có | `golden-macos` (lab 29, `flutter test --tags golden`, artifact `test/failures`) |
| Bước mới trong `verify` | không có | `dart analyze --fatal-infos` lab 32, `flutter test --coverage` lab 28 và upload `lcov.info` |

Phiên bản action kiểm bằng API GitHub ngày 06/11/2026: `actions/checkout` v7.0.1, `subosito/flutter-action` v2.23.0, `actions/setup-node` v7.1.0, `actions/upload-artifact` v7.0.2. Tag có thể bị di chuyển; ghim SHA đầy đủ chặt hơn (kiểm lại khi đọc).

## Chạy trên máy (tự thử)

Lab là Dart thuần, không có runner nền tảng. Để thử workflow thật, push nhánh lên GitHub của bạn rồi xem tab Actions, hoặc dùng một công cụ chạy cục bộ nếu bạn đã có (lab không cài và không kiểm công cụ nào).

## Phiên bản đã kiểm

Flutter 3.47.4 stable / Dart 3.13.3, `yaml` 3.1.4, `test` 1.33 (ràng buộc `^1.33.0`), `lints` 6.1.0.

## Chưa làm

- Chưa có run nào trên GitHub; chưa kiểm cache pub có hit, golden có xanh trên runner macOS, artifact có tải được.
- Chưa chạy `actionlint`.
- Chưa ghim action bằng SHA đầy đủ; test chấp nhận cả tag ba số lẫn SHA.
- Chưa có ngưỡng coverage: coverage chỉ là báo cáo, chủ ý không chặn.
- Chưa chia `verify` thành nhiều job hay matrix (hướng làm khi vượt thời gian); chưa có `needs`, matrix hay reusable workflow; job `golden-macos` và `verify` chạy song song.
- Workflow chưa đặt `persist-credentials: false` cho `actions/checkout`.
- Lab 22 phụ thuộc `sqlite3` (build hook tải binary SQLite); chưa chạy trên Linux nên chưa biết mỗi job có tải lại không, vì cache pub chỉ giữ `~/.pub-cache`.
- Dependency của lab không có lockfile nên CI có thể resolve bản mới hơn máy dev (đã gặp: `flutter_riverpod` 3.5.0 đánh dấu `container.exists` deprecated làm `flutter analyze` thoát 1). Dự án thật commit `pubspec.lock` và chạy `dart pub get --enforce-lockfile`; lab là ví dụ nên chấp nhận và để CI báo trôi.
