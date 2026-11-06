@Tags(['golden'])
library;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/golden_harness.dart';
import 'support/sample_cards.dart';

/// Ảnh chuẩn nằm ở `test/goldens/`, sinh bằng
/// `flutter test --update-goldens --tags golden` trên macOS.
void main() {
  group('TripCard', () {
    for (final brightness in Brightness.values) {
      testWidgets('bốn trạng thái, giao diện ${brightness.name}', (
        tester,
      ) async {
        await pumpGolden(
          tester,
          sampleCardColumn(),
          brightness: brightness,
          viewHeight: 1200,
        );

        await expectLater(
          find.byKey(goldenKey),
          matchesGoldenFile('goldens/trip_card_${brightness.name}.png'),
        );
      });
    }

    testWidgets('chữ gấp đôi, giao diện sáng', (tester) async {
      await pumpGolden(tester, oneCard(), textScale: 2);

      await expectLater(
        find.byKey(goldenKey),
        matchesGoldenFile('goldens/trip_card_text_x2_light.png'),
      );
    });
  }, skip: goldenSkip);

  group('TripStatusChip', () {
    for (final brightness in Brightness.values) {
      testWidgets('tám trạng thái, giao diện ${brightness.name}', (
        tester,
      ) async {
        await pumpGolden(tester, allStatusChips(), brightness: brightness);

        await expectLater(
          find.byKey(goldenKey),
          matchesGoldenFile('goldens/status_chips_${brightness.name}.png'),
        );
      });
    }
  }, skip: goldenSkip);
}
