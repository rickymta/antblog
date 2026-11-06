import 'package:toi_noi_core/toi_noi_core.dart';

/// Dựng một chuyến ở trạng thái [status] bằng `Trip.request` rồi `apply` từng
/// sự kiện, không tạo trạng thái bằng tay. Dữ liệu hư cấu.
Trip tripAt(
  TripStatus status, {
  required String id,
  ServiceKind kind = ServiceKind.ride,
  double km = 5,
  DateTime? at,
}) {
  var trip = Trip.request(
    id: id,
    serviceKind: kind,
    distanceKm: km,
    requestedAt: at ?? DateTime.utc(2026, 11, 6, 8, 30),
  );
  const path = <TripStatus, TripEvent>{
    TripStatus.searching: SearchStarted(),
    TripStatus.accepted: DriverAccepted('D-07'),
    TripStatus.arriving: DriverArrived(),
    TripStatus.pickedUp: PickedUp(),
    TripStatus.inTransit: TripStarted(),
    TripStatus.completed: Completed(),
  };
  if (status == TripStatus.cancelled) {
    return trip
        .apply(const SearchStarted())
        .apply(const Cancelled(by: CancelledBy.customer));
  }
  for (final entry in path.entries) {
    if (trip.status == status) break;
    trip = trip.apply(entry.value);
  }
  return trip;
}

/// Bốn chuyến mẫu cho app: đang tìm, đang tới, hoàn thành, đã huỷ.
List<Trip> sampleTrips() => [
  tripAt(TripStatus.searching, id: 'T-1045', km: 5),
  tripAt(TripStatus.arriving, id: 'T-1044', kind: ServiceKind.food, km: 2.5),
  tripAt(TripStatus.completed, id: 'T-1043', kind: ServiceKind.parcel, km: 12),
  tripAt(TripStatus.cancelled, id: 'T-1042', km: 3),
];
