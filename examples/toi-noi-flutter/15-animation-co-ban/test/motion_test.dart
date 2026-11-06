import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter/semantics.dart' show debugSemanticsDisableAnimations;
import 'package:toi_noi_15/motion.dart';

import 'support.dart';

class _Probe extends StatelessWidget {
  const _Probe({required this.onBuild});

  final void Function(bool reduced, Duration duration) onBuild;

  @override
  Widget build(BuildContext context) {
    onBuild(
      ToiNoiMotion.reduced(context),
      ToiNoiMotion.duration(context, ToiNoiMotion.standard),
    );
    return const SizedBox();
  }
}

void main() {
  group('ToiNoiMotion', () {
    testWidgets('mặc định trả nguyên thời lượng', (tester) async {
      Duration? seen;
      await tester.pumpWidget(
        harness(_Probe(onBuild: (_, duration) => seen = duration)),
      );
      expect(seen, ToiNoiMotion.standard);
    });

    testWidgets('bật giảm chuyển động (MediaQuery) thì thời lượng bằng 0', (
      tester,
    ) async {
      Duration? seen;
      await tester.pumpWidget(
        harness(
          _Probe(onBuild: (_, duration) => seen = duration),
          reduceMotion: true,
        ),
      );
      expect(seen, Duration.zero);
    });

    testWidgets(
      'đọc cờ disableAnimations thật của nền tảng, đổi khi đang chạy',
      (tester) async {
        // Không dùng harness: để MediaQuery tự lấy cờ từ platformDispatcher.
        final seen = <bool>[];
        await tester.pumpWidget(
          MaterialApp(home: _Probe(onBuild: (reduced, _) => seen.add(reduced))),
        );
        expect(seen.last, isFalse);

        tester.platformDispatcher.accessibilityFeaturesTestValue =
            const FakeAccessibilityFeatures(disableAnimations: true);
        addTearDown(
          tester.platformDispatcher.clearAccessibilityFeaturesTestValue,
        );
        await tester.pump();
        expect(seen.last, isTrue);
      },
    );

    testWidgets(
      'cờ reduceMotion riêng của iOS chưa được đọc (giới hạn đã biết)',
      (tester) async {
        final seen = <bool>[];
        tester.platformDispatcher.accessibilityFeaturesTestValue =
            const FakeAccessibilityFeatures(reduceMotion: true);
        addTearDown(
          tester.platformDispatcher.clearAccessibilityFeaturesTestValue,
        );
        await tester.pumpWidget(
          MaterialApp(home: _Probe(onBuild: (reduced, _) => seen.add(reduced))),
        );
        expect(seen.last, isFalse);
      },
    );
  });

  group('AnimationController của framework khi nền tảng bật cờ', () {
    Future<double> widthAfter(WidgetTester tester, Duration elapsed) async {
      final host = Host.of<double>(tester);
      host.value = 200;
      await tester.pump(); // dựng lại, animation bắt đầu ở khung này
      await tester.pump(elapsed);
      return tester.getSize(find.byKey(const Key('box'))).width;
    }

    Future<void> pumpBox(WidgetTester tester) {
      return tester.pumpWidget(
        Directionality(
          textDirection: TextDirection.ltr,
          child: Host<double>(
            initial: 100,
            builder: (context, width) => Align(
              alignment: Alignment.topLeft,
              child: AnimatedContainer(
                key: const Key('box'),
                duration: const Duration(seconds: 1),
                width: width,
                height: 10,
              ),
            ),
          ),
        ),
      );
    }

    testWidgets('không có cờ: 100 ms đầu của 1 giây mới đi được một phần', (
      tester,
    ) async {
      await pumpBox(tester);
      final width = await widthAfter(tester, const Duration(milliseconds: 100));
      expect(width, greaterThan(100));
      expect(width, lessThan(200));
    });

    testWidgets('có cờ: framework rút còn 5% thời lượng, 1 giây thành 50 ms', (
      tester,
    ) async {
      debugSemanticsDisableAnimations = true;
      addTearDown(() => debugSemanticsDisableAnimations = null);
      await pumpBox(tester);
      final width = await widthAfter(tester, const Duration(milliseconds: 60));
      expect(width, 200);
    });

    testWidgets('có cờ: 40 ms vẫn chưa tới đích, chưa phải nhảy ngay', (
      tester,
    ) async {
      debugSemanticsDisableAnimations = true;
      addTearDown(() => debugSemanticsDisableAnimations = null);
      await pumpBox(tester);
      final width = await widthAfter(tester, const Duration(milliseconds: 40));
      expect(width, lessThan(200));
    });
  });
}
