import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'test_helpers.dart';

Text _textOf(WidgetTester tester) => tester.widget<Text>(find.byType(Text));

void main() {
  testWidgets('hiện cước 5 km là 25.500 VND', (tester) async {
    await pumpThemed(
      tester,
      MoneyText(calculateFare(5)),
      theme: ToiNoiTheme.light(),
    );
    expect(find.text('25.500 VND'), findsOneWidget);
  });

  testWidgets('Money.zero hiện 0 VND, số lớn có dấu chấm nghìn', (
    tester,
  ) async {
    await pumpThemed(
      tester,
      const MoneyText(Money.zero),
      theme: ToiNoiTheme.light(),
    );
    expect(find.text('0 VND'), findsOneWidget);

    await pumpThemed(
      tester,
      MoneyText(calculateFare(100)),
      theme: ToiNoiTheme.light(),
    );
    expect(find.text('453.000 VND'), findsOneWidget);
  });

  testWidgets('dùng chữ số đều độ rộng và titleMedium của theme', (
    tester,
  ) async {
    final theme = ToiNoiTheme.light();
    await pumpThemed(tester, const MoneyText(Money(12005)), theme: theme);
    final style = _textOf(tester).style!;
    expect(style.fontFeatures, contains(const FontFeature.tabularFigures()));
    expect(style.fontWeight, FontWeight.w600);
    expect(style.color, theme.colorScheme.onSurface);
  });

  testWidgets('style truyền vào được gộp lên theme, không thay thế', (
    tester,
  ) async {
    await pumpThemed(
      tester,
      const MoneyText(
        Money(12005),
        style: TextStyle(color: Colors.red),
        textAlign: TextAlign.end,
      ),
      theme: ToiNoiTheme.light(),
    );
    final text = _textOf(tester);
    expect(text.style!.color, Colors.red);
    expect(text.style!.fontWeight, FontWeight.w600);
    expect(text.style!.fontFeatures, isNotEmpty);
    expect(text.textAlign, TextAlign.end);
  });

  testWidgets('đổi sang theme tối thì màu chữ theo theme', (tester) async {
    final dark = ToiNoiTheme.dark();
    await pumpThemed(tester, const MoneyText(Money(12005)), theme: dark);
    expect(_textOf(tester).style!.color, dark.colorScheme.onSurface);
  });

  testWidgets('formatter thay cho Money.toString()', (tester) async {
    await pumpThemed(
      tester,
      MoneyText(const Money(25500), formatter: (money) => '${money.vnd} đ'),
      theme: ToiNoiTheme.light(),
    );
    expect(find.text('25500 đ'), findsOneWidget);
    expect(find.text('25.500 VND'), findsNothing);
  });

  for (final entry in themes.entries) {
    testWidgets(
      'trong FilledButton chữ lấy onPrimary, đọc được (${entry.key})',
      (tester) async {
        final theme = entry.value;
        await pumpThemed(
          tester,
          FilledButton(onPressed: () {}, child: const MoneyText(Money(25500))),
          theme: theme,
        );
        final color = _textOf(tester).style!.color!;
        expect(color, theme.colorScheme.onPrimary);
        expect(
          contrastRatio(color, theme.colorScheme.primary),
          greaterThanOrEqualTo(4.5),
        );
      },
    );
  }

  test(
    'lý do phải kế thừa màu: onSurface trên primary gần như không đọc được',
    () {
      // Đây là cái sẽ xảy ra nếu MoneyText ghim màu onSurface của titleMedium
      // khi nằm trong FilledButton.
      final light = ToiNoiTheme.light().colorScheme;
      final dark = ToiNoiTheme.dark().colorScheme;
      expect(
        contrastRatio(light.onSurface, light.primary),
        closeTo(2.64, 0.01),
      );
      expect(contrastRatio(dark.onSurface, dark.primary), closeTo(1.32, 0.01));
    },
  );
}
