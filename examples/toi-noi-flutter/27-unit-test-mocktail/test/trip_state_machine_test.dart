import 'package:test/test.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support/transition_table.dart';

/// 8 trạng thái nhân 7 sự kiện là 56 cặp. Viết tay từng ca thì sẽ quên cặp
/// nào đó; vòng lặp sinh đủ 56 test có tên, bảng đúng là nguồn duy nhất.
void main() {
  group('bảng chuyển trạng thái', () {
    for (final status in TripStatus.values) {
      for (final event in events) {
        final name = '(${status.name}, ${eventName(event)})';
        final expected = validTransitions[(status, eventName(event))];
        if (expected != null) {
          test('$name sang ${expected.name}', () {
            final before = tripIn(status);
            final after = before.apply(event);
            expect(after.status, expected);
            expect(after.id, before.id);
            expect(after.fare, before.fare);
            expect(before.status, status, reason: 'chuyến cũ không bị sửa');
          });
        } else {
          test('$name ném StateError', () {
            expect(() => tripIn(status).apply(event), throwsStateError);
          });
        }
      }
    }
  });

  group('bảng đủ và đúng cỡ', () {
    test('8 trạng thái x 7 sự kiện = 56 cặp, 9 hợp lệ, 47 bị từ chối', () {
      final pairs = [
        for (final s in TripStatus.values)
          for (final e in events) (s, eventName(e)),
      ];
      expect(pairs, hasLength(56));
      expect(pairs.where(validTransitions.containsKey), hasLength(9));
      expect(
        pairs.where((p) => !validTransitions.containsKey(p)),
        hasLength(47),
      );
    });

    test('mọi cặp hợp lệ trong bảng đều nằm trong lưới 8 x 7', () {
      final names = events.map(eventName).toSet();
      for (final key in validTransitions.keys) {
        expect(TripStatus.values, contains(key.$1));
        expect(names, contains(key.$2));
      }
    });

    test('huỷ ở ba trạng thái khớp TripStatus.canCancel', () {
      final cancellable = {
        for (final s in TripStatus.values)
          if (validTransitions.containsKey((s, 'Cancelled'))) s,
      };
      expect(cancellable, {
        for (final s in TripStatus.values)
          if (s.canCancel) s,
      });
    });
  });

  group('ngoài bảng: điều bảng không nói', () {
    test('tài xế nhận chuyến với mã rỗng bị từ chối bằng ArgumentError', () {
      final searching = tripIn(TripStatus.searching);
      expect(
        () => searching.apply(const DriverAccepted('  ')),
        throwsArgumentError,
      );
    });

    test('huỷ bởi khách hay bởi tài xế cùng cho cancelled', () {
      for (final by in CancelledBy.values) {
        final next = tripIn(TripStatus.accepted).apply(Cancelled(by: by));
        expect(next.status, TripStatus.cancelled);
      }
    });
  });

  group('bảng bắt được lỗi cài vào', () {
    test('apply thật không sai cặp nào', () {
      expect(transitionFailures((trip, event) => trip.apply(event)), isEmpty);
    });

    test('cho phép huỷ khi đã đón khách: đúng một cặp đỏ', () {
      Trip lenient(Trip trip, TripEvent event) {
        if (trip.status == TripStatus.pickedUp && event is Cancelled) {
          return tripIn(TripStatus.cancelled);
        }
        return trip.apply(event);
      }

      expect(transitionFailures(lenient), ['(pickedUp, Cancelled)']);
    });

    test('chặn nhầm huỷ khi tài xế đã nhận: đúng một cặp đỏ', () {
      Trip strict(Trip trip, TripEvent event) {
        if (trip.status == TripStatus.accepted && event is Cancelled) {
          throw StateError('không cho huỷ');
        }
        return trip.apply(event);
      }

      expect(transitionFailures(strict), ['(accepted, Cancelled)']);
    });
  });
}
