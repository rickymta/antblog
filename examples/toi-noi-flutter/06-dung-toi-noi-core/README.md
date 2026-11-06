# Lab 06b: dùng toi_noi_core bằng path dependency

Project đi kèm bài 06 của loạt **Flutter qua Tới Nơi**: [Package, pubspec, lint và test: thư viện toi_noi_core kiểm chứng được](../../../api/seed/content/dart-package-pubspec-lint-test-toi-noi-core.md). Package Dart thuần `toi_noi_06`: một "ứng dụng khách" nhỏ của [`packages/toi_noi_core`](../packages/toi_noi_core) để thấy package được dùng từ bên ngoài ra sao.

## Điểm cần xem

`pubspec.yaml` khai báo package dùng chung bằng đường dẫn tương đối:

```yaml
dependencies:
  toi_noi_core:
    path: ../packages/toi_noi_core
```

| File | Vai trò |
|---|---|
| `lib/report.dart` | `quoteLine`, `describeEvent` (switch vét cạn trên `sealed class` của package khác), `runTrip` |
| `bin/main.dart` | in báo giá, chạy một chuyến hoàn thành (6 sự kiện) và một chuyến bị huỷ (sự kiện thứ 7) |
| `test/report_test.dart` | kiểm các hàm trên và việc lỗi của package đi xuyên qua |

Chỉ `import 'package:toi_noi_core/toi_noi_core.dart'`; không import `lib/src/...` (lint `implementation_imports` sẽ báo).

## Chạy

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/06-dung-toi-noi-core
dart pub get
dart run bin/main.dart
```

## Kiểm chứng

```bash
dart analyze
dart test
```

Mức: **Kiểm chứng tự động** (`dart analyze` 0 issue, `dart test` xanh). Phiên bản đã kiểm: Flutter 3.47.4 / Dart 3.13.3, `lints` 6.1, `test` 1.33.

## Chưa làm

- Chưa dùng pub workspace (`resolution: workspace`); lab dùng path dependency để thấy cơ chế gốc. Workspace nằm trong kế hoạch từ lúc code dồn vào `toi-noi-app`.
- Chưa publish package lên pub.dev hay git; `toi_noi_core` có `publish_to: none`.
