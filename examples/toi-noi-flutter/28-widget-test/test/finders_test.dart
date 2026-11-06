import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_28/providers.dart';
import 'package:toi_noi_28/quote_repository.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'support.dart';

/// Màn hình đã có báo giá 5 km: cước 25.500 VND.
Future<void> _pumpWithQuote(WidgetTester tester) async {
  await pumpQuoteScreen(
    tester,
    overrides: [
      quoteRepositoryProvider.overrideWithValue(FakeQuoteRepository()),
    ],
  );
  await tester.pumpAndSettle();
}

void main() {
  group('finder và matcher', () {
    testWidgets('byKey: đúng một widget; byType: ba chip dịch vụ', (
      tester,
    ) async {
      await _pumpWithQuote(tester);

      expect(find.byKey(const Key('quote-fare')), findsOneWidget);
      expect(find.byType(ChoiceChip), findsNWidgets(ServiceKind.values.length));
      expect(find.byType(ChoiceChip), findsNWidgets(3));
      expect(find.byKey(const Key('quote-error')), findsNothing);
    });

    testWidgets('text: cùng một số tiền xuất hiện hai lần', (tester) async {
      await _pumpWithQuote(tester);

      // Thẻ cước và dòng "Tổng" của bảng chi tiết cùng hiện 25.500 VND.
      expect(find.text('25.500 VND'), findsNWidgets(2));
      expect(find.text('25.500 VND'), findsWidgets);
      expect(find.text('25.500 VND'), isNot(findsOneWidget));
    });

    testWidgets('descendant: thu hẹp tìm kiếm trong một khối', (tester) async {
      await _pumpWithQuote(tester);

      expect(
        find.descendant(
          of: find.byKey(const Key('quote-card')),
          matching: find.text('25.500 VND'),
        ),
        findsOneWidget,
      );
      expect(
        find.descendant(
          of: find.byKey(const Key('fare-breakdown')),
          matching: find.text('25.500 VND'),
        ),
        findsOneWidget,
      );
    });

    testWidgets('widgetWithText tìm nút theo nhãn, ancestor đi ngược lên', (
      tester,
    ) async {
      await _pumpWithQuote(tester);

      expect(find.widgetWithText(FilledButton, 'Xem báo giá'), findsOneWidget);
      expect(find.widgetWithText(FilledButton, 'Đặt chuyến'), findsOneWidget);
      // "Thử lại" không có trong state dữ liệu.
      expect(find.widgetWithText(OutlinedButton, 'Thử lại'), findsNothing);
      expect(
        find.ancestor(
          of: find.text('Đặt chuyến'),
          matching: find.byType(FilledButton),
        ),
        findsOneWidget,
      );
    });

    testWidgets('textContaining bắt một phần chuỗi', (tester) async {
      await _pumpWithQuote(tester);

      expect(find.textContaining('Giá minh hoạ'), findsOneWidget);
      // find.text so khớp toàn bộ chuỗi, nên cụm con không đủ.
      expect(find.text('Giá minh hoạ'), findsNothing);
    });

    testWidgets('lấy widget ra để kiểm thuộc tính, không chỉ sự có mặt', (
      tester,
    ) async {
      await _pumpWithQuote(tester);

      final fare = tester.widget<MoneyText>(
        find.byKey(const Key('quote-fare')),
      );
      expect(fare.money, calculateFare(5));
      expect(fare.money.vnd, 25500);

      final ride = tester.widget<ChoiceChip>(
        find.byKey(const Key('service-ride')),
      );
      final food = tester.widget<ChoiceChip>(
        find.byKey(const Key('service-food')),
      );
      expect(ride.selected, isTrue);
      expect(food.selected, isFalse);
    });

    testWidgets('finder thấy widget nằm ngoài màn hình, người dùng thì không', (
      tester,
    ) async {
      usePhoneScreen(tester);
      await _pumpWithQuote(tester);

      final button = find.byKey(const Key('place-trip'));
      expect(button, findsOneWidget);
      final screenHeight =
          tester.view.physicalSize.height / tester.view.devicePixelRatio;
      expect(tester.getTopLeft(button).dy, greaterThan(screenHeight));
    });

    testWidgets('tìm theo text gãy khi đổi nhãn, tìm theo key thì không', (
      tester,
    ) async {
      Widget screen(String label) => MaterialApp(
        home: Scaffold(
          body: FilledButton(
            key: const Key('submit'),
            onPressed: () {},
            child: Text(label),
          ),
        ),
      );

      await tester.pumpWidget(screen('Đặt chuyến'));
      expect(find.text('Đặt chuyến'), findsOneWidget);
      expect(find.byKey(const Key('submit')), findsOneWidget);

      // Đội sản phẩm đổi nhãn.
      await tester.pumpWidget(screen('Đặt xe ngay'));
      expect(find.text('Đặt chuyến'), findsNothing);
      expect(find.byKey(const Key('submit')), findsOneWidget);
    });
  });
}
