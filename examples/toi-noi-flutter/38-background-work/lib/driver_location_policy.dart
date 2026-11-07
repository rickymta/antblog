import 'package:toi_noi_core/toi_noi_core.dart';

/// Quy tắc nghiệp vụ: khi nào app tài xế được phép gửi vị trí lúc ở nền.
///
/// Lớp thuần Dart, không biết gì về plugin hay nền tảng, nên test được bằng
/// `dart test` và dùng lại được ở app khác. Bật hay tắt luồng vị trí là việc
/// của [DriverLocationController]; lớp này chỉ trả lời "có được phép không".
final class DriverLocationPolicy {
  const DriverLocationPolicy();

  /// Chỉ khi tài xế đang online và có một chuyến ở trạng thái cần theo dõi.
  ///
  /// [tripStatus] là `null` khi tài xế chưa có chuyến nào.
  bool allowsBackgroundSharing({
    required TripStatus? tripStatus,
    required bool driverOnline,
  }) {
    if (!driverOnline || tripStatus == null) return false;
    return tracksStatus(tripStatus);
  }

  /// Trạng thái của chuyến mà khách cần thấy xe di chuyển.
  ///
  /// Liệt kê đủ tám trạng thái, không dùng `_`: thêm trạng thái mới thì
  /// `dart analyze` buộc ta quyết định nó có được theo dõi hay không.
  bool tracksStatus(TripStatus status) => switch (status) {
    TripStatus.accepted ||
    TripStatus.arriving ||
    TripStatus.pickedUp ||
    TripStatus.inTransit => true,
    // Chưa có tài xế hoặc chuyến đã kết thúc: không có lý do gì để thu vị trí.
    TripStatus.requested ||
    TripStatus.searching ||
    TripStatus.completed ||
    TripStatus.cancelled => false,
  };
}
