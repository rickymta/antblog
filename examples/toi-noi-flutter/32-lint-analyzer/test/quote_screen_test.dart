import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_riverpod/misc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_32/composition.dart';
import 'package:toi_noi_32/domain/quote_repository.dart';
import 'package:toi_noi_32/presentation/quote_providers.dart';
import 'package:toi_noi_32/presentation/quote_screen.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Repository giả thuần domain: không biết lớp data tồn tại.
class _FixedQuote implements QuoteRepository {
  _FixedQuote(this.fare);

  final Money fare;
  final requested = <double>[];

  @override
  Future<Money> quote(double distanceKm) async {
    requested.add(distanceKm);
    return fare;
  }
}

Future<void> _pump(WidgetTester tester, List<Override> overrides) =>
    tester.pumpWidget(
      ProviderScope(
        overrides: overrides,
        child: const MaterialApp(home: QuoteScreen()),
      ),
    );

void main() {
  group('QuoteScreen với thực thi thật của composition root', () {
    testWidgets('5 km báo 25.500 VND', (tester) async {
      await _pump(tester, productionOverrides);
      await tester.pumpAndSettle();

      expect(find.text('5.0 km'), findsOneWidget);
      expect(
        tester.widget<Text>(find.byKey(const Key('quote'))).data,
        '25.500 VND',
      );
    });

    testWidgets('thêm 1 km tính lại: 6 km báo 30.000 VND', (tester) async {
      await _pump(tester, productionOverrides);
      await tester.pumpAndSettle();
      await tester.tap(find.text('Thêm 1 km'));
      await tester.pumpAndSettle();

      expect(find.text('6.0 km'), findsOneWidget);
      expect(
        tester.widget<Text>(find.byKey(const Key('quote'))).data,
        '30.000 VND',
      );
    });
  });

  group('QuoteScreen chỉ cần cổng của domain', () {
    testWidgets('hiện cước do repository giả trả', (tester) async {
      final fake = _FixedQuote(const Money(99000));
      await _pump(tester, [quoteRepositoryProvider.overrideWithValue(fake)]);
      await tester.pumpAndSettle();

      expect(find.text('99.000 VND'), findsOneWidget);
      expect(fake.requested, [5.0]);
    });

    testWidgets('chưa ghi đè repository thì báo lỗi, không đoán', (
      tester,
    ) async {
      await _pump(tester, const []);
      await tester.pumpAndSettle();

      expect(find.text('Không báo giá được'), findsOneWidget);
    });
  });

  group('quoteProvider', () {
    ProviderContainer containerWith(List<Override> overrides) {
      final container = ProviderContainer(overrides: overrides);
      addTearDown(container.dispose);
      // autoDispose: giữ provider sống suốt test.
      container.listen(quoteProvider, (_, _) {});
      return container;
    }

    test('biên 100 km báo 453.000 VND', () async {
      final container = containerWith(productionOverrides);
      container.read(distanceProvider.notifier).setKm(100);

      expect(await container.read(quoteProvider.future), const Money(453000));
    });

    test('ngoài miền 0 đến 100 km là lỗi, không phải cước sai', () async {
      final container = containerWith(productionOverrides);
      container.read(distanceProvider.notifier).setKm(100.5);

      await expectLater(
        container.read(quoteProvider.future),
        throwsArgumentError,
      );
    });
  });
}
