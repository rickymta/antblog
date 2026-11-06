import 'package:toi_noi_core/toi_noi_core.dart';

/// Một dòng báo giá: `"5.0 km: 25.500 VND"`.
String quoteLine(double distanceKm) =>
    '$distanceKm km: ${calculateFare(distanceKm)}';

/// Mô tả sự kiện cho nhật ký.
///
/// `switch` trên `sealed class` của package khác vẫn bị bắt buộc vét cạn:
/// nếu `toi_noi_core` thêm biến thể mới, `dart analyze` ở đây sẽ báo lỗi.
String describeEvent(TripEvent event) => switch (event) {
  SearchStarted() => 'bắt đầu tìm tài xế',
  DriverAccepted(:final driverId) => 'tài xế $driverId nhận chuyến',
  DriverArrived() => 'tài xế đang tới điểm đón',
  PickedUp() => 'đã đón khách',
  TripStarted() => 'bắt đầu chở',
  Completed() => 'hoàn thành',
  Cancelled(:final by) => switch (by) {
    CancelledBy.customer => 'khách huỷ',
    CancelledBy.driver => 'tài xế huỷ',
  },
};

/// Chạy [events] lần lượt trên [trip] và trả về từng dòng nhật ký.
///
/// Dòng đầu là trạng thái ban đầu; mỗi sự kiện thêm một dòng
/// `"sự kiện -> trạng thái"`. Sự kiện không hợp lệ ném `StateError` như
/// `Trip.apply`.
List<String> runTrip(Trip trip, List<TripEvent> events) {
  final lines = <String>['${trip.id}: ${trip.status.label}'];
  var current = trip;
  for (final event in events) {
    current = current.apply(event);
    lines.add('${describeEvent(event)} -> ${current.status.label}');
  }
  return lines;
}
