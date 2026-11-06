import 'package:test/test.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

final _requestedAt = DateTime.utc(2026, 11, 6, 8, 30);

Trip _tripAt(TripStatus status) => Trip(
  id: 'TN-1',
  serviceKind: ServiceKind.ride,
  status: status,
  distanceKm: 5,
  fare: const Money(25500),
  driverId: switch (status) {
    TripStatus.requested || TripStatus.searching => null,
    _ => 'D-17',
  },
  requestedAt: _requestedAt,
);

/// Mỗi sự kiện mẫu có một tên ngắn để làm khoá của bảng kiểm.
const Map<String, TripEvent> _events = {
  'searchStarted': SearchStarted(),
  'driverAccepted': DriverAccepted('D-17'),
  'driverArrived': DriverArrived(),
  'pickedUp': PickedUp(),
  'tripStarted': TripStarted(),
  'completed': Completed(),
  'cancelledByCustomer': Cancelled(by: CancelledBy.customer),
  'cancelledByDriver': Cancelled(by: CancelledBy.driver),
};

/// Bảng đặc tả: chỉ liệt kê các cặp hợp lệ. Mọi cặp còn lại phải ném StateError.
const Map<(TripStatus, String), TripStatus> _valid = {
  (TripStatus.requested, 'searchStarted'): TripStatus.searching,
  (TripStatus.searching, 'driverAccepted'): TripStatus.accepted,
  (TripStatus.accepted, 'driverArrived'): TripStatus.arriving,
  (TripStatus.arriving, 'pickedUp'): TripStatus.pickedUp,
  (TripStatus.pickedUp, 'tripStarted'): TripStatus.inTransit,
  (TripStatus.inTransit, 'completed'): TripStatus.completed,
  (TripStatus.searching, 'cancelledByCustomer'): TripStatus.cancelled,
  (TripStatus.accepted, 'cancelledByCustomer'): TripStatus.cancelled,
  (TripStatus.arriving, 'cancelledByCustomer'): TripStatus.cancelled,
  (TripStatus.searching, 'cancelledByDriver'): TripStatus.cancelled,
  (TripStatus.accepted, 'cancelledByDriver'): TripStatus.cancelled,
  (TripStatus.arriving, 'cancelledByDriver'): TripStatus.cancelled,
};

void main() {
  group('bảng trạng thái x sự kiện', () {
    test('bảng phủ đủ 8 trạng thái x 8 sự kiện mẫu, 12 cặp hợp lệ', () {
      expect(TripStatus.values, hasLength(8));
      expect(_events, hasLength(8));
      expect(_valid, hasLength(12));
    });

    test('canCancel đúng ba trạng thái và khớp bảng huỷ', () {
      final cancellable = TripStatus.values.where((s) => s.canCancel).toSet();
      expect(cancellable, {
        TripStatus.searching,
        TripStatus.accepted,
        TripStatus.arriving,
      });
      for (final status in TripStatus.values) {
        expect(
          _valid.containsKey((status, 'cancelledByCustomer')),
          status.canCancel,
          reason: status.name,
        );
      }
    });

    for (final status in TripStatus.values) {
      for (final entry in _events.entries) {
        final expected = _valid[(status, entry.key)];
        if (expected != null) {
          test('${status.name} + ${entry.key} -> ${expected.name}', () {
            expect(_tripAt(status).apply(entry.value).status, expected);
          });
        } else {
          test('${status.name} + ${entry.key} ném StateError', () {
            expect(() => _tripAt(status).apply(entry.value), throwsStateError);
          });
        }
      }
    }

    test('trạng thái cuối không nhận sự kiện nào', () {
      for (final status in TripStatus.values.where((s) => s.isTerminal)) {
        for (final event in _events.values) {
          expect(() => _tripAt(status).apply(event), throwsStateError);
        }
      }
    });

    test('thông báo StateError nêu tên sự kiện, id và trạng thái', () {
      expect(
        () => _tripAt(TripStatus.completed).apply(const PickedUp()),
        throwsA(
          isA<StateError>().having(
            (e) => e.message,
            'message',
            allOf(
              contains('PickedUp'),
              contains('TN-1'),
              contains('completed'),
            ),
          ),
        ),
      );
    });

    test('driverId rỗng khi searching là ArgumentError', () {
      expect(
        () => _tripAt(TripStatus.searching).apply(const DriverAccepted('  ')),
        throwsArgumentError,
      );
    });
  });

  group('Trip.request', () {
    late Trip trip;

    setUp(() {
      trip = Trip.request(
        id: 'TN-2',
        serviceKind: ServiceKind.food,
        distanceKm: 5,
        requestedAt: _requestedAt,
      );
    });

    test('bắt đầu ở requested, chưa có tài xế, cước 25.500 VND', () {
      expect(trip.status, TripStatus.requested);
      expect(trip.driverId, isNull);
      expect(trip.fare, const Money(25500));
      expect(trip.fare.toString(), '25.500 VND');
    });

    test('khoảng cách ngoài miền ném ArgumentError', () {
      expect(
        () => Trip.request(
          id: 'TN-3',
          serviceKind: ServiceKind.ride,
          distanceKm: 100.5,
          requestedAt: _requestedAt,
        ),
        throwsArgumentError,
      );
    });

    test('đi hết đường hạnh phúc qua 6 sự kiện', () {
      final seen = <TripStatus>[trip.status];
      for (final event in const <TripEvent>[
        SearchStarted(),
        DriverAccepted('D-23'),
        DriverArrived(),
        PickedUp(),
        TripStarted(),
        Completed(),
      ]) {
        trip = trip.apply(event);
        seen.add(trip.status);
      }
      expect(seen, [
        TripStatus.requested,
        TripStatus.searching,
        TripStatus.accepted,
        TripStatus.arriving,
        TripStatus.pickedUp,
        TripStatus.inTransit,
        TripStatus.completed,
      ]);
      expect(trip.driverId, 'D-23');
      expect(trip.status.isTerminal, isTrue);
    });

    test('huỷ giữ nguyên driverId đã có', () {
      final cancelled = _tripAt(TripStatus.arriving)
          .apply(const Cancelled(by: CancelledBy.driver));
      expect(cancelled.status, TripStatus.cancelled);
      expect(cancelled.driverId, 'D-17');
    });
  });

  group('bất biến', () {
    test('bản gốc không bị sửa sau apply', () {
      final original = _tripAt(TripStatus.requested);
      final next = original.apply(const SearchStarted());
      expect(original.status, TripStatus.requested);
      expect(next.status, TripStatus.searching);
      expect(identical(original, next), isFalse);
    });

    test('apply bị từ chối cũng không làm đổi bản gốc', () {
      final original = _tripAt(TripStatus.completed);
      expect(() => original.apply(const Completed()), throwsStateError);
      expect(original.status, TripStatus.completed);
    });

    test('apply giữ các trường còn lại', () {
      final original = _tripAt(TripStatus.searching);
      final next = original.apply(const DriverAccepted('D-99'));
      expect(next.id, original.id);
      expect(next.serviceKind, original.serviceKind);
      expect(next.distanceKm, original.distanceKm);
      expect(next.fare, original.fare);
      expect(next.requestedAt, original.requestedAt);
      expect(next.driverId, 'D-99');
      expect(original.driverId, isNull);
    });

    test('requestedAt không phải UTC bị assert chặn', () {
      expect(
        () => Trip(
          id: 'TN-4',
          serviceKind: ServiceKind.ride,
          status: TripStatus.requested,
          distanceKm: 5,
          fare: const Money(25500),
          requestedAt: DateTime(2026, 11, 6, 8, 30),
        ),
        throwsA(isA<AssertionError>()),
      );
    });
  });

  group('== và hashCode', () {
    test('mọi trường giống nhau thì bằng nhau', () {
      final a = _tripAt(TripStatus.searching);
      final b = _tripAt(TripStatus.searching);
      expect(identical(a, b), isFalse);
      expect(a, equals(b));
      expect(a.hashCode, b.hashCode);
    });

    test('khác trạng thái thì khác nhau', () {
      expect(
        _tripAt(TripStatus.searching),
        isNot(equals(_tripAt(TripStatus.accepted))),
      );
    });

    test('khác driverId thì khác nhau', () {
      final a = _tripAt(TripStatus.accepted);
      final b = Trip(
        id: a.id,
        serviceKind: a.serviceKind,
        status: a.status,
        distanceKm: a.distanceKm,
        fare: a.fare,
        driverId: 'D-99',
        requestedAt: a.requestedAt,
      );
      expect(a, isNot(equals(b)));
    });

    test('khác cước thì khác nhau', () {
      final a = _tripAt(TripStatus.requested);
      final b = Trip(
        id: a.id,
        serviceKind: a.serviceKind,
        status: a.status,
        distanceKm: a.distanceKm,
        fare: const Money(25501),
        requestedAt: a.requestedAt,
      );
      expect(a, isNot(equals(b)));
    });

    test('Set<Trip> không gộp nhầm hai chuyến khác tài xế', () {
      final byD17 = _tripAt(TripStatus.accepted);
      final byD99 = Trip(
        id: byD17.id,
        serviceKind: byD17.serviceKind,
        status: byD17.status,
        distanceKm: byD17.distanceKm,
        fare: byD17.fare,
        driverId: 'D-99',
        requestedAt: byD17.requestedAt,
      );
      expect({byD17, byD99}, hasLength(2));
    });
  });

  group('isSameTrip', () {
    test('cùng id là cùng một chuyến dù khác trạng thái', () {
      final before = _tripAt(TripStatus.requested);
      final after = before.apply(const SearchStarted());
      expect(before == after, isFalse);
      expect(before.isSameTrip(after), isTrue);
    });

    test('khác id là chuyến khác', () {
      final other = Trip.request(
        id: 'TN-OTHER',
        serviceKind: ServiceKind.ride,
        distanceKm: 5,
        requestedAt: _requestedAt,
      );
      expect(_tripAt(TripStatus.requested).isSameTrip(other), isFalse);
    });
  });

  group('bất biến status và driverId', () {
    Trip build(TripStatus status, String? driverId) => Trip(
      id: 'TN-5',
      serviceKind: ServiceKind.ride,
      status: status,
      distanceKm: 5,
      fare: const Money(25500),
      driverId: driverId,
      requestedAt: _requestedAt,
    );

    test('requested và searching phải chưa có tài xế', () {
      for (final status in [TripStatus.requested, TripStatus.searching]) {
        expect(build(status, null).driverId, isNull);
        expect(() => build(status, 'D-1'), throwsArgumentError);
      }
    });

    test('từ accepted tới completed phải có driverId không rỗng', () {
      const withDriver = [
        TripStatus.accepted,
        TripStatus.arriving,
        TripStatus.pickedUp,
        TripStatus.inTransit,
        TripStatus.completed,
      ];
      for (final status in withDriver) {
        expect(build(status, 'D-1').driverId, 'D-1');
        expect(() => build(status, null), throwsArgumentError);
        expect(() => build(status, '  '), throwsArgumentError);
      }
    });

    test('cancelled chấp nhận cả có lẫn chưa có tài xế', () {
      expect(build(TripStatus.cancelled, null).driverId, isNull);
      expect(build(TripStatus.cancelled, 'D-1').driverId, 'D-1');
    });

    test('huỷ ở searching rồi huỷ ở arriving đều tạo được Trip hợp lệ', () {
      final early = _tripAt(TripStatus.searching)
          .apply(const Cancelled(by: CancelledBy.customer));
      final late = _tripAt(TripStatus.arriving)
          .apply(const Cancelled(by: CancelledBy.customer));
      expect(early.driverId, isNull);
      expect(late.driverId, 'D-17');
    });
  });

  group('nhãn', () {
    test('mọi trạng thái và dịch vụ có nhãn không rỗng', () {
      for (final status in TripStatus.values) {
        expect(status.label, isNotEmpty);
      }
      for (final kind in ServiceKind.values) {
        expect(kind.label, isNotEmpty);
      }
      expect(TripStatus.searching.label, 'Đang tìm tài xế');
      expect(ServiceKind.parcel.label, 'Giao bưu kiện');
    });
  });
}
