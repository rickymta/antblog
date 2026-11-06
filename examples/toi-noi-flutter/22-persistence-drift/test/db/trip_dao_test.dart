import 'package:drift/drift.dart' hide isNull;
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_22/db/app_database.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import '../support.dart';

TripsCompanion row(
  String id, {
  TripStatus status = TripStatus.completed,
  int fareVnd = 25500,
  int requestedAtMs = 1000,
}) => TripsCompanion.insert(
  id: id,
  status: status,
  distanceMeters: 5000,
  fareVnd: fareVnd,
  requestedAtMs: requestedAtMs,
);

void main() {
  late AppDatabase db;

  setUp(() => db = openMemoryDatabase());
  tearDown(() => db.close());

  group('upsertAll', () {
    test('cùng id thì ghi đè, không nhân đôi', () async {
      await db.tripDao.upsertAll([row('T1', status: TripStatus.searching)]);
      await db.tripDao.upsertAll([row('T1')]);

      final rows = await db.select(db.trips).get();
      expect(rows, hasLength(1));
      expect(rows.single.status, TripStatus.completed);
    });

    test('dòng thiếu serviceKind nhận mặc định ride, driverId null', () async {
      await db.tripDao.upsertAll([row('T1')]);

      final saved = await db.tripDao.findById('T1');
      expect(saved?.serviceKind, ServiceKind.ride);
      expect(saved?.driverId, isNull);
    });

    test('database từ chối cước âm bằng CHECK, không ghi gì', () async {
      await expectLater(
        db.tripDao.upsertAll([row('T1'), row('T2', fareVnd: -1)]),
        throwsA(
          predicate(
            (e) => e.toString().contains('CHECK constraint failed: fare_vnd'),
          ),
        ),
      );
      // batch chạy trong một transaction: dòng T1 hợp lệ cũng không được ghi.
      expect(await db.select(db.trips).get(), isEmpty);
    });
  });

  group('lưu dạng nào trên đĩa', () {
    test('status lưu bằng tên enum, thời điểm bằng mili giây', () async {
      await db.tripDao.upsertAll([row('T1', requestedAtMs: 1762158600000)]);

      final raw = await db
          .customSelect('SELECT status, requested_at_ms FROM trips')
          .getSingle();
      expect(raw.read<String>('status'), 'completed');
      expect(raw.read<int>('requested_at_ms'), 1762158600000);
    });

    test('tên enum lạ trong database làm lần đọc ném ArgumentError', () async {
      await db.customStatement(
        "INSERT INTO trips (id, status, distance_meters, fare_vnd, "
        "requested_at_ms) VALUES ('X', 'teleporting', 1, 1, 1)",
      );

      await expectLater(
        db.tripDao.findById('X'),
        throwsA(
          isA<ArgumentError>().having(
            (e) => e.toString(),
            'message',
            contains('teleporting'),
          ),
        ),
      );
    });
  });

  group('watchRecent', () {
    test('mới trước, cùng thời điểm thì theo id, cắt theo limit', () async {
      await db.tripDao.upsertAll([
        row('B', requestedAtMs: 2000),
        row('A', requestedAtMs: 2000),
        row('C', requestedAtMs: 3000),
        row('D', requestedAtMs: 1000),
      ]);

      final all = await db.tripDao.watchRecent().first;
      expect(all.map((r) => r.id), ['C', 'A', 'B', 'D']);

      final top2 = await db.tripDao.watchRecent(limit: 2).first;
      expect(top2.map((r) => r.id), ['C', 'A']);
    });

    test('phát danh sách mới mỗi khi bảng đổi qua drift', () async {
      final emitted = <List<String>>[];
      final sub = db.tripDao.watchRecent().listen(
        (rows) => emitted.add([for (final r in rows) r.id]),
      );
      addTearDown(sub.cancel);

      await pumpEventQueue();
      await db.tripDao.upsertAll([row('T1', requestedAtMs: 1000)]);
      await pumpEventQueue();
      await db.tripDao.upsertAll([row('T2', requestedAtMs: 2000)]);
      await pumpEventQueue();
      await db.tripDao.deleteAll();
      await pumpEventQueue();

      expect(emitted, [
        <String>[],
        ['T1'],
        ['T2', 'T1'],
        <String>[],
      ]);
    });

    test('ghi lại dữ liệu y hệt vẫn phát lại', () async {
      final emitted = <int>[];
      final sub = db.tripDao.watchRecent().listen(
        (rows) => emitted.add(rows.length),
      );
      addTearDown(sub.cancel);

      await pumpEventQueue();
      await db.tripDao.upsertAll([row('T1')]);
      await pumpEventQueue();
      await db.tripDao.upsertAll([row('T1')]);
      await pumpEventQueue();

      expect(emitted, [0, 1, 1]);
    });

    test('customStatement không báo stream, notifyUpdates thì có', () async {
      final emitted = <int>[];
      final sub = db.tripDao.watchRecent().listen(
        (rows) => emitted.add(rows.length),
      );
      addTearDown(sub.cancel);
      await pumpEventQueue();

      await db.customStatement(
        "INSERT INTO trips (id, status, distance_meters, fare_vnd, "
        "requested_at_ms) VALUES ('T1', 'completed', 5000, 25500, 1)",
      );
      await pumpEventQueue();
      expect(emitted, [0], reason: 'stream chưa biết bảng đã đổi');

      db.notifyUpdates({
        TableUpdate.onTable(db.trips, kind: UpdateKind.insert),
      });
      await pumpEventQueue();
      expect(emitted, [0, 1]);
    });

    test('customInsert khai báo updates thì báo được cho stream', () async {
      final emitted = <int>[];
      final sub = db.tripDao.watchRecent().listen(
        (rows) => emitted.add(rows.length),
      );
      addTearDown(sub.cancel);
      await pumpEventQueue();

      await db.customInsert(
        "INSERT INTO trips (id, status, distance_meters, fare_vnd, "
        "requested_at_ms) VALUES ('T1', 'completed', 5000, 25500, 1)",
        updates: {db.trips},
      );
      await pumpEventQueue();

      expect(emitted, [0, 1]);
    });
  });

  group('findById', () {
    test('trả null khi không có', () async {
      expect(await db.tripDao.findById('none'), isNull);
    });
  });

  group('totalFareVnd', () {
    test('chỉ cộng chuyến đúng trạng thái; rỗng cho 0', () async {
      expect(await db.tripDao.totalFareVnd(TripStatus.completed), 0);

      await db.tripDao.upsertAll([
        row('T1', fareVnd: 25500),
        row('T2', fareVnd: 12005),
        row('T3', status: TripStatus.cancelled, fareVnd: 12000),
      ]);

      expect(await db.tripDao.totalFareVnd(TripStatus.completed), 37505);
      expect(await db.tripDao.totalFareVnd(TripStatus.cancelled), 12000);
      expect(await db.tripDao.totalFareVnd(TripStatus.inTransit), 0);
    });
  });

  group('deleteRequestedBefore', () {
    test('mốc cắt là cận loại trừ: chuyến đúng mốc được giữ', () async {
      await db.tripDao.upsertAll([
        row('old', requestedAtMs: 999),
        row('edge', requestedAtMs: 1000),
        row('new', requestedAtMs: 1001),
      ]);

      final removed = await db.tripDao.deleteRequestedBefore(
        DateTime.fromMillisecondsSinceEpoch(1000, isUtc: true),
      );

      expect(removed, 1);
      final left = await db.tripDao.watchRecent().first;
      expect(left.map((r) => r.id), ['new', 'edge']);
    });
  });
}
