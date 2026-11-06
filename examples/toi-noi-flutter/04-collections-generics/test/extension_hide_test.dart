import 'package:test/test.dart';
import 'package:toi_noi_04/model.dart';
import 'package:toi_noi_04/summary_long.dart';
// hide loại extension không dùng: `summary` chỉ còn một nghĩa, mà vẫn dùng được
// pluralTrips từ cùng file.
import 'package:toi_noi_04/summary_short.dart' hide TripSummaryShort;

import 'fixtures.dart';

void main() {
  test('hide một extension thì trips.summary dùng extension còn lại', () {
    final trips = <Trip>[makeTrip('T1', TripStatus.completed, 25500)];
    expect(trips.summary, '1 chuyến, tổng 25500 VND');
    expect(pluralTrips(trips.length), '1 chuyến');
  });
}
