import 'model.dart';

/// Số chuyến kèm đơn vị. Có thêm hàm này để `hide TripSummaryShort` vẫn còn
/// một tên khác cần import từ file.
String pluralTrips(int count) => '$count chuyến';

/// Hai extension khác tên nhưng cùng member `summary`: ví dụ xung đột.
extension TripSummaryShort on Iterable<Trip> {
  String get summary => pluralTrips(length);
}
