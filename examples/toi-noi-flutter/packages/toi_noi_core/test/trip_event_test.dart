import 'package:test/test.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

void main() {
  group('TripEvent bằng nhau theo giá trị', () {
    test('DriverAccepted so theo driverId', () {
      expect(const DriverAccepted('D-1'), const DriverAccepted('D-1'));
      expect(
        const DriverAccepted('D-1').hashCode,
        const DriverAccepted('D-1').hashCode,
      );
      expect(
        const DriverAccepted('D-1'),
        isNot(equals(const DriverAccepted('D-2'))),
      );
    });

    test('Cancelled so theo by', () {
      const byCustomer = Cancelled(by: CancelledBy.customer);
      expect(byCustomer, const Cancelled(by: CancelledBy.customer));
      expect(
        byCustomer.hashCode,
        const Cancelled(by: CancelledBy.customer).hashCode,
      );
      expect(
        byCustomer,
        isNot(equals(const Cancelled(by: CancelledBy.driver))),
      );
    });

    test('hai biến thể khác loại không bằng nhau', () {
      expect(const PickedUp() == const TripStarted(), isFalse);
      expect(const SearchStarted(), const SearchStarted());
    });

    test('đưa được vào Set mà không nhân đôi', () {
      final events = <TripEvent>{}
        ..add(const DriverAccepted('D-1'))
        ..add(const DriverAccepted('D-1'))
        ..add(const Cancelled(by: CancelledBy.driver));
      expect(events, hasLength(2));
    });
  });
}
