import 'package:collection/collection.dart' show IterableExtension, groupBy;

import 'model.dart';

/// Thêm truy vấn nghiệp vụ cho mọi `Iterable<Trip>` mà không kế thừa gì.
extension TripIterableX on Iterable<Trip> {
  /// Tổng cước bằng fold: bắt đầu từ 0, cộng dồn từng chuyến.
  int get totalFareVnd => fold(0, (sum, trip) => sum + trip.fareVnd);

  /// Nhóm theo trạng thái. Dùng groupBy của package:collection.
  Map<TripStatus, List<Trip>> get byStatus =>
      groupBy(this, (trip) => trip.status);

  /// Chuyến chưa kết thúc. Trả Iterable lười: chưa lọc gì cho tới khi duyệt.
  Iterable<Trip> get active => where((trip) => !trip.status.isTerminal);

  /// Tất cả đã kết thúc (every). Iterable rỗng cho true.
  bool get allSettled => every((trip) => trip.status.isTerminal);

  /// Chuyến cước cao nhất (reduce). Rỗng thì reduce ném StateError, nên chặn trước.
  Trip? get priciest => isEmpty
      ? null
      : reduce((best, trip) => trip.fareVnd > best.fareVnd ? trip : best);

  /// Cước giảm dần, giữ thứ tự ban đầu khi cước bằng nhau (sắp xếp ổn định).
  ///
  /// sortedByCompare được cài đặt bằng mergeSortBy (merge sort, ổn định); test
  /// 200 chuyến cước trùng nhau giữ hành vi này. Còn `sorted` của
  /// package:collection gọi List.sort, không cam kết ổn định.
  List<Trip> get sortedByFareDesc =>
      sortedByCompare((trip) => trip.fareVnd, (a, b) => b.compareTo(a));
}

/// Cùng kết quả với `byStatus` nhưng viết tay, để thấy groupBy làm gì.
Map<TripStatus, List<Trip>> groupByStatusManually(Iterable<Trip> trips) {
  final groups = <TripStatus, List<Trip>>{};
  for (final trip in trips) {
    groups.putIfAbsent(trip.status, () => []).add(trip);
  }
  return groups;
}
