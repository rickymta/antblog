import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_22/db/app_database.dart';
import 'package:toi_noi_22/trip_history_repository.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

void main() {
  late AppDatabase db;
  late DriftTripHistoryRepository repository;

  setUp(() {
    db = openMemoryDatabase();
    repository = DriftTripHistoryRepository(db.tripDao);
  });
  tearDown(() => db.close());

  group('đổi qua lại giữa Trip và dòng database', () {
    test('Trip lưu rồi đọc ra bằng đúng Trip ban đầu', () async {
      final trips = [
        sampleTrip('T5', km: 5),
        sampleTrip('T2001', km: 2.001, kind: ServiceKind.food),
        sampleTrip('T100', km: 100, kind: ServiceKind.parcel),
        sampleTrip(
          'Tdriver',
          km: 12.5,
          status: TripStatus.accepted,
          driverId: 'D-07',
        ),
      ];

      await repository.saveAll(trips);

      for (final trip in trips) {
        expect(await repository.findById(trip.id), trip, reason: trip.id);
      }
    });

    test('lưu số nguyên: mét và đồng, cước khớp công thức', () async {
      await repository.saveAll([
        sampleTrip('A', km: 5),
        sampleTrip('B', km: 2.001),
        sampleTrip('C', km: 100),
      ]);

      final rows = {for (final r in await db.select(db.trips).get()) r.id: r};
      expect(rows['A']?.distanceMeters, 5000);
      expect(rows['A']?.fareVnd, 25500);
      expect(rows['B']?.distanceMeters, 2001);
      expect(rows['B']?.fareVnd, 12005);
      expect(rows['C']?.distanceMeters, 100000);
      expect(rows['C']?.fareVnd, 453000);
    });

    test('thời điểm lưu UTC mili giây và đọc lại vẫn là UTC', () async {
      await repository.saveAll([sampleTrip('A', requestedAt: t0)]);

      final row = await db.tripDao.findById('A');
      expect(row?.requestedAtMs, t0.millisecondsSinceEpoch);

      final trip = await repository.findById('A');
      expect(trip?.requestedAt.isUtc, isTrue);
      expect(trip?.requestedAt, t0);
    });

    test('phần dưới mili giây của thời điểm bị cắt', () async {
      final precise = DateTime.utc(2026, 11, 3, 8, 30, 0, 123, 456);
      await repository.saveAll([sampleTrip('A', requestedAt: precise)]);

      final loaded = await repository.findById('A');
      expect(loaded?.requestedAt, DateTime.utc(2026, 11, 3, 8, 30, 0, 123));
      expect(loaded?.requestedAt, isNot(precise));
    });

    test(
      'dòng hỏng (accepted nhưng không có tài xế) ném ArgumentError',
      () async {
        await db.customStatement(
          "INSERT INTO trips (id, status, distance_meters, fare_vnd, "
          "requested_at_ms, service_kind) VALUES "
          "('BAD', 'accepted', 5000, 25500, 1, 'ride')",
        );

        await expectLater(repository.findById('BAD'), throwsArgumentError);
      },
    );
  });

  group('saveAll', () {
    test('lưu lại cùng chuyến với trạng thái mới thì ghi đè', () async {
      final searching = sampleTrip('A', status: TripStatus.searching);
      final accepted = searching.apply(const DriverAccepted('D-07'));

      await repository.saveAll([searching]);
      await repository.saveAll([accepted]);

      expect(await repository.findById('A'), accepted);
      expect(await db.select(db.trips).get(), hasLength(1));
    });

    test('danh sách rỗng không làm gì', () async {
      await repository.saveAll(const []);
      expect(await db.select(db.trips).get(), isEmpty);
    });
  });

  group('watchRecent', () {
    test('giao Trip của domain, mới nhất trước', () async {
      await repository.saveAll([
        sampleTrip('old', requestedAt: t0),
        sampleTrip('new', requestedAt: t0.add(const Duration(hours: 1))),
      ]);

      final trips = await repository.watchRecent().first;
      expect(trips.map((t) => t.id), ['new', 'old']);
      expect(trips.first, isA<Trip>());
    });

    test('một dòng hỏng làm stream báo lỗi, clear() khôi phục', () async {
      await repository.saveAll([sampleTrip('OK')]);
      await db.customStatement(
        "INSERT INTO trips (id, status, distance_meters, fare_vnd, "
        "requested_at_ms, service_kind) VALUES "
        "('BAD', 'completed', 5000, 25500, 1, 'ride')",
      );
      // 'completed' mà không có driverId: Trip từ chối.
      await expectLater(repository.watchRecent().first, throwsArgumentError);

      expect(await repository.clear(), 2);
      expect(await repository.watchRecent().first, isEmpty);
    });
  });

  test('clear trả số chuyến đã xoá', () async {
    await repository.saveAll([sampleTrip('A'), sampleTrip('B')]);
    expect(await repository.clear(), 2);
    expect(await repository.clear(), 0);
  });
}
