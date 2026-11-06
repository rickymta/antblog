import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_15/driver.dart';
import 'package:toi_noi_15/motion.dart';
import 'package:toi_noi_15/progress_ring.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

Widget _ring({
  required double initial,
  bool reduceMotion = false,
  VoidCallback? onEnd,
}) => harness(
  Scaffold(
    body: Host<double>(
      initial: initial,
      builder: (context, progress) =>
          ProgressRing(progress: progress, onEnd: onEnd),
    ),
  ),
  reduceMotion: reduceMotion,
);

double _ringValue(WidgetTester tester) => tester
    .widget<CircularProgressIndicator>(find.byType(CircularProgressIndicator))
    .value!;

void main() {
  const half = Duration(milliseconds: 300);

  group('ProgressRing', () {
    testWidgets('lần dựng đầu hiện đúng đích, không chạy từ 0', (tester) async {
      await tester.pumpWidget(_ring(initial: 0.5));
      expect(_ringValue(tester), 0.5);
      expect(find.text('50%'), findsOneWidget);
    });

    testWidgets('đổi đích: giá trị trung gian nằm giữa cũ và mới', (
      tester,
    ) async {
      await tester.pumpWidget(_ring(initial: 0.5));
      Host.of<double>(tester).value = 1;
      await tester.pump();
      await tester.pump(half);

      expect(_ringValue(tester), inExclusiveRange(0.5, 1));
    });

    testWidgets('sau đủ 600 ms: đúng đích, và onEnd chỉ gọi khi hoàn tất', (
      tester,
    ) async {
      var ended = 0;
      await tester.pumpWidget(_ring(initial: 0.5, onEnd: () => ended++));
      Host.of<double>(tester).value = 1;
      await tester.pump();
      await tester.pump(half);
      expect(ended, 0);

      await tester.pump(ToiNoiMotion.slow);
      expect(_ringValue(tester), 1);
      expect(find.text('100%'), findsOneWidget);
      expect(ended, 1);
    });

    testWidgets('đổi đích giữa chừng: chạy tiếp từ giá trị đang hiển thị', (
      tester,
    ) async {
      await tester.pumpWidget(_ring(initial: 0.5));
      final host = Host.of<double>(tester);
      host.value = 1;
      await tester.pump();
      await tester.pump(half);
      final midway = _ringValue(tester);
      // Hình 1 của bài: tại 300 ms vòng ở khoảng 93,75%.
      expect(midway, closeTo(0.9375, 0.001));

      host.value = 0;
      await tester.pump(); // khung đổi đích, chưa có thời gian trôi

      // Không nhảy về 0.5 (đích cũ ban đầu) hay 1 (đích vừa bỏ).
      expect(_ringValue(tester), closeTo(midway, 1e-9));

      // Đường cong chạy lại từ đầu với đủ 600 ms.
      await tester.pump(half);
      expect(_ringValue(tester), inExclusiveRange(0, midway));
      await tester.pump(half + const Duration(milliseconds: 1));
      expect(_ringValue(tester), 0);
    });

    testWidgets('huỷ lúc 50%: sau 300 ms (nửa thời lượng) chữ hiện 6%', (
      tester,
    ) async {
      await tester.pumpWidget(_ring(initial: 0.5));
      Host.of<double>(tester).value = 0;
      await tester.pump();
      await tester.pump(half);

      // easeOutCubic ở nửa thời gian đã đi được khoảng 87,5% quãng đường,
      // nên còn lại 0,5 x (1 - 0,875) = khoảng 0,0625.
      expect(_ringValue(tester), closeTo(0.0625, 0.001));
      expect(find.text('6%'), findsOneWidget);
    });

    testWidgets('bật giảm chuyển động: nhảy tới đích ngay trong khung đầu', (
      tester,
    ) async {
      await tester.pumpWidget(_ring(initial: 0.5, reduceMotion: true));
      Host.of<double>(tester).value = 1;
      await tester.pump();

      expect(_ringValue(tester), 1);
      expect(find.text('100%'), findsOneWidget);
    });
  });

  group('Vòng quay không có value', () {
    testWidgets('lặp vô hạn nên pumpAndSettle hết hạn; ProgressRing có value '
        'thì không', (tester) async {
      await tester.pumpWidget(
        harness(
          const Scaffold(body: Center(child: CircularProgressIndicator())),
        ),
      );
      Object? timeout;
      try {
        await tester.pumpAndSettle(
          const Duration(milliseconds: 100),
          EnginePhase.sendSemanticsUpdate,
          const Duration(seconds: 5),
        );
      } on FlutterError catch (error) {
        timeout = error;
      }
      expect(timeout.toString(), contains('pumpAndSettle timed out'));

      await tester.pumpWidget(_ring(initial: 0.5));
      Host.of<double>(tester).value = 1;
      await tester.pumpAndSettle();
      expect(_ringValue(tester), 1);
    });
  });

  group('TripProgress', () {
    test('sáu bước của đường đi bình thường tăng dần từ 0 tới 1', () {
      const path = [
        TripStatus.requested,
        TripStatus.searching,
        TripStatus.accepted,
        TripStatus.arriving,
        TripStatus.pickedUp,
        TripStatus.inTransit,
        TripStatus.completed,
      ];
      final values = [for (final s in path) s.progress];
      expect(values.first, 0);
      expect(values.last, 1);
      for (var i = 1; i < values.length; i++) {
        expect(values[i], greaterThan(values[i - 1]));
      }
    });

    test('huỷ về 0 và mọi giá trị nằm trong 0..1', () {
      expect(TripStatus.cancelled.progress, 0);
      for (final status in TripStatus.values) {
        expect(status.progress, inInclusiveRange(0, 1));
      }
    });
  });
}
