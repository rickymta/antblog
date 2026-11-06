import 'package:test/test.dart';
import 'package:toi_noi_06/report.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

Trip _newTrip(double km) => Trip.request(
  id: 'TN-9',
  serviceKind: ServiceKind.ride,
  distanceKm: km,
  requestedAt: DateTime.utc(2026, 11, 6, 8),
);

void main() {
  group('quoteLine', () {
    test('dùng Money.toString của package', () {
      expect(quoteLine(5), '5.0 km: 25.500 VND');
      expect(quoteLine(2.001), '2.001 km: 12.005 VND');
    });

    test('lỗi của package đi xuyên qua: ngoài miền ném ArgumentError', () {
      expect(() => quoteLine(100.5), throwsArgumentError);
    });
  });

  group('describeEvent', () {
    test('có mô tả cho cả 7 biến thể (8 kể cả hai CancelledBy)', () {
      const events = <TripEvent>[
        SearchStarted(),
        DriverAccepted('D-1'),
        DriverArrived(),
        PickedUp(),
        TripStarted(),
        Completed(),
        Cancelled(by: CancelledBy.customer),
        Cancelled(by: CancelledBy.driver),
      ];
      final texts = events.map(describeEvent).toSet();
      expect(texts, hasLength(8));
      expect(describeEvent(const DriverAccepted('D-1')), contains('D-1'));
    });
  });

  group('runTrip', () {
    test('chuyến hoàn thành: 1 dòng đầu + 6 dòng sự kiện', () {
      final lines = runTrip(_newTrip(5), const [
        SearchStarted(),
        DriverAccepted('D-17'),
        DriverArrived(),
        PickedUp(),
        TripStarted(),
        Completed(),
      ]);
      expect(lines, hasLength(7));
      expect(lines.first, 'TN-9: Đã đặt');
      expect(lines.last, 'hoàn thành -> Hoàn thành');
    });

    test('không sửa chuyến gốc', () {
      final trip = _newTrip(5);
      runTrip(trip, const [SearchStarted()]);
      expect(trip.status, TripStatus.requested);
    });

    test('sự kiện sai thứ tự ném StateError', () {
      expect(() => runTrip(_newTrip(5), const [Completed()]), throwsStateError);
    });
  });

  group('truy vấn từ extension của package', () {
    test('totalFare cộng Money của hai chuyến', () {
      expect([_newTrip(5), _newTrip(2)].totalFare, const Money(37500));
    });
  });
}
