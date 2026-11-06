import 'dart:io';

import 'package:clock/clock.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_30/countdown.dart';
import 'package:toi_noi_30/countdown_text.dart';

String _shown(WidgetTester tester) =>
    tester.widget<Text>(find.byType(Text)).data!;

void main() {
  // testWidgets đã chạy trong FakeAsync: không cần gọi fakeAsync, và
  // `tester.pump(duration)` đóng vai `async.elapse`.
  group('testWidgets có sẵn FakeAsync', () {
    testWidgets('CountdownText đếm 0:03, 0:02, 0:01, 0:00 theo pump', (
      tester,
    ) async {
      final deadline = clock.now().add(const Duration(seconds: 3));
      await tester.pumpWidget(
        ProviderScope(child: CountdownText(deadline: deadline)),
      );
      expect(_shown(tester), '--:--'); // stream chưa kịp phát giá trị đầu

      await tester.pump();
      expect(_shown(tester), '0:03');

      await tester.pump(const Duration(seconds: 1));
      expect(_shown(tester), '0:02');

      await tester.pump(const Duration(seconds: 1));
      expect(_shown(tester), '0:01');

      await tester.pump(const Duration(seconds: 1));
      expect(_shown(tester), '0:00');
    });

    testWidgets('hạn nhận cuốc 15 giây', (tester) async {
      final deadline = offerDeadline(clock.now());
      await tester.pumpWidget(
        ProviderScope(child: CountdownText(deadline: deadline)),
      );
      await tester.pump();
      expect(_shown(tester), '0:15');

      await tester.pump(const Duration(seconds: 14));
      expect(_shown(tester), '0:01');
    });

    testWidgets('clock đi theo pump, DateTime.now() thì không', (tester) async {
      final fakeStart = clock.now();
      final realStart = DateTime.now();

      await tester.pump(const Duration(minutes: 30));

      expect(clock.now().difference(fakeStart), const Duration(minutes: 30));
      expect(
        DateTime.now().difference(realStart),
        lessThan(const Duration(minutes: 30)),
      );
    });

    testWidgets('tester.binding.clock là đồng hồ khác, bắt đầu từ năm 2015', (
      tester,
    ) async {
      expect(tester.binding.clock.now().year, 2015);
      expect(clock.now().year, isNot(2015));
    });

    testWidgets('hết hạn thì dừng ở 0:00 dù pump thêm', (tester) async {
      final deadline = clock.now().add(const Duration(seconds: 2));
      await tester.pumpWidget(
        ProviderScope(child: CountdownText(deadline: deadline)),
      );

      await tester.pump(const Duration(seconds: 2));
      await tester.pump(const Duration(hours: 1));

      expect(_shown(tester), '0:00');
    });

    testWidgets('gỡ widget giữa chừng: provider bị bỏ, test không kẹt Timer', (
      tester,
    ) async {
      final deadline = clock.now().add(const Duration(minutes: 1));
      await tester.pumpWidget(
        ProviderScope(child: CountdownText(deadline: deadline)),
      );
      await tester.pump(const Duration(seconds: 5));

      // Nếu Timer.periodic còn sống, testWidgets sẽ báo "A Timer is still
      // pending even after the widget tree was disposed" khi kết thúc.
      await tester.pumpWidget(const SizedBox());
    });
  });

  group('I/O thật trong testWidgets', () {
    testWidgets('chỉ chạy được trong tester.runAsync', (tester) async {
      final exists = await tester.runAsync(() => File('pubspec.yaml').exists());

      expect(exists, isTrue);
    });
  });
}
