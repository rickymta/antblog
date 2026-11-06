import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_15/driver.dart';
import 'package:toi_noi_15/driver_card.dart';
import 'package:toi_noi_15/motion.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

final _driver = kDrivers.first;

Widget _card({bool reduceMotion = false}) => harness(
  Scaffold(
    body: Host<TripStatus>(
      initial: TripStatus.accepted,
      builder: (context, status) => Align(
        alignment: Alignment.topCenter,
        child: DriverCard(driver: _driver, status: status),
      ),
    ),
  ),
  reduceMotion: reduceMotion,
);

BoxDecoration _decoration(WidgetTester tester) {
  final box = tester.widget<DecoratedBox>(
    find
        .descendant(
          of: find.byKey(const Key('driver-card-box')),
          matching: find.byType(DecoratedBox),
        )
        .first,
  );
  return box.decoration as BoxDecoration;
}

double _etaOpacity(WidgetTester tester) => tester
    .widget<FadeTransition>(
      find.descendant(
        of: find.byType(AnimatedOpacity),
        matching: find.byType(FadeTransition),
      ),
    )
    .opacity
    .value;

void main() {
  final etaText = 'Tới điểm đón sau khoảng ${_driver.etaMinutes} phút';

  group('DriverCard', () {
    testWidgets('hiện tên, xe và biển số của tài xế', (tester) async {
      await tester.pumpWidget(_card());
      expect(find.text(_driver.name), findsOneWidget);
      expect(
        find.text('${_driver.vehicle} · ${_driver.plate}'),
        findsOneWidget,
      );
    });

    testWidgets('accepted sang arriving: màu nền đi qua giá trị trung gian rồi '
        'tới primaryContainer', (tester) async {
      await tester.pumpWidget(_card());
      final scheme = Theme.of(tester.element(find.byType(DriverCard)))
          .colorScheme;
      expect(_decoration(tester).color, scheme.surfaceContainerLow);

      Host.of<TripStatus>(tester).value = TripStatus.arriving;
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 150));

      final mid = _decoration(tester).color!;
      expect(mid, isNot(scheme.surfaceContainerLow));
      expect(mid, isNot(scheme.primaryContainer));
      // easeInOut đối xứng nên giữa chặng thời gian là giữa chặng giá trị.
      final expected = Color.lerp(
        scheme.surfaceContainerLow,
        scheme.primaryContainer,
        ToiNoiMotion.standardCurve.transform(0.5),
      )!;
      expect(mid, expected);

      await tester.pump(const Duration(milliseconds: 150));
      expect(_decoration(tester).color, scheme.primaryContainer);
    });

    testWidgets('viền dày dần từ 1 lên 2 khi sang arriving', (tester) async {
      await tester.pumpWidget(_card());
      double width() => ((_decoration(tester).border! as Border).top.width);
      expect(width(), 1);

      Host.of<TripStatus>(tester).value = TripStatus.arriving;
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 150));
      expect(width(), inExclusiveRange(1, 2));

      await tester.pump(const Duration(milliseconds: 150));
      expect(width(), 2);
    });

    testWidgets('dòng ETA mờ dần hiện lên trong 150 ms', (tester) async {
      await tester.pumpWidget(_card());
      expect(_etaOpacity(tester), 0);

      Host.of<TripStatus>(tester).value = TripStatus.arriving;
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 75));
      expect(_etaOpacity(tester), inExclusiveRange(0, 1));

      await tester.pump(const Duration(milliseconds: 75));
      expect(_etaOpacity(tester), 1);
    });

    testWidgets('dòng ETA có trong cây semantics khi hiện, vắng khi độ mờ 0', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      await tester.pumpWidget(_card());
      expect(find.bySemanticsLabel(RegExp('Tới điểm đón')), findsNothing);

      Host.of<TripStatus>(tester).value = TripStatus.arriving;
      await tester.pumpAndSettle();
      expect(find.bySemanticsLabel(RegExp(etaText)), findsOneWidget);
      handle.dispose();
    });

    testWidgets('bật giảm chuyển động: màu, viền và ETA đổi ngay trong khung '
        'đầu', (tester) async {
      await tester.pumpWidget(_card(reduceMotion: true));
      final scheme = Theme.of(tester.element(find.byType(DriverCard)))
          .colorScheme;

      Host.of<TripStatus>(tester).value = TripStatus.arriving;
      await tester.pump();

      expect(_decoration(tester).color, scheme.primaryContainer);
      expect(((_decoration(tester).border! as Border).top.width), 2);
      expect(_etaOpacity(tester), 1);
    });
  });
}
