import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_18/providers.dart';
import 'package:toi_noi_18/quote_repository.dart';
import 'package:toi_noi_18/trip_screen.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'support.dart';

void main() {
  group('tripByIdProvider (family, auto-dispose)', () {
    test('cùng đối số là cùng provider, khác đối số là khác provider', () {
      expect(tripByIdProvider('T-1'), tripByIdProvider('T-1'));
      expect(tripByIdProvider('T-1'), isNot(tripByIdProvider('T-2')));
    });

    test('đọc chuyến vừa đặt qua repository giả', () async {
      final repository = FakeQuoteRepository();
      final container = ProviderContainer.test(
        overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
      );
      final placed = await repository.placeTrip(quoteFor(100));

      final trip = await container.read(tripByIdProvider(placed.id).future);

      expect(trip, placed);
      expect(trip.fare, const Money(453000));
    });

    test(
      'mỗi id có state riêng và chỉ gọi repository một lần khi còn người nghe',
      () async {
        final repository = StubQuoteRepository();
        final container = ProviderContainer.test(
          overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
        );
        container.listen(tripByIdProvider('T-1'), (_, _) {});
        container.listen(tripByIdProvider('T-2'), (_, _) {});

        await container.read(tripByIdProvider('T-1').future);
        await container.read(tripByIdProvider('T-2').future);
        await container.read(tripByIdProvider('T-1').future);

        expect(repository.tripRequests, ['T-1', 'T-2']);
      },
    );

    test('bỏ một id không ảnh hưởng id khác', () async {
      final container = ProviderContainer.test(
        overrides: [
          quoteRepositoryProvider.overrideWithValue(StubQuoteRepository()),
        ],
      );
      final one = container.listen(tripByIdProvider('T-1'), (_, _) {});
      container.listen(tripByIdProvider('T-2'), (_, _) {});
      await container.read(tripByIdProvider('T-1').future);
      await container.read(tripByIdProvider('T-2').future);

      one.close();
      await container.pump();

      expect(container.read(tripByIdProvider('T-1').exists), isFalse);
      expect(container.read(tripByIdProvider('T-2').exists), isTrue);
    });

    test(
      'không có chuyến: AsyncError(TripNotFound); .future ném lỗi gốc',
      () async {
        final container = ProviderContainer.test(
          overrides: [
            quoteRepositoryProvider.overrideWithValue(FakeQuoteRepository()),
          ],
        );
        container.listen(tripByIdProvider('T-404'), (_, _) {});

        await expectLater(
          container.read(tripByIdProvider('T-404').future),
          throwsA(isA<TripNotFound>().having((e) => e.id, 'id', 'T-404')),
        );
        expect(container.read(tripByIdProvider('T-404')).hasError, isTrue);
      },
    );

    test('family.overrideWith thay cách tạo cho mọi id', () async {
      final container = ProviderContainer.test(
        overrides: [
          tripByIdProvider.overrideWith(
            (ref, id) => Future.value(tripFor(id, km: 2.001)),
          ),
        ],
      );

      final trip = await container.read(tripByIdProvider('T-7').future);

      expect(trip.id, 'T-7');
      expect(trip.fare, const Money(12005));
    });
  });

  group('TripScreen với một thành viên của family bị ghi đè', () {
    testWidgets('overrideWithValue cho đúng id T-9, không cần repository', (
      tester,
    ) async {
      final searching = tripFor('T-9').apply(const SearchStarted());

      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            tripByIdProvider('T-9').overrideWithValue(AsyncData(searching)),
          ],
          child: MaterialApp(
            theme: ToiNoiTheme.light(),
            home: const TripScreen(tripId: 'T-9'),
          ),
        ),
      );

      expect(find.text('Đang tìm tài xế'), findsOneWidget);
      expect(find.text('25.500 VND'), findsOneWidget);
    });
  });
}
