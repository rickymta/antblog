import 'package:test/test.dart';
import 'package:toi_noi_03/functions_tour.dart';
import 'package:toi_noi_03/quote.dart';
import 'package:toi_noi_03/service_kind.dart';
import 'package:toi_noi_03/trip.dart';
import 'package:toi_noi_03/trip_event.dart';
import 'package:toi_noi_03/trip_status.dart';

final _requestedAt = DateTime.utc(2026, 11, 6, 8, 30);

Trip _tripAt(TripStatus status) => Trip(
  id: 'TN-1',
  serviceKind: ServiceKind.ride,
  status: status,
  distanceKm: 5,
  fareVnd: 25500,
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
/// Khoá là record (trạng thái, tên sự kiện), so sánh theo cấu trúc.
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

    for (final status in TripStatus.values) {
      for (final entry in _events.entries) {
        final expected = _valid[(status, entry.key)];
        if (expected != null) {
          test('${status.name} + ${entry.key} -> ${expected.name}', () {
            final next = _tripAt(status).apply(entry.value);
            expect(next.status, expected);
          });
        } else {
          test('${status.name} + ${entry.key} ném StateError', () {
            expect(
              () => _tripAt(status).apply(entry.value),
              throwsA(isA<StateError>()),
            );
          });
        }
      }
    }

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

    test(
      'driverId rỗng khi searching là ArgumentError, không phải StateError',
      () {
        expect(
          () => _tripAt(TripStatus.searching).apply(const DriverAccepted('  ')),
          throwsA(isA<ArgumentError>()),
        );
      },
    );

    test('driverId rỗng ở trạng thái khác vẫn là StateError', () {
      expect(
        () => _tripAt(TripStatus.requested).apply(const DriverAccepted('')),
        throwsA(isA<StateError>()),
      );
    });
  });

  group('luồng đầy đủ', () {
    test('đi hết 7 trạng thái theo đường hạnh phúc', () {
      var trip = Trip.request(
        id: 'TN-2',
        serviceKind: ServiceKind.food,
        distanceKm: 3,
        requestedAt: _requestedAt,
      );
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

    test('Trip.request tính cước từ quoteFor và bắt đầu ở requested', () {
      final trip = Trip.request(
        id: 'TN-3',
        serviceKind: ServiceKind.ride,
        distanceKm: 5,
        requestedAt: _requestedAt,
      );
      expect(trip.status, TripStatus.requested);
      expect(trip.fareVnd, 25500);
      expect(trip.driverId, isNull);
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
      expect(next.fareVnd, original.fareVnd);
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
          distanceKm: 1,
          fareVnd: 12000,
          requestedAt: DateTime(2026, 11, 6, 8, 30),
        ),
        throwsA(isA<AssertionError>()),
      );
    });

    test(
      'const constructor: hai sự kiện const giống nhau là một đối tượng',
      () {
        expect(identical(const PickedUp(), const PickedUp()), isTrue);
        expect(
          identical(
            const Cancelled(by: CancelledBy.customer),
            const Cancelled(by: CancelledBy.customer),
          ),
          isTrue,
        );
      },
    );
  });

  group('==, hashCode và toString', () {
    test('cùng id và status thì bằng nhau, dù khác driverId', () {
      final a = _tripAt(TripStatus.accepted);
      final b = Trip(
        id: 'TN-1',
        serviceKind: ServiceKind.ride,
        status: TripStatus.accepted,
        distanceKm: 5,
        fareVnd: 25500,
        driverId: 'D-99',
        requestedAt: _requestedAt,
      );
      expect(a, b);
      expect(a.hashCode, b.hashCode);
    });

    test('khác status hoặc khác id thì không bằng nhau', () {
      expect(_tripAt(TripStatus.accepted), isNot(_tripAt(TripStatus.arriving)));
      final other = Trip(
        id: 'TN-2',
        serviceKind: ServiceKind.ride,
        status: TripStatus.accepted,
        distanceKm: 5,
        fareVnd: 25500,
        driverId: 'D-17',
        requestedAt: _requestedAt,
      );
      expect(_tripAt(TripStatus.accepted), isNot(other));
    });

    test('Set coi hai bản chụp cùng id và status là một', () {
      final set = {
        _tripAt(TripStatus.searching),
        _tripAt(TripStatus.searching),
        _tripAt(TripStatus.accepted),
      };
      expect(set, hasLength(2));
    });

    test('toString có id, dịch vụ, trạng thái và cước', () {
      final text = _tripAt(TripStatus.accepted).toString();
      expect(text, contains('TN-1'));
      expect(text, contains('ride'));
      expect(text, contains('accepted'));
      expect(text, contains('25500'));
      expect(text, contains('D-17'));
    });
  });

  group('enum', () {
    test('canCancel chỉ đúng với searching, accepted, arriving', () {
      const cancellable = {
        TripStatus.searching,
        TripStatus.accepted,
        TripStatus.arriving,
      };
      for (final status in TripStatus.values) {
        expect(
          status.canCancel,
          cancellable.contains(status),
          reason: status.name,
        );
      }
    });

    test('isTerminal chỉ đúng với completed và cancelled', () {
      final terminal = TripStatus.values.where((s) => s.isTerminal).toSet();
      expect(terminal, {TripStatus.completed, TripStatus.cancelled});
    });

    test(
      'canCancel khớp với bảng: có Cancelled hợp lệ khi và chỉ khi canCancel',
      () {
        for (final status in TripStatus.values) {
          final accepts = _valid.containsKey((status, 'cancelledByCustomer'));
          expect(status.canCancel, accepts, reason: status.name);
        }
      },
    );

    test('trạng thái cuối không nhận sự kiện nào', () {
      for (final status in TripStatus.values.where((s) => s.isTerminal)) {
        for (final event in _events.values) {
          expect(
            () => _tripAt(status).apply(event),
            throwsStateError,
            reason: '${status.name} + ${event.runtimeType}',
          );
        }
      }
    });

    test('name và thứ tự values ổn định, label là tiếng Việt', () {
      expect(TripStatus.values.map((s) => s.name), [
        'requested',
        'searching',
        'accepted',
        'arriving',
        'pickedUp',
        'inTransit',
        'completed',
        'cancelled',
      ]);
      expect(TripStatus.cancelled.label, 'Đã huỷ');
      expect(ServiceKind.parcel.label, 'Giao bưu kiện');
      expect(ServiceKind.values, hasLength(3));
    });

    test('describeEvent phủ mọi sự kiện mẫu', () {
      for (final event in _events.values) {
        expect(describeEvent(event), isNotEmpty);
      }
      expect(describeEvent(const DriverAccepted('D-17')), contains('D-17'));
      expect(
        describeEvent(const Cancelled(by: CancelledBy.driver)),
        'Tài xế huỷ chuyến',
      );
    });
  });

  group('record', () {
    test('quoteFor trả record đặt tên, destructuring theo tên trường', () {
      final (:fareVnd, :distanceKm) = quoteFor(5);
      expect(fareVnd, 25500);
      expect(distanceKm, 5);
    });

    test('truy cập trường bằng .fareVnd', () {
      expect(quoteFor(2).fareVnd, 12000);
    });

    test('record so sánh bằng nhau theo cấu trúc', () {
      expect(quoteFor(5), quoteFor(5));
      expect(quoteFor(5) == quoteFor(3), isFalse);
      expect(quoteFor(5).hashCode, quoteFor(5).hashCode);
    });

    test('ca biên của bảng giá', () {
      expect(quoteFor(0).fareVnd, 12000);
      expect(quoteFor(1.999).fareVnd, 12000);
      expect(quoteFor(2).fareVnd, 12000);
      expect(quoteFor(2.001).fareVnd, 12005);
      expect(quoteFor(2.0004).fareVnd, 12000);
      expect(quoteFor(100).fareVnd, 453000);
    });

    test('ngoài miền 0 đến 100 km ném ArgumentError', () {
      expect(() => quoteFor(-0.001), throwsArgumentError);
      expect(() => quoteFor(100.001), throwsArgumentError);
    });

    test('record làm khoá và destructuring trong pattern', () {
      const pair = (TripStatus.searching, 'driverAccepted');
      final (status, name) = pair;
      expect(status, TripStatus.searching);
      expect(name, 'driverAccepted');
      expect(_valid[pair], TripStatus.accepted);
    });
  });

  group('hàm', () {
    test('formatVnd chèn dấu chấm ngăn cách hàng nghìn', () {
      expect(formatVnd(0), '0 VND');
      expect(formatVnd(999), '999 VND');
      expect(formatVnd(12000), '12.000 VND');
      expect(formatVnd(453000), '453.000 VND');
      expect(formatVnd(1234567, 'đ'), '1.234.567 đ');
      expect(formatVnd(-4500), '-4.500 VND');
    });

    test('tripSummary: tham số đặt tên, có mặc định và có thể thiếu', () {
      expect(
        tripSummary('TN-1', kind: ServiceKind.ride, fareVnd: 25500),
        'TN-1 | Đặt xe | 25.500 VND',
      );
      expect(
        tripSummary('TN-1', kind: ServiceKind.parcel),
        'TN-1 | Giao bưu kiện | chưa có giá',
      );
      expect(
        tripSummary('TN-1', fareVnd: 12000, unit: 'đ', kind: ServiceKind.food),
        'TN-1 | Giao đồ ăn | 12.000 đ',
      );
    });

    test('isLongTrip là hàm mũi tên, biên tại 10 km', () {
      expect(isLongTrip(10), isFalse);
      expect(isLongTrip(10.1), isTrue);
    });

    test('hàm là giá trị: truyền lambda và tear-off cho countWhere', () {
      final trips = [
        _tripAt(TripStatus.searching),
        _tripAt(TripStatus.inTransit),
        _tripAt(TripStatus.cancelled),
      ];
      expect(countWhere(trips, isCancellable), 1);
      expect(countWhere(trips, (t) => t.status.isTerminal), 1);
      expect(countWhere(trips, (t) => isLongTrip(t.distanceKm)), 0);
      bool hasDefaultId(Trip t) => t.id == 'TN-1';
      final TripFilter filter = hasDefaultId;
      expect(countWhere(trips, filter), 3);
    });
  });
}
