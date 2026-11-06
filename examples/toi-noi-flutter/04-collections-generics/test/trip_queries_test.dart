import 'package:test/test.dart';
import 'package:toi_noi_04/model.dart';
import 'package:toi_noi_04/trip_queries.dart';

import 'fixtures.dart';

void main() {
  group('TripIterableX', () {
    test('totalFareVnd cộng cước mọi chuyến; rỗng thì bằng 0', () {
      expect(fiveTrips().totalFareVnd, 25500 + 17400 + 36750 + 12000 + 12000);
      expect(<Trip>[].totalFareVnd, 0);
    });

    test('byStatus nhóm đúng và giữ thứ tự trong nhóm', () {
      final groups = fiveTrips().byStatus;
      expect(groups.keys.toSet(), {
        TripStatus.completed,
        TripStatus.inTransit,
        TripStatus.searching,
        TripStatus.cancelled,
      });
      expect(groups[TripStatus.completed]!.map((t) => t.id), ['T1', 'T5']);
      expect(groups[TripStatus.accepted], isNull);
    });

    test('groupBy của package:collection khớp bản viết tay', () {
      final trips = fiveTrips();
      final manual = groupByStatusManually(trips);
      final fromPackage = trips.byStatus;
      expect(fromPackage.keys, manual.keys);
      for (final status in manual.keys) {
        expect(
          fromPackage[status]!.map((t) => t.id),
          manual[status]!.map((t) => t.id),
        );
      }
    });

    test('active bỏ completed và cancelled', () {
      final active = fiveTrips().active;
      expect(active.map((t) => t.id), ['T2', 'T3']);
      expect(active.totalFareVnd, 17400 + 36750);
    });

    test('allSettled và priciest', () {
      expect(fiveTrips().allSettled, isFalse);
      expect(fiveTrips().where((t) => t.status.isTerminal).allSettled, isTrue);
      expect(<Trip>[].allSettled, isTrue);
      expect(fiveTrips().priciest!.id, 'T3');
      expect(<Trip>[].priciest, isNull);
    });

    test('sortedByFareDesc giảm dần và không sửa danh sách gốc', () {
      final trips = fiveTrips();
      final sorted = trips.sortedByFareDesc;
      expect(sorted.map((t) => t.fareVnd), [36750, 25500, 17400, 12000, 12000]);
      expect(trips.map((t) => t.id), ['T1', 'T2', 'T3', 'T4', 'T5']);
      expect(identical(sorted, trips), isFalse);
    });

    test('sortedByFareDesc ổn định: cước bằng nhau giữ thứ tự ban đầu', () {
      // 200 chuyến, chỉ 3 mức cước: nhiều cặp bằng nhau, vượt xa ngưỡng
      // insertion sort của List.sort.
      final trips = [
        for (var i = 0; i < 200; i++)
          makeTrip('T$i', TripStatus.completed, (i % 3 + 1) * 10000),
      ];
      final sorted = trips.sortedByFareDesc;
      for (var i = 1; i < sorted.length; i++) {
        final prev = sorted[i - 1];
        final cur = sorted[i];
        expect(prev.fareVnd >= cur.fareVnd, isTrue);
        if (prev.fareVnd == cur.fareVnd) {
          final prevIndex = int.parse(prev.id.substring(1));
          final curIndex = int.parse(cur.id.substring(1));
          expect(
            prevIndex < curIndex,
            isTrue,
            reason: '${prev.id} phải đứng trước ${cur.id}',
          );
        }
      }
    });

    test('extension chạy trên Set và trên pipeline lười, không chỉ List', () {
      final set = fiveTrips().toSet();
      expect(set.totalFareVnd, fiveTrips().totalFareVnd);
      expect(fiveTrips().map((t) => t).totalFareVnd, 103650);
    });
  });
}
