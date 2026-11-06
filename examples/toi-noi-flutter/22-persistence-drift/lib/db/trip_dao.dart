import 'package:drift/drift.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'app_database.dart';
import 'trips_table.dart';

part 'trip_dao.g.dart';

/// Mọi câu truy vấn trên bảng `trips` nằm ở đây, trả về `TripRow` của drift.
///
/// DAO không biết `Trip` của domain; việc đổi qua lại do repository làm.
@DriftAccessor(tables: [Trips])
class TripDao extends DatabaseAccessor<AppDatabase> with _$TripDaoMixin {
  TripDao(super.attachedDatabase);

  /// Chèn hoặc ghi đè theo `id`: cùng một chuyến tải lại từ server không nhân đôi.
  Future<void> upsertAll(List<TripsCompanion> rows) =>
      batch((batch) => batch.insertAllOnConflictUpdate(trips, rows));

  /// Stream [limit] chuyến gần nhất, mới trước. Phát lại mỗi khi bảng đổi.
  Stream<List<TripRow>> watchRecent({int limit = 20}) {
    final query = select(trips)
      ..orderBy([
        (t) => OrderingTerm.desc(t.requestedAtMs),
        // Cùng thời điểm thì xếp theo id để thứ tự luôn cố định.
        (t) => OrderingTerm.asc(t.id),
      ])
      ..limit(limit);
    return query.watch();
  }

  Future<TripRow?> findById(String id) =>
      (select(trips)..where((t) => t.id.equals(id))).getSingleOrNull();

  /// Tổng cước của các chuyến ở trạng thái [status], tính trong SQL.
  Future<int> totalFareVnd(TripStatus status) async {
    final sum = trips.fareVnd.sum();
    final query = selectOnly(trips)
      ..addColumns([sum])
      ..where(trips.status.equalsValue(status));
    final row = await query.getSingle();
    return row.read(sum) ?? 0;
  }

  /// Xoá các chuyến đặt trước [cutoffUtc]; trả số dòng đã xoá.
  Future<int> deleteRequestedBefore(DateTime cutoffUtc) =>
      (delete(trips)..where(
            (t) => t.requestedAtMs.isSmallerThanValue(
              cutoffUtc.millisecondsSinceEpoch,
            ),
          ))
          .go();

  Future<int> deleteAll() => delete(trips).go();
}
