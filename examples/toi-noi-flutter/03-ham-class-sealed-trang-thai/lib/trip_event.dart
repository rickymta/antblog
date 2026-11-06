/// Ai là người huỷ chuyến.
enum CancelledBy { customer, driver }

/// Sự kiện làm chuyến đổi trạng thái. `sealed` để compiler biết đủ các biến thể.
sealed class TripEvent {
  const TripEvent();
}

/// Hệ thống bắt đầu tìm tài xế.
final class SearchStarted extends TripEvent {
  const SearchStarted();
}

/// Một tài xế nhận chuyến.
final class DriverAccepted extends TripEvent {
  const DriverAccepted(this.driverId);

  final String driverId;
}

/// Tài xế báo đang tới điểm đón.
final class DriverArrived extends TripEvent {
  const DriverArrived();
}

/// Khách lên xe hoặc tài xế đã lấy hàng.
final class PickedUp extends TripEvent {
  const PickedUp();
}

/// Xe bắt đầu chở khách hoặc giao hàng tới điểm đến.
final class TripStarted extends TripEvent {
  const TripStarted();
}

/// Chuyến kết thúc thành công.
final class Completed extends TripEvent {
  const Completed();
}

/// Chuyến bị huỷ bởi khách hoặc tài xế.
final class Cancelled extends TripEvent {
  const Cancelled({required this.by});

  final CancelledBy by;
}

/// Mô tả một sự kiện bằng tiếng Việt.
///
/// Không có nhánh `_`: thêm một lớp con mới vào [TripEvent] mà quên sửa hàm
/// này sẽ khiến `dart analyze` báo lỗi.
String describeEvent(TripEvent event) => switch (event) {
  SearchStarted() => 'Bắt đầu tìm tài xế',
  DriverAccepted(:final driverId) => 'Tài xế $driverId nhận chuyến',
  DriverArrived() => 'Tài xế đang tới điểm đón',
  PickedUp() => 'Đã đón khách hoặc lấy hàng',
  TripStarted() => 'Bắt đầu chở hoặc giao',
  Completed() => 'Hoàn thành chuyến',
  Cancelled(:final by) => switch (by) {
    CancelledBy.customer => 'Khách huỷ chuyến',
    CancelledBy.driver => 'Tài xế huỷ chuyến',
  },
};
