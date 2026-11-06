import 'package:collection/collection.dart';
import 'package:test/test.dart';
import 'package:toi_noi_04/lazy_demo.dart';
import 'package:toi_noi_04/model.dart';

import 'fixtures.dart';

void main() {
  group('Iterable lười', () {
    test('dựng where + map chưa chạy hàm nào: bộ đếm vẫn bằng 0', () {
      final probe = QueryProbe();
      final trips = fiveTrips();
      final ids = probe.activeIds(trips);
      expect(probe.predicateCalls, 0);
      expect(probe.projectionCalls, 0);
      expect(ids, isA<Iterable<String>>());
      expect(ids, isNot(isA<List<String>>()));
    });

    test(
      'toList duyệt hết: where chạy n lần, map chỉ chạy cho chuyến qua lọc',
      () {
        final probe = QueryProbe();
        final trips = fiveTrips();
        final snapshot = probe.activeIds(trips).toList();
        expect(snapshot, ['T2', 'T3']);
        expect(probe.predicateCalls, trips.length); // n = 5
        expect(probe.projectionCalls, 2);
      },
    );

    test('duyệt hai lần thì đếm hai lần', () {
      final probe = QueryProbe();
      final ids = probe.activeIds(fiveTrips());
      ids.toList();
      ids.toList();
      expect(probe.predicateCalls, 10);
      expect(probe.projectionCalls, 4);
    });

    test('length trên Iterable lười cũng duyệt lại mỗi lần gọi', () {
      final probe = QueryProbe();
      final ids = probe.activeIds(fiveTrips());
      expect(ids.length, 2);
      expect(ids.length, 2);
      expect(probe.predicateCalls, 10);
    });

    test('first dừng sớm: chỉ kiểm tra tới chuyến đầu tiên thoả điều kiện', () {
      final probe = QueryProbe();
      expect(probe.activeIds(fiveTrips()).first, 'T2');
      expect(probe.predicateCalls, 2); // T1 bị loại, T2 qua rồi dừng
      expect(probe.projectionCalls, 1);
    });

    test(
      'first rồi toList: bộ đếm cộng dồn vì mỗi lần duyệt chạy lại từ đầu',
      () {
        final probe = QueryProbe();
        final ids = probe.activeIds(fiveTrips());
        expect(ids.first, 'T2');
        expect(ids.toList(), ['T2', 'T3']);
        expect(probe.predicateCalls, 2 + 5);
        expect(probe.projectionCalls, 1 + 2);
      },
    );

    test(
      'toList là snapshot, còn pipeline lười nhìn thấy thay đổi của nguồn',
      () {
        final probe = QueryProbe();
        final trips = fiveTrips();
        final lazyIds = probe.activeIds(trips);
        final snapshot = lazyIds.toList();

        trips.add(makeTrip('T6', TripStatus.requested, 21000));

        expect(snapshot, ['T2', 'T3']);
        expect(lazyIds.toList(), ['T2', 'T3', 'T6']);
      },
    );

    test('hàm nhận Iterable chạy được với List, Set và pipeline lười', () {
      final trips = [
        makeTrip('T1', TripStatus.searching, 1),
        makeTrip('T2', TripStatus.accepted, 1),
        makeTrip('T3', TripStatus.inTransit, 1),
      ];
      expect(countCancellable(trips), 2);
      expect(countCancellable(trips.toSet()), 2);
      expect(countCancellable(trips.where((t) => t.id != 'T1')), 1);
    });
  });

  group('Lỗi thường gặp', () {
    test('sửa list đang duyệt ném ConcurrentModificationError', () {
      final trips = fiveTrips();
      expect(
        () => removeCompletedWrong(trips),
        throwsA(isA<ConcurrentModificationError>()),
      );
    });

    test('removeWhere xoá an toàn mọi chuyến completed', () {
      final trips = fiveTrips();
      removeCompleted(trips);
      expect(trips.map((t) => t.id), ['T2', 'T3', 'T4']);
    });

    test('== của List so sánh tham chiếu, không so sánh nội dung', () {
      final a = [1, 2, 3];
      final b = [1, 2, 3];
      expect(a == b, isFalse);
      expect(a == a, isTrue);
      expect(a, equals(b)); // matcher của package:test so sánh sâu
    });

    test('List.from không giữ kiểu, toList và List.of thì giữ', () {
      final trips = fiveTrips();
      final fromList = List.from(trips); // List<dynamic>
      expect(fromList, isNot(isA<List<Trip>>()));
      expect(trips.toList(), isA<List<Trip>>());
      expect(List.of(trips), isA<List<Trip>>());
    });

    test('sort sửa tại chỗ, sorted trả list mới', () {
      final ids = ['b', 'c', 'a'];
      final fresh = ids.sorted((a, b) => a.compareTo(b));
      expect(fresh, ['a', 'b', 'c']);
      expect(ids, ['b', 'c', 'a']); // sorted không đụng tới list gốc
      ids.sort(); // sort trả void và đổi chính list này
      expect(ids, ['a', 'b', 'c']);
    });
  });
}
