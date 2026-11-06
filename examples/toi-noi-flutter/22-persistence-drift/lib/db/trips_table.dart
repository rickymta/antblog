import 'package:drift/drift.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Bảng lịch sử chuyến đi lưu trên máy (schema phiên bản 2).
@DataClassName('TripRow')
class Trips extends Table {
  /// Mã chuyến do server cấp, dùng làm khoá chính.
  TextColumn get id => text()();

  /// Tên của enum `TripStatus` (ví dụ `completed`), không lưu chỉ số.
  TextColumn get status => textEnum<TripStatus>()();

  /// Quãng đường bằng mét nguyên.
  IntColumn get distanceMeters => integer()();

  /// Cước bằng đồng nguyên; database từ chối số âm.
  // ignore: recursive_getters
  IntColumn get fareVnd => integer().check(fareVnd.isBiggerOrEqualValue(0))();

  /// Thời điểm đặt, UTC, tính bằng mili giây từ epoch.
  IntColumn get requestedAtMs => integer()();

  /// Tên của enum `ServiceKind`. Thêm ở schema 2; dòng cũ nhận `ride`.
  TextColumn get serviceKind =>
      textEnum<ServiceKind>().withDefault(Constant(ServiceKind.ride.name))();

  /// Tài xế nhận chuyến; null khi chưa có. Thêm ở schema 2.
  TextColumn get driverId => text().nullable()();

  @override
  Set<Column<Object>> get primaryKey => {id};
}
