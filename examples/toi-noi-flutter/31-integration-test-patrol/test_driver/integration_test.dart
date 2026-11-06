import 'dart:io';

import 'package:integration_test/integration_test_driver_extended.dart';

/// Chạy trên máy dev khi dùng `flutter drive`: nhận ảnh chụp từ thiết bị và
/// ghi vào `build/screenshots/`.
///
///   flutter drive --driver=test_driver/integration_test.dart \
///     --target=integration_test/booking_flow_test.dart -d THIẾT_BỊ
Future<void> main() => integrationDriver(
  onScreenshot: (name, bytes, [args]) async {
    final file = File('build/screenshots/$name.png');
    await file.parent.create(recursive: true);
    await file.writeAsBytes(bytes);
    return true;
  },
);
