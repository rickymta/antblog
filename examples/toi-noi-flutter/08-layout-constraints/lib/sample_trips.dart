import 'package:toi_noi_core/toi_noi_core.dart';

/// Địa chỉ ngắn và địa chỉ dài (hư cấu) dùng cho demo và test.
const shortAddress = 'Số 12 đường Lá Cọ, Quận 1';
const longAddress =
    'Căn hộ B-1807, Chung cư Hoa Sen Xanh, Số 235/12/4 đường Nguyễn Văn Linh '
    'Nối Dài, Phường Tân Phong, Quận 7, Thành phố Hồ Chí Minh';

/// Dựng chuyến đang ở [status] đúng như app sẽ làm: `Trip.request` rồi `apply`
/// từng sự kiện. Cước tính bằng `calculateFare`, không gõ tay.
Trip tripAt(
  TripStatus status, {
  String id = 'T-1042',
  ServiceKind serviceKind = ServiceKind.ride,
  double distanceKm = 5,
}) {
  var trip = Trip.request(
    id: id,
    serviceKind: serviceKind,
    distanceKm: distanceKm,
    requestedAt: DateTime.utc(2026, 11, 6, 2, 30),
  );
  for (final event in _eventsTo(status)) {
    trip = trip.apply(event);
  }
  return trip;
}

List<TripEvent> _eventsTo(TripStatus status) => switch (status) {
  TripStatus.requested => const [],
  TripStatus.searching => const [SearchStarted()],
  TripStatus.accepted => const [SearchStarted(), DriverAccepted('D-17')],
  TripStatus.arriving => const [
    SearchStarted(),
    DriverAccepted('D-17'),
    DriverArrived(),
  ],
  TripStatus.pickedUp => const [
    SearchStarted(),
    DriverAccepted('D-17'),
    DriverArrived(),
    PickedUp(),
  ],
  TripStatus.inTransit => const [
    SearchStarted(),
    DriverAccepted('D-17'),
    DriverArrived(),
    PickedUp(),
    TripStarted(),
  ],
  TripStatus.completed => const [
    SearchStarted(),
    DriverAccepted('D-17'),
    DriverArrived(),
    PickedUp(),
    TripStarted(),
    Completed(),
  ],
  TripStatus.cancelled => const [
    SearchStarted(),
    Cancelled(by: CancelledBy.customer),
  ],
};
