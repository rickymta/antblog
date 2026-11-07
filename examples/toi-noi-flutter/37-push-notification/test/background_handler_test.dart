import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

/// Handler nền chạy trong isolate riêng và được gọi qua tên, nên các điều kiện
/// của tài liệu Firebase là điều kiện về MÃ NGUỒN. Ta kiểm bằng cách đọc file.
void main() {
  final source = File('lib/background_handler.dart').readAsStringSync();
  final main = File('lib/main.dart').readAsStringSync();

  test('hàm nằm ở top-level, không thụt lề, không ẩn danh', () {
    expect(
      RegExp(
        r'^Future<void> firebaseMessagingBackgroundHandler\(RemoteMessage message\) async \{',
        multiLine: true,
      ).hasMatch(source),
      isTrue,
    );
  });

  test("có @pragma('vm:entry-point') ngay trên hàm", () {
    expect(
      RegExp(
        r"^@pragma\('vm:entry-point'\)\nFuture<void> firebaseMessagingBackgroundHandler",
        multiLine: true,
      ).hasMatch(source),
      isTrue,
    );
  });

  test('gọi Firebase.initializeApp trước khi làm việc khác', () {
    final body = source.substring(source.indexOf('async {'));
    expect(
      body.trimLeft(),
      startsWith('async {\n  await Firebase.initializeApp();'),
    );
  });

  test('chỉ ghi log tên khoá, không ghi nội dung tin', () {
    expect(source, contains('PushRouter.safeSummary(message.data)'));
    expect(source, isNot(contains('message.notification')));
  });

  test('main đăng ký đúng hàm đó, trước runApp', () {
    final register = main.indexOf(
      'FirebaseMessaging.onBackgroundMessage(firebaseMessagingBackgroundHandler)',
    );
    expect(register, isNonNegative);
    expect(register, lessThan(main.indexOf('runApp(')));
  });
}
