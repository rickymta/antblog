import 'fare.dart';
import 'money.dart';
import 'service_kind.dart';
import 'trip_event.dart';
import 'trip_status.dart';

/// Một chuyến của Tới Nơi. Bất biến: mọi thay đổi trả về bản sao mới.
final class Trip {
  Trip({
    required this.id,
    required this.serviceKind,
    required this.status,
    required this.distanceKm,
    required this.fare,
    this.driverId,
    required this.requestedAt,
  }) : assert(requestedAt.isUtc, 'requestedAt phải là giờ UTC') {
    _checkDriver();
  }

  /// Tạo chuyến mới ở trạng thái `requested`, tính cước bằng [calculateFare].
  ///
  /// Ném [ArgumentError] khi [distanceKm] ngoài miền 0..100.
  Trip.request({
    required String id,
    required ServiceKind serviceKind,
    required double distanceKm,
    required DateTime requestedAt,
  }) : this(
         id: id,
         serviceKind: serviceKind,
         status: TripStatus.requested,
         distanceKm: distanceKm,
         fare: calculateFare(distanceKm),
         requestedAt: requestedAt,
       );

  final String id;
  final ServiceKind serviceKind;
  final TripStatus status;
  final double distanceKm;
  final Money fare;
  final String? driverId;

  /// Thời điểm yêu cầu, luôn là UTC.
  final DateTime requestedAt;

  /// Giữ bất biến giữa trạng thái và tài xế; `switch` vét cạn trên [status].
  void _checkDriver() {
    final driver = driverId;
    switch (status) {
      case TripStatus.requested || TripStatus.searching:
        if (driver != null) {
          throw ArgumentError.value(
            driver,
            'driverId',
            'phải null khi chuyến ở ${status.name}',
          );
        }
      case TripStatus.accepted ||
          TripStatus.arriving ||
          TripStatus.pickedUp ||
          TripStatus.inTransit ||
          TripStatus.completed:
        if (driver == null || driver.trim().isEmpty) {
          throw ArgumentError.value(
            driver,
            'driverId',
            'phải có tài xế khi chuyến ở ${status.name}',
          );
        }
      case TripStatus.cancelled:
        break; // huỷ ở searching thì chưa có tài xế, huỷ sau đó thì có
    }
  }

  /// Áp dụng [event] và trả về chuyến mới; chuyến hiện tại không bị sửa.
  ///
  /// Ném [StateError] khi cặp (trạng thái, sự kiện) không hợp lệ, và
  /// [ArgumentError] khi tài xế nhận chuyến có `driverId` rỗng.
  Trip apply(TripEvent event) => switch ((status, event)) {
    (TripStatus.requested, SearchStarted()) => _next(TripStatus.searching),
    (TripStatus.searching, DriverAccepted(:final driverId))
        when driverId.trim().isEmpty =>
      throw ArgumentError.value(driverId, 'driverId', 'không được để trống'),
    (TripStatus.searching, DriverAccepted(:final driverId)) => _next(
      TripStatus.accepted,
      driverId: driverId,
    ),
    (TripStatus.accepted, DriverArrived()) => _next(TripStatus.arriving),
    (TripStatus.arriving, PickedUp()) => _next(TripStatus.pickedUp),
    (TripStatus.pickedUp, TripStarted()) => _next(TripStatus.inTransit),
    (TripStatus.inTransit, Completed()) => _next(TripStatus.completed),
    (
      TripStatus.searching || TripStatus.accepted || TripStatus.arriving,
      Cancelled(),
    ) =>
      _next(TripStatus.cancelled),
    _ => throw StateError(
      'Không thể áp dụng ${event.runtimeType} khi chuyến $id ở '
      'trạng thái ${status.name}',
    ),
  };

  Trip _next(TripStatus nextStatus, {String? driverId}) => Trip(
    id: id,
    serviceKind: serviceKind,
    status: nextStatus,
    distanceKm: distanceKm,
    fare: fare,
    driverId: driverId ?? this.driverId,
    requestedAt: requestedAt,
  );

  /// Bằng nhau khi mọi trường bằng nhau (so sánh giá trị, không so tham chiếu).
  ///
  /// Hai bản chụp của cùng một chuyến ở hai trạng thái là hai giá trị khác
  /// nhau; muốn hỏi "có phải cùng một chuyến" thì dùng [isSameTrip].
  @override
  bool operator ==(Object other) =>
      other is Trip &&
      other.id == id &&
      other.serviceKind == serviceKind &&
      other.status == status &&
      other.distanceKm == distanceKm &&
      other.fare == fare &&
      other.driverId == driverId &&
      other.requestedAt == requestedAt;

  @override
  int get hashCode => Object.hash(
    id,
    serviceKind,
    status,
    distanceKm,
    fare,
    driverId,
    requestedAt,
  );

  /// Cùng một chuyến (cùng [id]) bất kể đang ở trạng thái nào.
  bool isSameTrip(Trip other) => other.id == id;

  @override
  String toString() =>
      'Trip($id, ${serviceKind.name}, ${status.name}, '
      '$distanceKm km, $fare, driver: ${driverId ?? '-'})';
}
