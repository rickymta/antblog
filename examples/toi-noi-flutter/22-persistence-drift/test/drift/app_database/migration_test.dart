// dart format width=80
// ignore_for_file: unused_local_variable, unused_import
import 'package:drift/drift.dart';
import 'package:drift_dev/api/migrations_native.dart';
import 'package:toi_noi_22/db/app_database.dart';
import 'package:flutter_test/flutter_test.dart';

import 'generated/schema.dart';

import 'generated/schema_v1.dart' as v1;
import 'generated/schema_v2.dart' as v2;

void main() {
  driftRuntimeOptions.dontWarnAboutMultipleDatabases = true;
  late SchemaVerifier verifier;

  setUpAll(() {
    verifier = SchemaVerifier(GeneratedHelper());
  });

  group('simple database migrations', () {
    // These simple tests verify all possible schema updates with a simple (no
    // data) migration. This is a quick way to ensure that written database
    // migrations properly alter the schema.
    const versions = GeneratedHelper.versions;
    for (final (i, fromVersion) in versions.indexed) {
      group('from $fromVersion', () {
        for (final toVersion in versions.skip(i + 1)) {
          test('to $toVersion', () async {
            final schema = await verifier.schemaAt(fromVersion);
            final db = AppDatabase(schema.newConnection());
            await verifier.migrateAndValidate(db, toVersion);
            await db.close();
          });
        }
      });
    }
  });

  // Thêm cột không đổi dữ liệu cũ, nhưng ta vẫn kiểm: dòng v1 phải sống sót,
  // nhận `ride` và tài xế null ở schema 2.
  test('migration from v1 to v2 does not corrupt data', () async {
    const oldTripsData = <v1.TripsData>[
      v1.TripsData(
        id: 'T1',
        status: 'completed',
        distanceMeters: 5000,
        fareVnd: 25500,
        requestedAtMs: 1762158600000,
      ),
    ];
    const expectedNewTripsData = <v2.TripsData>[
      v2.TripsData(
        id: 'T1',
        status: 'completed',
        distanceMeters: 5000,
        fareVnd: 25500,
        requestedAtMs: 1762158600000,
        serviceKind: 'ride',
      ),
    ];

    await verifier.testWithDataIntegrity(
      oldVersion: 1,
      newVersion: 2,
      createOld: v1.DatabaseAtV1.new,
      createNew: v2.DatabaseAtV2.new,
      openTestedDatabase: AppDatabase.new,
      createItems: (batch, oldDb) {
        batch.insertAll(oldDb.trips, oldTripsData);
      },
      validateItems: (newDb) async {
        expect(await newDb.select(newDb.trips).get(), expectedNewTripsData);
      },
    );
  });
}
