import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_18/quote_repository.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

void main() {
  group('FakeQuoteRepository', () {
    final repository = FakeQuoteRepository();

    test('cước do calculateFare tính: 5 km, 2,001 km và 100 km', () async {
      final fares = [
        for (final km in [5.0, 2.001, 100.0])
          (await repository.fetchQuote(
            serviceKind: ServiceKind.ride,
            distanceKm: km,
          )).fare,
      ];

      expect(fares, const [Money(25500), Money(12005), Money(453000)]);
    });

    test('ngoài miền 0..100 km ném ArgumentError', () {
      expect(
        repository.fetchQuote(serviceKind: ServiceKind.ride, distanceKm: 100.5),
        throwsArgumentError,
      );
    });

    test('unavailableAboveKm: quá ngưỡng ném QuoteUnavailable, đúng ngưỡng thì được', () async {
      final limited = FakeQuoteRepository(unavailableAboveKm: 30);

      final ok = await limited.fetchQuote(
        serviceKind: ServiceKind.food,
        distanceKm: 30,
      );
      expect(ok.serviceKind, ServiceKind.food);
      expect(
        limited.fetchQuote(serviceKind: ServiceKind.food, distanceKm: 30.5),
        throwsA(isA<QuoteUnavailable>()),
      );
    });

    test(
      'placeTrip cấp mã tăng dần và lưu chuyến ở trạng thái requested',
      () async {
        final repo = FakeQuoteRepository(
          now: () => DateTime.utc(2026, 11, 6, 8),
        );
        final quote = await repo.fetchQuote(
          serviceKind: ServiceKind.parcel,
          distanceKm: 5,
        );

        final first = await repo.placeTrip(quote);
        final second = await repo.placeTrip(quote);

        expect([first.id, second.id], ['T-0001', 'T-0002']);
        expect(first.status, TripStatus.requested);
        expect(first.serviceKind, ServiceKind.parcel);
        expect(first.requestedAt, DateTime.utc(2026, 11, 6, 8));
        expect(await repo.fetchTrip('T-0002'), second);
      },
    );

    test('fetchTrip với mã lạ ném TripNotFound', () {
      expect(repository.fetchTrip('T-9999'), throwsA(isA<TripNotFound>()));
    });

    test('latency: kết quả chỉ về sau đúng khoảng latency (đồng hồ giả)', () {
      fakeAsync((async) {
        final slow = FakeQuoteRepository(latency: const Duration(seconds: 2));
        var done = false;
        slow
            .fetchQuote(serviceKind: ServiceKind.ride, distanceKm: 5)
            .then((_) => done = true);

        async.elapse(const Duration(milliseconds: 1999));
        expect(done, isFalse);
        async.elapse(const Duration(milliseconds: 1));
        expect(done, isTrue);
      });
    });
  });
}
