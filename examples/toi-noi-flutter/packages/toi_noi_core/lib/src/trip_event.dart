/// Ai là người huỷ chuyến.
enum CancelledBy { customer, driver }

/// Sự kiện làm chuyến đổi trạng thái. `sealed` để compiler biết đủ các biến
/// thể, kể cả khi `switch` nằm ở package khác.
sealed class TripEvent {
  const TripEvent();
}

/// Hệ thống bắt đầu tìm tài xế (`requested` sang `searching`).
final class SearchStarted extends TripEvent {
  const SearchStarted();
}

/// Một tài xế nhận chuyến (`searching` sang `accepted`).
final class DriverAccepted extends TripEvent {
  const DriverAccepted(this.driverId);

  final String driverId;

  @override
  bool operator ==(Object other) =>
      other is DriverAccepted && other.driverId == driverId;

  @override
  int get hashCode => Object.hash(DriverAccepted, driverId);
}

/// Tài xế báo đang tới điểm đón (`accepted` sang `arriving`).
final class DriverArrived extends TripEvent {
  const DriverArrived();
}

/// Khách lên xe hoặc tài xế đã lấy hàng (`arriving` sang `pickedUp`).
final class PickedUp extends TripEvent {
  const PickedUp();
}

/// Xe bắt đầu chở hoặc giao tới điểm đến (`pickedUp` sang `inTransit`).
final class TripStarted extends TripEvent {
  const TripStarted();
}

/// Chuyến kết thúc thành công (`inTransit` sang `completed`).
final class Completed extends TripEvent {
  const Completed();
}

/// Chuyến bị huỷ ở `searching`, `accepted` hoặc `arriving`.
final class Cancelled extends TripEvent {
  const Cancelled({required this.by});

  final CancelledBy by;

  @override
  bool operator ==(Object other) => other is Cancelled && other.by == by;

  @override
  int get hashCode => Object.hash(Cancelled, by);
}
