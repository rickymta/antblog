import 'package:test/test.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

Trip _trip(String id, TripStatus status, int fareVnd) => Trip(
  id: id,
  serviceKind: ServiceKind.ride,
  status: status,
  distanceKm: 3,
  fare: Money(fareVnd),
  driverId: switch (status) {
    TripStatus.requested || TripStatus.searching => null,
    _ => 'D-1',
  },
  requestedAt: DateTime.utc(2026, 11, 6, 1),
);

void main() {
  group('TripIterableX', () {
    late List<Trip> trips;

    setUp(() {
      // Hai completed, một inTransit, một searching, một cancelled.
      trips = [
        _trip('T1', TripStatus.completed, 25500),
        _trip('T2', TripStatus.inTransit, 17400),
        _trip('T3', TripStatus.searching, 36750),
        _trip('T4', TripStatus.cancelled, 12000),
        _trip('T5', TripStatus.completed, 12000),
      ];
    });

    test('totalFare cộng cước mọi chuyến và trả Money', () {
      expect(trips.totalFare, const Money(103650));
      expect(trips.totalFare.toString(), '103.650 VND');
    });

    test('totalFare của danh sách rỗng là Money.zero', () {
      expect(<Trip>[].totalFare, Money.zero);
    });

    test('byStatus nhóm đúng và giữ thứ tự trong nhóm', () {
      final groups = trips.byStatus;
      expect(groups.keys.toSet(), {
        TripStatus.completed,
        TripStatus.inTransit,
        TripStatus.searching,
        TripStatus.cancelled,
      });
      expect(groups[TripStatus.completed]!.map((t) => t.id), ['T1', 'T5']);
      expect(groups[TripStatus.accepted], isNull);
    });

    test('active bỏ completed và cancelled, là Iterable lười', () {
      final active = trips.active;
      expect(active.map((t) => t.id), ['T2', 'T3']);
      expect(active.totalFare, const Money(17400 + 36750));

      // Lười: thêm chuyến sau khi tạo `active` vẫn được thấy khi duyệt.
      trips.add(_trip('T6', TripStatus.accepted, 10000));
      expect(active.map((t) => t.id), ['T2', 'T3', 'T6']);
    });

    test('extension chạy trên Set và pipeline lười, không chỉ List', () {
      expect(trips.toSet().totalFare, trips.totalFare);
      expect(trips.map((t) => t).totalFare, const Money(103650));
    });
  });
}
