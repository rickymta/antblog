import 'package:toi_noi_core/toi_noi_core.dart';

import 'doubles.dart';

typedef ApplyFn = Trip Function(Trip trip, TripEvent event);

/// Bảy sự kiện của chuyến, mỗi biến thể một đại diện.
const events = <TripEvent>[
  SearchStarted(),
  DriverAccepted('D-1'),
  DriverArrived(),
  PickedUp(),
  TripStarted(),
  Completed(),
  Cancelled(by: CancelledBy.customer),
];

/// Tên ngắn của sự kiện. `switch` vét cạn trên `sealed class`: thêm biến thể
/// thứ tám mà quên cập nhật bảng thì file này không biên dịch.
String eventName(TripEvent event) => switch (event) {
  SearchStarted() => 'SearchStarted',
  DriverAccepted() => 'DriverAccepted',
  DriverArrived() => 'DriverArrived',
  PickedUp() => 'PickedUp',
  TripStarted() => 'TripStarted',
  Completed() => 'Completed',
  Cancelled() => 'Cancelled',
};

/// Chín cặp (trạng thái, sự kiện) hợp lệ, viết tay từ hồ sơ Tới Nơi: sáu mũi
/// tên đường đi bình thường và huỷ ở ba trạng thái. Mọi cặp khác phải ném.
const validTransitions = <(TripStatus, String), TripStatus>{
  (TripStatus.requested, 'SearchStarted'): TripStatus.searching,
  (TripStatus.searching, 'DriverAccepted'): TripStatus.accepted,
  (TripStatus.accepted, 'DriverArrived'): TripStatus.arriving,
  (TripStatus.arriving, 'PickedUp'): TripStatus.pickedUp,
  (TripStatus.pickedUp, 'TripStarted'): TripStatus.inTransit,
  (TripStatus.inTransit, 'Completed'): TripStatus.completed,
  (TripStatus.searching, 'Cancelled'): TripStatus.cancelled,
  (TripStatus.accepted, 'Cancelled'): TripStatus.cancelled,
  (TripStatus.arriving, 'Cancelled'): TripStatus.cancelled,
};

const _withDriver = {
  TripStatus.accepted,
  TripStatus.arriving,
  TripStatus.pickedUp,
  TripStatus.inTransit,
  TripStatus.completed,
};

/// Một chuyến đang ở [status], dựng thẳng bằng constructor (không đi qua
/// `apply`, để test bảng không dựa vào chính thứ nó kiểm).
Trip tripIn(TripStatus status) => Trip(
  id: 'T-1',
  serviceKind: ServiceKind.ride,
  status: status,
  distanceKm: 5,
  fare: calculateFare(5),
  driverId: _withDriver.contains(status) ? 'D-1' : null,
  requestedAt: fixedNow,
);

/// Chạy mọi cặp qua [apply], trả tên các cặp mà [apply] làm sai.
List<String> transitionFailures(ApplyFn apply) {
  final failed = <String>[];
  for (final status in TripStatus.values) {
    for (final event in events) {
      final name = '(${status.name}, ${eventName(event)})';
      final expected = validTransitions[(status, eventName(event))];
      try {
        final next = apply(tripIn(status), event);
        if (next.status != expected) {
          failed.add(name);
        }
      } on StateError {
        if (expected != null) {
          failed.add(name);
        }
      }
    }
  }
  return failed;
}
