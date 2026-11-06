import 'quote.dart';
import 'service_kind.dart';
import 'trip_event.dart';
import 'trip_status.dart';

/// Một chuyến của Tới Nơi. Bất biến: mọi thay đổi trả về bản sao mới.
class Trip {
  Trip({
    required this.id,
    required this.serviceKind,
    required this.status,
    required this.distanceKm,
    required this.fareVnd,
    this.driverId,
    required this.requestedAt,
  }) : assert(requestedAt.isUtc, 'requestedAt phải là giờ UTC');

  /// Tạo chuyến mới ở trạng thái `requested`, tính cước bằng [quoteFor].
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
         fareVnd: quoteFor(distanceKm).fareVnd,
         requestedAt: requestedAt,
       );

  final String id;
  final ServiceKind serviceKind;
  final TripStatus status;
  final double distanceKm;
  final int fareVnd;
  final String? driverId;
  final DateTime requestedAt;

  /// Áp dụng [event] và trả về chuyến mới; chuyến hiện tại không bị sửa.
  ///
  /// Ném [StateError] khi cặp (trạng thái, sự kiện) không hợp lệ.
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
    fareVnd: fareVnd,
    driverId: driverId ?? this.driverId,
    requestedAt: requestedAt,
  );

  /// Hai bản chụp của cùng một chuyến bằng nhau khi cùng id và cùng trạng thái.
  @override
  bool operator ==(Object other) =>
      other is Trip && other.id == id && other.status == status;

  @override
  int get hashCode => Object.hash(id, status);

  @override
  String toString() =>
      'Trip($id, ${serviceKind.name}, ${status.name}, '
      '$distanceKm km, $fareVnd VND, driver: ${driverId ?? '-'})';
}
