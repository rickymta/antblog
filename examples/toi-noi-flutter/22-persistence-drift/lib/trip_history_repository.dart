import 'package:drift/drift.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'db/app_database.dart';
import 'db/trip_dao.dart';

/// Lịch sử chuyến lưu trên máy, nhìn từ phía domain: chỉ có `Trip`.
abstract interface class TripHistoryRepository {
  /// [limit] chuyến gần nhất, mới trước; phát lại khi dữ liệu đổi.
  Stream<List<Trip>> watchRecent({int limit = 20});

  Future<Trip?> findById(String id);

  /// Lưu (chèn hoặc ghi đè theo `id`) các chuyến lấy từ server.
  Future<void> saveAll(List<Trip> trips);

  /// Xoá toàn bộ; trả số chuyến đã xoá.
  Future<int> clear();
}

/// Bản cài bằng drift.
class DriftTripHistoryRepository implements TripHistoryRepository {
  DriftTripHistoryRepository(this._dao);

  final TripDao _dao;

  @override
  Stream<List<Trip>> watchRecent({int limit = 20}) => _dao
      .watchRecent(limit: limit)
      .map((rows) => [for (final row in rows) row.toTrip()]);

  @override
  Future<Trip?> findById(String id) async =>
      (await _dao.findById(id))?.toTrip();

  @override
  Future<void> saveAll(List<Trip> trips) =>
      _dao.upsertAll([for (final trip in trips) trip.toCompanion()]);

  @override
  Future<int> clear() => _dao.deleteAll();
}

/// Đổi dòng database thành `Trip`.
///
/// Dữ liệu đọc từ đĩa là dữ liệu ngoài: `Money.fromVnd` và constructor của
/// `Trip` kiểm lại, dòng hỏng ném `ArgumentError` thay vì trở thành chuyến sai.
extension TripRowToTrip on TripRow {
  Trip toTrip() => Trip(
    id: id,
    serviceKind: serviceKind,
    status: status,
    distanceKm: distanceMeters / 1000,
    fare: Money.fromVnd(fareVnd),
    driverId: driverId,
    requestedAt: DateTime.fromMillisecondsSinceEpoch(
      requestedAtMs,
      isUtc: true,
    ),
  );
}

/// Đổi `Trip` thành dòng để ghi.
///
/// Cước lưu `fare.vnd`, quãng đường lưu mét nguyên, thời điểm lưu mili giây
/// UTC. Độ chính xác dưới mili giây của `requestedAt` bị cắt.
extension TripToCompanion on Trip {
  TripsCompanion toCompanion() => TripsCompanion(
    id: Value(id),
    serviceKind: Value(serviceKind),
    status: Value(status),
    distanceMeters: Value((distanceKm * 1000).round()),
    fareVnd: Value(fare.vnd),
    driverId: Value(driverId),
    requestedAtMs: Value(requestedAt.millisecondsSinceEpoch),
  );
}
