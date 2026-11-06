import 'model.dart';

/// Đếm side effect để thấy Iterable lười chạy khi nào.
class QueryProbe {
  int predicateCalls = 0;
  int projectionCalls = 0;

  /// where + map: chỉ dựng pipeline, chưa gọi hàm nào của hai bước.
  Iterable<String> activeIds(Iterable<Trip> trips) => trips
      .where((trip) {
        predicateCalls++;
        return !trip.status.isTerminal;
      })
      .map((trip) {
        projectionCalls++;
        return trip.id;
      });
}

/// Nhận Iterable: truyền List, Set hay một pipeline lười đều được.
int countCancellable(Iterable<Trip> trips) =>
    trips.where((trip) => trip.status.canCancel).length;

/// Sai: sửa list đang duyệt, lần lặp kế tiếp ném ConcurrentModificationError.
void removeCompletedWrong(List<Trip> trips) {
  for (final trip in trips) {
    if (trip.status == TripStatus.completed) trips.remove(trip);
  }
}

/// Đúng: removeWhere sửa tại chỗ an toàn, không cần vòng for.
void removeCompleted(List<Trip> trips) =>
    trips.removeWhere((trip) => trip.status == TripStatus.completed);
