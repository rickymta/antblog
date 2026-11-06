import 'package:toi_noi_04/driver_lookup.dart';
import 'package:toi_noi_04/model.dart';
import 'package:toi_noi_04/paged.dart';
import 'package:toi_noi_04/trip_queries.dart';

void main() {
  final trips = sampleTrips();
  final drivers = sampleDrivers();

  print('Tổng cước ${trips.totalFareVnd} VND (số liệu minh hoạ)');
  print(
    'Chưa kết thúc: ${trips.active.length} chuyến, '
    '${trips.active.totalFareVnd} VND',
  );

  print('\nNhóm theo trạng thái:');
  for (final MapEntry(key: status, value: group) in trips.byStatus.entries) {
    print('  ${status.name}: ${group.map((trip) => trip.id).join(', ')}');
  }

  print('\nCước giảm dần:');
  for (final trip in trips.sortedByFareDesc) {
    print('  ${trip.id} ${trip.fareVnd} VND');
  }

  for (final pageNumber in [1, 2]) {
    final page = paginate(trips, page: pageNumber, pageSize: 3);
    print('\nTrang $pageNumber (hasNext: ${page.hasNext}):');
    for (final line
        in page.map((trip) => '${trip.id} ${trip.status.name}').items) {
      print('  $line');
    }
  }

  final byId = indexById(drivers);
  print('\nThành phố có tài xế online: ${onlineCities(drivers).join(', ')}');
  for (final trip in trips.active) {
    final driver = driverOf(trip, byId);
    print('  ${trip.id} -> ${driver?.name ?? 'chưa có tài xế'}');
  }
}

/// Tám chuyến hư cấu; cước theo bảng giá minh hoạ 12.000 + 4.500/km vượt 2 km.
List<Trip> sampleTrips() {
  final t0 = DateTime.utc(2026, 11, 6, 1);
  Trip trip(
    String id,
    ServiceKind kind,
    TripStatus status,
    double km,
    int fare,
    String? driverId,
    int minute,
  ) => Trip(
    id: id,
    serviceKind: kind,
    status: status,
    distanceKm: km,
    fareVnd: fare,
    driverId: driverId,
    requestedAt: t0.add(Duration(minutes: minute)),
  );
  return [
    trip('TN-1', ServiceKind.ride, TripStatus.completed, 5, 25500, 'D-17', 0),
    trip('TN-2', ServiceKind.food, TripStatus.inTransit, 3.2, 17400, 'D-23', 4),
    trip('TN-3', ServiceKind.parcel, TripStatus.accepted, 10, 48000, 'D-17', 9),
    trip('TN-4', ServiceKind.ride, TripStatus.cancelled, 1.5, 12000, null, 12),
    trip('TN-5', ServiceKind.ride, TripStatus.searching, 7.5, 36750, null, 15),
    trip('TN-6', ServiceKind.food, TripStatus.completed, 2, 12000, 'D-31', 20),
    trip('TN-7', ServiceKind.ride, TripStatus.arriving, 12, 57000, 'D-23', 25),
    trip('TN-8', ServiceKind.parcel, TripStatus.requested, 4, 21000, null, 30),
  ];
}

/// Năm tài xế hư cấu ở ba thành phố.
List<Driver> sampleDrivers() => const [
  Driver(id: 'D-17', name: 'Lê Minh An', city: 'Hà Nội', isOnline: true),
  Driver(id: 'D-23', name: 'Trần Bảo Bình', city: 'Đà Nẵng', isOnline: true),
  Driver(id: 'D-31', name: 'Phạm Chí Công', city: 'Hà Nội', isOnline: false),
  Driver(id: 'D-42', name: 'Võ Thu Dung', city: 'TP.HCM', isOnline: true),
  Driver(id: 'D-58', name: 'Đỗ Gia Em', city: 'Đà Nẵng', isOnline: true),
];
