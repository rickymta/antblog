import 'package:test/test.dart';
import 'package:toi_noi_04/model.dart';
// Import cả hai: gọi `trips.summary` sẽ là lỗi biên dịch vì không biết chọn
// TripSummaryShort hay TripSummaryLong. Áp extension tường minh để chọn.
import 'package:toi_noi_04/summary_long.dart';
import 'package:toi_noi_04/summary_short.dart';

import 'fixtures.dart';

void main() {
  test('áp extension tường minh để giải xung đột tên member', () {
    final trips = <Trip>[
      makeTrip('T1', TripStatus.completed, 25500),
      makeTrip('T2', TripStatus.searching, 12000),
    ];
    expect(TripSummaryShort(trips).summary, '2 chuyến');
    expect(TripSummaryLong(trips).summary, '2 chuyến, tổng 37500 VND');
  });
}
