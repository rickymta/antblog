/// Mô hình rút gọn của Tới Nơi, dùng lại tên trong README quy ước.
library;

enum ServiceKind { ride, food, parcel }

enum TripStatus {
  requested,
  searching,
  accepted,
  arriving,
  pickedUp,
  inTransit,
  completed,
  cancelled;

  /// Trạng thái kết thúc: chuyến không đi tiếp được nữa.
  bool get isTerminal => switch (this) {
    completed || cancelled => true,
    requested ||
    searching ||
    accepted ||
    arriving ||
    pickedUp ||
    inTransit => false,
  };

  /// Huỷ được ở searching, accepted, arriving.
  bool get canCancel => switch (this) {
    searching || accepted || arriving => true,
    requested || pickedUp || inTransit || completed || cancelled => false,
  };
}

/// Chuyến bất biến: mọi field là final, đổi trạng thái thì trả bản sao mới.
class Trip {
  const Trip({
    required this.id,
    required this.serviceKind,
    required this.status,
    required this.distanceKm,
    required this.fareVnd,
    required this.requestedAt,
    this.driverId,
  });

  final String id;
  final ServiceKind serviceKind;
  final TripStatus status;
  final double distanceKm;
  final int fareVnd;
  final String? driverId;

  /// Thời điểm yêu cầu, luôn là UTC.
  final DateTime requestedAt;

  Trip withStatus(TripStatus next) => Trip(
    id: id,
    serviceKind: serviceKind,
    status: next,
    distanceKm: distanceKm,
    fareVnd: fareVnd,
    driverId: driverId,
    requestedAt: requestedAt,
  );

  @override
  String toString() => 'Trip($id, ${status.name}, $fareVnd VND)';
}

class Driver {
  const Driver({
    required this.id,
    required this.name,
    required this.city,
    required this.isOnline,
  });

  final String id;
  final String name;
  final String city;
  final bool isOnline;

  @override
  String toString() => 'Driver($id, $name, $city, online: $isOnline)';
}
