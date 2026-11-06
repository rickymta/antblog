import 'package:toi_noi_03/functions_tour.dart';
import 'package:toi_noi_03/service_kind.dart';
import 'package:toi_noi_03/trip.dart';
import 'package:toi_noi_03/trip_event.dart';

void main() {
  print('--- Chuyến hoàn thành ---');
  var trip = Trip.request(
    id: 'TN-1001',
    serviceKind: ServiceKind.ride,
    distanceKm: 5,
    requestedAt: DateTime.utc(2026, 11, 6, 8, 30),
  );
  print('${trip.status.label}: $trip');

  const events = <TripEvent>[
    SearchStarted(),
    DriverAccepted('D-17'),
    DriverArrived(),
    PickedUp(),
    TripStarted(),
    Completed(),
  ];
  for (final event in events) {
    trip = trip.apply(event);
    print(
      '${describeEvent(event)} -> ${trip.status.label} (${trip.status.name})',
    );
  }
  print('Kết thúc: ${trip.status.isTerminal}');

  print('--- Chuyến bị huỷ khi tài xế đang tới ---');
  var parcel = Trip.request(
    id: 'TN-1002',
    serviceKind: ServiceKind.parcel,
    distanceKm: 3.2,
    requestedAt: DateTime.utc(2026, 11, 6, 9),
  );
  for (final event in const <TripEvent>[
    SearchStarted(),
    DriverAccepted('D-23'),
    DriverArrived(),
  ]) {
    parcel = parcel.apply(event);
  }
  print('${parcel.status.label}, huỷ được: ${parcel.status.canCancel}');
  parcel = parcel.apply(const Cancelled(by: CancelledBy.customer));
  print(
    '${describeEvent(const Cancelled(by: CancelledBy.customer))} -> '
    '${parcel.status.label}',
  );

  try {
    parcel.apply(const PickedUp());
  } on StateError catch (e) {
    print('Từ chối: ${e.message}');
  }
  print('--- Hàm ---');
  print(tripSummary('TN-1003', kind: ServiceKind.food, fareVnd: 25500));
  print(tripSummary('TN-1004', kind: ServiceKind.ride));
  print(formatVnd(453000));
  final waiting = Trip.request(
    id: 'TN-1005',
    serviceKind: ServiceKind.ride,
    distanceKm: 12.5,
    requestedAt: DateTime.utc(2026, 11, 6, 9, 30),
  ).apply(const SearchStarted());
  final trips = [trip, parcel, waiting];
  print('Huỷ được: ${countWhere(trips, isCancellable)}');
  print('Chuyến dài: ${countWhere(trips, (t) => isLongTrip(t.distanceKm))}');
}
