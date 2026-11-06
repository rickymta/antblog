import 'package:toi_noi_06/report.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

void main() {
  print('--- Báo giá (số minh hoạ) ---');
  for (final km in [2.0, 2.001, 5.0, 100.0]) {
    print(quoteLine(km));
  }

  print('--- Một chuyến hoàn thành (6 sự kiện) ---');
  final trip = Trip.request(
    id: 'TN-1001',
    serviceKind: ServiceKind.ride,
    distanceKm: 5,
    requestedAt: DateTime.utc(2026, 11, 6, 8, 30),
  );
  const events = <TripEvent>[
    SearchStarted(),
    DriverAccepted('D-17'),
    DriverArrived(),
    PickedUp(),
    TripStarted(),
    Completed(),
  ];
  runTrip(trip, events).forEach(print);

  print('--- Chuyến khác bị huỷ khi tài xế đang tới (sự kiện thứ 7) ---');
  final cancelledTrip = Trip.request(
    id: 'TN-1003',
    serviceKind: ServiceKind.food,
    distanceKm: 2.5,
    requestedAt: DateTime.utc(2026, 11, 6, 8, 45),
  );
  runTrip(cancelledTrip, const [
    SearchStarted(),
    DriverAccepted('D-23'),
    DriverArrived(),
    Cancelled(by: CancelledBy.customer),
  ]).forEach(print);

  print('--- Tổng cước hai chuyến ---');
  final trips = [
    trip,
    Trip.request(
      id: 'TN-1002',
      serviceKind: ServiceKind.parcel,
      distanceKm: 3.2,
      requestedAt: DateTime.utc(2026, 11, 6, 9),
    ),
  ];
  print('${trips.totalFare} cho ${trips.length} chuyến');
}
