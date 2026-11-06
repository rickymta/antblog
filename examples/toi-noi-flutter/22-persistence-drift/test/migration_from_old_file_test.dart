import 'package:drift/drift.dart' hide isNull;
import 'package:drift/native.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_22/db/app_database.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Cách thứ hai kiểm migration, không cần file schema do công cụ sinh: dựng
/// tay một database đúng như app bản cũ để lại trên máy người dùng, rồi mở
/// bằng code hiện tại và để drift tự chạy `onUpgrade`.
const _schemaV1 = '''
CREATE TABLE trips (
  id TEXT NOT NULL,
  status TEXT NOT NULL,
  distance_meters INTEGER NOT NULL,
  fare_vnd INTEGER NOT NULL CHECK (fare_vnd >= 0),
  requested_at_ms INTEGER NOT NULL,
  PRIMARY KEY (id)
)''';

AppDatabase openAtVersion1() => AppDatabase(
  DatabaseConnection(
    // `setup` chạy trên kết nối SQLite thô trước khi drift đọc user_version.
    NativeDatabase.memory(
      setup: (raw) {
        raw.execute(_schemaV1);
        raw.execute(
          "INSERT INTO trips VALUES ('T1', 'completed', 5000, 25500, 1762158600000)",
        );
        raw.execute('PRAGMA user_version = 1');
      },
    ),
    closeStreamsSynchronously: true,
  ),
);

void main() {
  test('mở database cũ: drift nâng lên schema 2 và giữ dòng cũ', () async {
    final db = openAtVersion1();
    addTearDown(db.close);

    final row = await db.tripDao.findById('T1');
    expect(row?.status, TripStatus.completed);
    expect(row?.fareVnd, 25500);
    expect(row?.serviceKind, ServiceKind.ride, reason: 'mặc định của cột mới');
    expect(row?.driverId, isNull);

    final version = await db.customSelect('PRAGMA user_version').getSingle();
    expect(version.read<int>('user_version'), 2);
  });

  test('sau khi nâng cấp, ghi và đọc dòng schema 2 bình thường', () async {
    final db = openAtVersion1();
    addTearDown(db.close);

    await db.tripDao.upsertAll([
      TripsCompanion.insert(
        id: 'T2',
        status: TripStatus.accepted,
        distanceMeters: 2001,
        fareVnd: 12005,
        requestedAtMs: 1762162200000,
        serviceKind: const Value(ServiceKind.food),
        driverId: const Value('D-07'),
      ),
    ]);

    final row = await db.tripDao.findById('T2');
    expect(row?.serviceKind, ServiceKind.food);
    expect(row?.driverId, 'D-07');
    expect(await db.select(db.trips).get(), hasLength(2));
  });

  test(
    'app bản cũ mở database mới hơn: StateError, migration chỉ đi lên',
    () async {
      final db = AppDatabase(
        NativeDatabase.memory(
          setup: (raw) => raw.execute('PRAGMA user_version = 3'),
        ),
      );
      addTearDown(db.close);

      await expectLater(
        db.select(db.trips).get(),
        throwsA(
          isA<StateError>().having(
            (e) => e.message,
            'message',
            contains('downgrade'),
          ),
        ),
      );
    },
  );
}
