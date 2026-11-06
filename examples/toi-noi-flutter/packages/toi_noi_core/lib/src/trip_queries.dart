import 'package:collection/collection.dart' show groupBy;

import 'money.dart';
import 'trip.dart';
import 'trip_status.dart';

/// Truy vấn nghiệp vụ cho mọi `Iterable<Trip>`, không cần kế thừa gì.
extension TripIterableX on Iterable<Trip> {
  /// Tổng cước của mọi chuyến; danh sách rỗng cho [Money.zero].
  Money get totalFare => fold(Money.zero, (sum, trip) => sum + trip.fare);

  /// Nhóm theo trạng thái, giữ thứ tự ban đầu trong từng nhóm.
  Map<TripStatus, List<Trip>> get byStatus =>
      groupBy(this, (trip) => trip.status);

  /// Chuyến chưa kết thúc. Trả `Iterable` lười: chưa lọc gì cho tới khi duyệt.
  Iterable<Trip> get active => where((trip) => !trip.status.isTerminal);
}
