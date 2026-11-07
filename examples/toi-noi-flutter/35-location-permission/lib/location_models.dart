import 'package:toi_noi_core/toi_noi_core.dart';

/// Quyền vị trí hệ điều hành đang cấp cho app, đã gộp về bốn giá trị.
///
/// `denied` gồm cả "chưa hỏi" lẫn "đã từ chối nhưng còn hỏi lại được": plugin
/// không phân biệt hai trường hợp này, nên [LocationAccessState] phải tự nhớ
/// app đã hỏi chưa (xem `AskedStore`).
enum SystemPermission { denied, deniedForever, whileInUse, always }

/// Độ chính xác người dùng cho phép: Android 12+ (Precise/Approximate) và
/// iOS 14+ (Precise Location bật/tắt).
enum LocationPrecision { precise, reduced }

/// App nào đang chạy. Chỉ app tài xế được phép xin quyền nền.
enum AppRole { customer, driver }

/// Một lần đo vị trí, không phụ thuộc kiểu `Position` của plugin.
final class LocationFix {
  const LocationFix({
    required this.latitude,
    required this.longitude,
    required this.accuracyMeters,
    required this.timestamp,
  });

  final double latitude;
  final double longitude;

  /// Sai số ngang ước tính, mét. 0 nghĩa là thiết bị không báo.
  final double accuracyMeters;

  /// Thời điểm đo, UTC.
  final DateTime timestamp;

  @override
  bool operator ==(Object other) =>
      other is LocationFix &&
      other.latitude == latitude &&
      other.longitude == longitude &&
      other.accuracyMeters == accuracyMeters &&
      other.timestamp == timestamp;

  @override
  int get hashCode =>
      Object.hash(latitude, longitude, accuracyMeters, timestamp);

  @override
  String toString() => 'LocationFix($latitude, $longitude ±${accuracyMeters}m)';
}

/// Mức chính xác muốn xin, độc lập với plugin.
enum TrackingAccuracy { balanced, high }

/// Cấu hình một luồng vị trí. Các số là minh hoạ, không phải số đã đo.
final class TrackingProfile {
  const TrackingProfile({
    required this.name,
    required this.accuracy,
    required this.distanceFilterMeters,
    required this.interval,
    required this.background,
  });

  /// Khách chọn điểm đón: chỉ khi màn hình mở, ngắt khi rời màn hình.
  static const customerPickup = TrackingProfile(
    name: 'customerPickup',
    accuracy: TrackingAccuracy.high,
    distanceFilterMeters: 10,
    interval: Duration(seconds: 5),
    background: false,
  );

  /// Tài xế đang online nhưng chưa có chuyến: thưa hơn, tiết kiệm pin.
  static const driverOnline = TrackingProfile(
    name: 'driverOnline',
    accuracy: TrackingAccuracy.balanced,
    distanceFilterMeters: 50,
    interval: Duration(seconds: 15),
    background: false,
  );

  /// Tài xế đang có chuyến hợp lệ: dày hơn và chạy tiếp khi app xuống nền.
  static const driverOnTrip = TrackingProfile(
    name: 'driverOnTrip',
    accuracy: TrackingAccuracy.high,
    distanceFilterMeters: 25,
    interval: Duration(seconds: 5),
    background: true,
  );

  final String name;
  final TrackingAccuracy accuracy;

  /// Khoảng di chuyển tối thiểu (mét) trước khi có bản cập nhật mới.
  final int distanceFilterMeters;

  /// Khoảng cách thời gian tối thiểu giữa hai bản cập nhật (Android).
  final Duration interval;

  /// Có chạy tiếp khi app xuống nền không.
  final bool background;
}

/// Chọn profile theo vai trò và trạng thái chuyến.
///
/// Vị trí nền chỉ bật khi tài xế có chuyến đang diễn ra (từ `accepted` tới
/// `inTransit`); khách không bao giờ có profile nền.
TrackingProfile profileFor(AppRole role, TripStatus? tripStatus) {
  if (role == AppRole.customer) {
    return TrackingProfile.customerPickup;
  }
  return switch (tripStatus) {
    TripStatus.accepted ||
    TripStatus.arriving ||
    TripStatus.pickedUp ||
    TripStatus.inTransit => TrackingProfile.driverOnTrip,
    TripStatus.requested ||
    TripStatus.searching ||
    TripStatus.completed ||
    TripStatus.cancelled ||
    null => TrackingProfile.driverOnline,
  };
}

/// Lỗi của lớp [LocationSource]: tách khỏi exception của plugin để phần còn
/// lại của app không import `geolocator`.
sealed class LocationSourceException implements Exception {
  const LocationSourceException(this.message);
  final String message;

  @override
  String toString() => '$runtimeType: $message';
}

/// Thiếu khai báo trong `Info.plist` hoặc `AndroidManifest.xml`.
final class LocationConfigurationException extends LocationSourceException {
  const LocationConfigurationException(super.message);
}

/// Đang có một hộp thoại xin quyền chưa trả lời.
final class LocationRequestInProgressException extends LocationSourceException {
  const LocationRequestInProgressException(super.message);
}

/// Người dùng tắt dịch vụ vị trí của thiết bị khi luồng đang chạy.
final class LocationServiceOffException extends LocationSourceException {
  const LocationServiceOffException(super.message);
}
