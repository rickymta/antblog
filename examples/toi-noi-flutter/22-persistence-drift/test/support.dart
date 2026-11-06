import 'package:drift/drift.dart';
import 'package:drift/native.dart';
import 'package:toi_noi_22/db/app_database.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Database trong bộ nhớ, schema mới nhất, không đụng file hay plugin.
///
/// `closeStreamsSynchronously` được drift khuyến nghị cho widget test: không
/// để timer đóng stream còn treo khi test kết thúc.
AppDatabase openMemoryDatabase() => AppDatabase(
  DatabaseConnection(NativeDatabase.memory(), closeStreamsSynchronously: true),
);

/// Mốc thời gian cố định để test không phụ thuộc đồng hồ thật.
final DateTime t0 = DateTime.utc(2026, 11, 3, 8, 30);

/// Chuyến mẫu; cước luôn do `calculateFare` tính từ [km].
Trip sampleTrip(
  String id, {
  double km = 5,
  ServiceKind kind = ServiceKind.ride,
  DateTime? requestedAt,
  TripStatus status = TripStatus.requested,
  String? driverId,
}) => Trip(
  id: id,
  serviceKind: kind,
  status: status,
  distanceKm: km,
  fare: calculateFare(km),
  driverId: driverId,
  requestedAt: requestedAt ?? t0,
);
