import 'package:collection/collection.dart' show IterableExtension;

import 'model.dart';

/// Map tra tài xế theo id. Trùng id thì bản sau ghi đè bản trước.
Map<String, Driver> indexById(Iterable<Driver> drivers) => {
  for (final driver in drivers) driver.id: driver,
};

/// Set thành phố có ít nhất một tài xế online. Trùng thành phố tự bị loại.
Set<String> onlineCities(Iterable<Driver> drivers) => {
  for (final driver in drivers)
    if (driver.isOnline) driver.city,
};

/// Tra tài xế theo id; id null hoặc không có trong map đều trả null.
Driver? findDriver(Map<String, Driver> byId, String? id) =>
    id == null ? null : byId[id];

/// Tài xế của một chuyến (Trip.driverId có thể null).
Driver? driverOf(Trip trip, Map<String, Driver> byId) =>
    findDriver(byId, trip.driverId);

/// any: có ít nhất một tài xế online trong thành phố?
bool hasOnlineDriverIn(Iterable<Driver> drivers, String city) =>
    drivers.any((driver) => driver.isOnline && driver.city == city);

/// firstWhereOrNull của package:collection: không có thì trả null.
Driver? firstOnlineIn(Iterable<Driver> drivers, String city) => drivers
    .firstWhereOrNull((driver) => driver.isOnline && driver.city == city);

/// firstWhere + orElse: không có thì dùng tài xế dự phòng thay vì StateError.
Driver firstOnlineOrFallback(
  Iterable<Driver> drivers,
  String city,
  Driver fallback,
) => drivers.firstWhere(
  (driver) => driver.isOnline && driver.city == city,
  orElse: () => fallback,
);
