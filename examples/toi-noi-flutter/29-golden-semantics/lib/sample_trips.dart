import 'package:toi_noi_core/toi_noi_core.dart';

/// Dựng một chuyến ở trạng thái [status] bằng `Trip.request` rồi `apply` từng
/// sự kiện, không tạo trạng thái bằng tay. Dữ liệu hư cấu.
Trip tripAt(
  TripStatus status, {
  required String id,
  ServiceKind kind = ServiceKind.ride,
  double km = 5,
}) {
  var trip = Trip.request(
    id: id,
    serviceKind: kind,
    distanceKm: km,
    requestedAt: DateTime.utc(2026, 11, 6, 8, 30),
  );
  if (status == TripStatus.cancelled) {
    return trip
        .apply(const SearchStarted())
        .apply(const Cancelled(by: CancelledBy.customer));
  }
  const path = <TripStatus, TripEvent>{
    TripStatus.searching: SearchStarted(),
    TripStatus.accepted: DriverAccepted('D-07'),
    TripStatus.arriving: DriverArrived(),
    TripStatus.pickedUp: PickedUp(),
    TripStatus.inTransit: TripStarted(),
    TripStatus.completed: Completed(),
  };
  for (final entry in path.entries) {
    if (trip.status == status) break;
    trip = trip.apply(entry.value);
  }
  return trip;
}
