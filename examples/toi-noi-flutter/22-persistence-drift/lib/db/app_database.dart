import 'package:drift/drift.dart';
import 'package:drift_flutter/drift_flutter.dart';
import 'package:path_provider/path_provider.dart';
// Mã sinh ra trong app_database.g.dart dùng TripStatus và ServiceKind từ đây.
import 'package:toi_noi_core/toi_noi_core.dart';

import 'app_database.steps.dart';
import 'trip_dao.dart';
import 'trips_table.dart';

part 'app_database.g.dart';

/// Database SQLite của app: hiện chỉ có lịch sử chuyến.
///
/// Constructor nhận `QueryExecutor` tuỳ chọn: app để trống và dùng file thật
/// qua `drift_flutter`, test truyền `NativeDatabase.memory()` hoặc một kết nối
/// tới file do `SchemaVerifier` dựng.
@DriftDatabase(tables: [Trips], daos: [TripDao])
class AppDatabase extends _$AppDatabase {
  AppDatabase([QueryExecutor? executor]) : super(executor ?? _openOnDevice());

  /// File `toi_noi.sqlite` trong thư mục application support: chỗ dành cho
  /// file nội bộ của app, không phải thư mục dữ liệu người dùng. drift_flutter
  /// mở nó trong một isolate nền nên truy vấn không chặn UI.
  static QueryExecutor _openOnDevice() => driftDatabase(
    name: 'toi_noi',
    native: const DriftNativeOptions(
      databaseDirectory: getApplicationSupportDirectory,
    ),
  );

  /// Tăng mỗi khi schema đổi, và thêm một bước `fromXToY` tương ứng.
  @override
  int get schemaVersion => 2;

  @override
  MigrationStrategy get migration => MigrationStrategy(
    onUpgrade: stepByStep(
      // Schema 2: thêm loại dịch vụ và tài xế. Dòng cũ nhận `ride` nhờ giá trị
      // mặc định của cột; `addColumn` chạy ALTER TABLE ... ADD COLUMN.
      from1To2: (m, schema) async {
        await m.addColumn(schema.trips, schema.trips.serviceKind);
        await m.addColumn(schema.trips, schema.trips.driverId);
      },
    ),
  );
}
