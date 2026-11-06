import 'package:toi_noi_04/model.dart';

Trip makeTrip(
  String id,
  TripStatus status,
  int fareVnd, {
  String? driverId,
  ServiceKind kind = ServiceKind.ride,
}) => Trip(
  id: id,
  serviceKind: kind,
  status: status,
  distanceKm: 3,
  fareVnd: fareVnd,
  driverId: driverId,
  requestedAt: DateTime.utc(2026, 11, 6, 1),
);

/// Năm chuyến: hai completed, một inTransit, một searching, một cancelled.
List<Trip> fiveTrips() => [
  makeTrip('T1', TripStatus.completed, 25500, driverId: 'D-17'),
  makeTrip('T2', TripStatus.inTransit, 17400, driverId: 'D-23'),
  makeTrip('T3', TripStatus.searching, 36750),
  makeTrip('T4', TripStatus.cancelled, 12000),
  makeTrip('T5', TripStatus.completed, 12000, driverId: 'D-17'),
];
