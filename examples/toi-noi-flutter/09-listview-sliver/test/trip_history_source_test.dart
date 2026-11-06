import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_09/trip_history_source.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

void main() {
  group('FakeTripHistorySource', () {
    late FakeTripHistorySource source;

    setUp(() {
      source = FakeTripHistorySource(total: 47, latency: Duration.zero);
    });

    test('47 chuyến chia thành các trang 20, 20, 7', () async {
      final first = await source.fetchPage(offset: 0, limit: 20);
      final second = await source.fetchPage(offset: 20, limit: 20);
      final third = await source.fetchPage(offset: 40, limit: 20);

      expect(first.trips, hasLength(20));
      expect(first.hasMore, isTrue);
      expect(second.trips, hasLength(20));
      expect(second.hasMore, isTrue);
      expect(third.trips, hasLength(7));
      expect(third.hasMore, isFalse);
    });

    test(
      'các trang nối lại thành danh sách không trùng, mới nhất trước',
      () async {
        final all = <Trip>[];
        for (var offset = 0; offset < 47; offset += 20) {
          all.addAll((await source.fetchPage(offset: offset, limit: 20)).trips);
        }

        expect(all.map((t) => t.id).toSet(), hasLength(47));
        expect(all.first.id, 'T-0047');
        expect(all.last.id, 'T-0001');
        for (var i = 1; i < all.length; i++) {
          expect(all[i].requestedAt.isBefore(all[i - 1].requestedAt), isTrue);
        }
      },
    );

    test('mọi chuyến đã kết thúc; cứ 5 chuyến có 1 chuyến bị huỷ', () async {
      final page = await source.fetchPage(offset: 0, limit: 47);

      expect(page.trips.every((t) => t.status.isTerminal), isTrue);
      expect(page.trips.byStatus[TripStatus.cancelled], hasLength(9));
      expect(page.trips.byStatus[TripStatus.completed], hasLength(38));
      for (final trip in page.trips) {
        if (trip.status == TripStatus.cancelled) {
          expect(trip.driverId, isNull);
        } else {
          expect(trip.driverId, isNotNull);
        }
      }
    });

    test('cước đúng công thức: 5 km, 2,001 km và 100 km', () async {
      final trips = (await source.fetchPage(offset: 0, limit: 47)).trips;

      expect(trips[0].distanceKm, 5);
      expect(trips[0].fare, const Money(25500));
      expect(trips[1].distanceKm, 2.001);
      expect(trips[1].fare, const Money(12005));
      expect(trips[5].distanceKm, 100);
      expect(trips[5].fare, const Money(453000));
      for (final trip in trips) {
        expect(trip.fare, calculateFare(trip.distanceKm));
      }
    });

    test('offset vượt tổng trả trang rỗng và hết trang', () async {
      final page = await source.fetchPage(offset: 100, limit: 20);

      expect(page.trips, isEmpty);
      expect(page.hasMore, isFalse);
    });

    test('nguồn rỗng trả trang rỗng', () async {
      final empty = FakeTripHistorySource(total: 0, latency: Duration.zero);
      final page = await empty.fetchPage(offset: 0, limit: 20);

      expect(page.trips, isEmpty);
      expect(page.hasMore, isFalse);
    });

    test('offset âm và limit không dương ném ArgumentError', () {
      expect(
        () => source.fetchPage(offset: -1, limit: 20),
        throwsArgumentError,
      );
      expect(() => source.fetchPage(offset: 0, limit: 0), throwsArgumentError);
    });

    test('prepend đẩy các chuyến cũ xuống một vị trí', () async {
      final newest = Trip.request(
        id: 'T-0048',
        serviceKind: ServiceKind.ride,
        distanceKm: 5,
        requestedAt: DateTime.utc(2026, 11, 6),
      );
      source.prepend(newest);

      final page = await source.fetchPage(offset: 0, limit: 2);
      expect(page.trips.map((t) => t.id), ['T-0048', 'T-0047']);
      expect(source.total, 48);
    });

    test('phân trang theo offset: chuyến chen vào đầu làm trang kế lặp lại '
        'chuyến cuối trang trước', () async {
      final first = await source.fetchPage(offset: 0, limit: 20);
      source.prepend(
        Trip.request(
          id: 'T-0048',
          serviceKind: ServiceKind.ride,
          distanceKm: 5,
          requestedAt: DateTime.utc(2026, 11, 6),
        ),
      );
      final second = await source.fetchPage(offset: 20, limit: 20);

      expect(second.trips.first.id, first.trips.last.id);
    });

    test('fetchCount đếm số lần gọi', () async {
      await source.fetchPage(offset: 0, limit: 20);
      await source.fetchPage(offset: 20, limit: 20);

      expect(source.fetchCount, 2);
    });
  });
}
