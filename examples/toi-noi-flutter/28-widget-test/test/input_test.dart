import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_28/providers.dart';
import 'package:toi_noi_28/quote_screen.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

const _field = Key('distance-field');
const _apply = Key('apply-distance');
const _error = 'Nhập số từ 0 đến 100 km';

/// Màn hình với repository do test điều khiển; báo giá đầu tiên (5 km) đã về.
Future<ControlledQuoteRepository> _pumpReady(WidgetTester tester) async {
  final repository = ControlledQuoteRepository();
  await pumpQuoteScreen(
    tester,
    overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
  );
  repository.completeQuote();
  await tester.pump();
  return repository;
}

QuoteRequest _request(WidgetTester tester) =>
    ProviderScope.containerOf(tester.element(find.byType(QuoteScreen)))
        .read(quoteRequestProvider);

void main() {
  group('enterText và nút Xem báo giá', () {
    // Số cước kỳ vọng tính tay từ bảng giá: 12.000 + (mét vượt x 4.500 + 500)
    // ~/ 1.000. 2,001 km vượt 1 m: 12.005; 100 km vượt 98.000 m: 453.000.
    const cases = [
      (input: '0', fare: '12.000 VND'),
      (input: '7', fare: '34.500 VND'),
      (input: '2,001', fare: '12.005 VND'),
      (input: '2.001', fare: '12.005 VND'),
      (input: ' 100 ', fare: '453.000 VND'),
    ];

    for (final c in cases) {
      testWidgets('nhập "${c.input}" km: cước ${c.fare}', (tester) async {
        final repository = await _pumpReady(tester);

        await tester.enterText(find.byKey(_field), c.input);
        await tester.tap(find.byKey(_apply));
        await tester.pump();
        // Provider tính lại: đang tải. Trả lời yêu cầu mới nhất.
        expect(find.byKey(const Key('quote-loading')), findsOneWidget);
        repository.completeQuote();
        await tester.pump();

        expect(
          find.descendant(
            of: find.byKey(const Key('quote-card')),
            matching: find.text(c.fare),
          ),
          findsOneWidget,
        );
        expect(find.text(_error), findsNothing);
        expect(repository.fetchCount, 2);
      });
    }

    for (final input in [
      '',
      'abc',
      '-1',
      '100,5',
      '100.001',
      'NaN',
      'Infinity',
    ]) {
      testWidgets('nhập "$input": báo lỗi, không gọi repository', (
        tester,
      ) async {
        final repository = await _pumpReady(tester);

        await tester.enterText(find.byKey(_field), input);
        await tester.tap(find.byKey(_apply));
        await tester.pump();

        expect(find.text(_error), findsOneWidget);
        // Bản nháp giữ nguyên 5 km, báo giá cũ còn đó, không có yêu cầu mới.
        expect(_request(tester).distanceKm, 5);
        expect(repository.fetchCount, 1);
        expect(find.byKey(const Key('quote-fare')), findsOneWidget);
      });
    }

    testWidgets('nhập lại đúng 5 km thì không hỏi lại server', (tester) async {
      final repository = await _pumpReady(tester);

      await tester.enterText(find.byKey(_field), '5.0');
      await tester.tap(find.byKey(_apply));
      await tester.pump();

      // QuoteRequest(ride, 5) == QuoteRequest(ride, 5): state không đổi nên
      // provider không tính lại.
      expect(repository.fetchCount, 1);
      expect(find.byKey(const Key('quote-fare')), findsOneWidget);
    });

    testWidgets('sửa lại cho đúng thì thông báo lỗi biến mất', (tester) async {
      final repository = await _pumpReady(tester);

      await tester.enterText(find.byKey(_field), 'abc');
      await tester.tap(find.byKey(_apply));
      await tester.pump();
      expect(find.text(_error), findsOneWidget);

      await tester.enterText(find.byKey(_field), '12');
      await tester.tap(find.byKey(_apply));
      await tester.pump();
      repository.completeQuote();
      await tester.pump();

      expect(find.text(_error), findsNothing);
      expect(_request(tester).distanceKm, 12);
    });

    testWidgets('enterText thay toàn bộ nội dung, không nối thêm', (
      tester,
    ) async {
      await _pumpReady(tester);
      expect(distanceText(tester), '5');

      await tester.enterText(find.byKey(_field), '7');
      expect(distanceText(tester), '7');

      await tester.enterText(find.byKey(_field), '12');
      expect(distanceText(tester), '12');
    });

    testWidgets('phím Done trên bàn phím cũng gửi như bấm nút', (tester) async {
      final repository = await _pumpReady(tester);

      await tester.enterText(find.byKey(_field), '8');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pump();
      repository.completeQuote();
      await tester.pump();

      expect(_request(tester).distanceKm, 8);
      expect(repository.fetchCount, 2);
    });

    testWidgets('chạm chip dịch vụ đổi báo giá và dịch vụ được chọn', (
      tester,
    ) async {
      final repository = await _pumpReady(tester);

      await tester.tap(find.byKey(const Key('service-food')));
      await tester.pump();
      repository.completeQuote();
      await tester.pump();

      expect(repository.quoteRequests.last.serviceKind, ServiceKind.food);
      expect(
        tester
            .widget<ChoiceChip>(find.byKey(const Key('service-food')))
            .selected,
        isTrue,
      );
      expect(
        find.descendant(
          of: find.byKey(const Key('quote-card')),
          matching: find.text('Giao đồ ăn'),
        ),
        findsOneWidget,
      );
    });
  });
}
