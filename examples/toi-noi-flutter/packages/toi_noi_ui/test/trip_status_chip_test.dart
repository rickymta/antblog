import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'test_helpers.dart';

Color _backgroundOf(WidgetTester tester) {
  final box = tester.widget<DecoratedBox>(
    find.descendant(
      of: find.byType(TripStatusChip),
      matching: find.byType(DecoratedBox),
    ),
  );
  return (box.decoration as ShapeDecoration).color!;
}

Color _foregroundOf(WidgetTester tester) {
  final text = tester.widget<Text>(
    find.descendant(
      of: find.byType(TripStatusChip),
      matching: find.byType(Text),
    ),
  );
  return text.style!.color!;
}

void main() {
  testWidgets('label thay cho nhãn mặc định, null thì giữ nhãn tiếng Việt', (
    tester,
  ) async {
    await pumpThemed(
      tester,
      const TripStatusChip(status: TripStatus.searching, label: 'Searching'),
      theme: ToiNoiTheme.light(),
    );
    expect(find.text('Searching'), findsOneWidget);
    expect(find.text(TripStatus.searching.label), findsNothing);
  });

  test('toi_noi_core có đúng 8 trạng thái, chip phải bao phủ đủ', () {
    expect(TripStatus.values, hasLength(8));
  });

  group('cả 8 trạng thái, cả hai theme', () {
    for (final entry in themes.entries) {
      for (final status in TripStatus.values) {
        testWidgets('${entry.key}: ${status.name}', (tester) async {
          await pumpThemed(
            tester,
            TripStatusChip(status: status),
            theme: entry.value,
          );
          expect(find.text(status.label), findsOneWidget);
          expect(
            find.descendant(
              of: find.byType(TripStatusChip),
              matching: find.byType(Icon),
            ),
            findsOneWidget,
          );
          expect(
            contrastRatio(_foregroundOf(tester), _backgroundOf(tester)),
            greaterThanOrEqualTo(4.5),
          );
        });
      }
    }
  });

  group('ánh xạ màu', () {
    testWidgets(
      'completed dùng successContainer, cancelled dùng dangerContainer, '
      'searching dùng warningContainer',
      (tester) async {
        const colors = ToiNoiColors.light;
        final expected = {
          TripStatus.completed: (
            colors.successContainer,
            colors.onSuccessContainer,
          ),
          TripStatus.cancelled: (
            colors.dangerContainer,
            colors.onDangerContainer,
          ),
          TripStatus.searching: (
            colors.warningContainer,
            colors.onWarningContainer,
          ),
        };
        for (final entry in expected.entries) {
          await pumpThemed(
            tester,
            TripStatusChip(status: entry.key),
            theme: ToiNoiTheme.light(),
          );
          expect(_backgroundOf(tester), entry.value.$1, reason: entry.key.name);
          expect(_foregroundOf(tester), entry.value.$2, reason: entry.key.name);
        }
      },
    );

    testWidgets('bốn trạng thái đang diễn ra dùng primaryContainer', (
      tester,
    ) async {
      final theme = ToiNoiTheme.light();
      for (final status in [
        TripStatus.accepted,
        TripStatus.arriving,
        TripStatus.pickedUp,
        TripStatus.inTransit,
      ]) {
        await pumpThemed(tester, TripStatusChip(status: status), theme: theme);
        expect(_backgroundOf(tester), theme.colorScheme.primaryContainer);
      }
    });

    testWidgets('cùng trạng thái, sáng và tối cho nền khác nhau', (
      tester,
    ) async {
      await pumpThemed(
        tester,
        const TripStatusChip(status: TripStatus.completed),
        theme: ToiNoiTheme.light(),
      );
      final lightBackground = _backgroundOf(tester);
      await pumpThemed(
        tester,
        const TripStatusChip(status: TripStatus.completed),
        theme: ToiNoiTheme.dark(),
      );
      expect(_backgroundOf(tester), isNot(lightBackground));
    });

    testWidgets('mỗi trạng thái có icon riêng khi cùng nhóm màu (không chỉ '
        'dựa vào màu)', (tester) async {
      final icons = <IconData>{};
      for (final status in TripStatus.values) {
        await pumpThemed(
          tester,
          TripStatusChip(status: status),
          theme: ToiNoiTheme.light(),
        );
        icons.add(tester.widget<Icon>(find.byType(Icon)).icon!);
      }
      expect(icons, hasLength(TripStatus.values.length));
    });
  });

  group('textContrastGuideline', () {
    Widget allChips() => Wrap(
      spacing: 8,
      runSpacing: 8,
      children: [
        for (final status in TripStatus.values) TripStatusChip(status: status),
      ],
    );

    for (final entry in themes.entries) {
      testWidgets('theme ${entry.key}: cả 8 chip đạt guideline', (
        tester,
      ) async {
        final handle = tester.ensureSemantics();
        await pumpThemed(tester, allChips(), theme: entry.value);
        await expectLater(tester, meetsGuideline(textContrastGuideline));
        handle.dispose();
      });
    }
  });

  testWidgets('chữ phóng 2 lần trong chỗ hẹp xuống dòng, không tràn', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: ToiNoiTheme.light(),
        home: const MediaQuery(
          data: MediaQueryData(textScaler: TextScaler.linear(2)),
          child: Scaffold(
            body: Center(
              child: SizedBox(
                width: 140,
                child: TripStatusChip(status: TripStatus.pickedUp),
              ),
            ),
          ),
        ),
      ),
    );
    expect(tester.takeException(), isNull);
    expect(
      tester.getSize(find.byType(TripStatusChip)).width,
      lessThanOrEqualTo(140),
    );
  });
}
