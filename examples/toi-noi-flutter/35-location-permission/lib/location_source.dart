import 'location_models.dart';

/// Ranh giới duy nhất giữa app và plugin vị trí. Mọi thứ còn lại (máy trạng
/// thái quyền, màn hình, tracker) chỉ biết interface này, nên test bằng fake.
abstract interface class LocationSource {
  /// Dịch vụ vị trí của thiết bị có đang bật không.
  Future<bool> isServiceEnabled();

  /// Quyền hiện tại, không hiện hộp thoại.
  Future<SystemPermission> checkPermission();

  /// Độ chính xác đang được cấp. Chỉ có nghĩa khi đã có quyền.
  Future<LocationPrecision> precision();

  /// Hiện hộp thoại xin quyền "khi dùng app". Trả về quyền sau khi người
  /// dùng trả lời.
  Future<SystemPermission> requestForeground();

  /// Nền tảng này có bước xin nâng lên "luôn cho phép" không. Android thì
  /// không: foreground service đã đủ, và app không khai quyền nền.
  bool get canRequestBackground;

  /// Xin nâng lên "luôn cho phép". Chỉ gọi sau khi đã có quyền "khi dùng app".
  Future<SystemPermission> requestBackground();

  /// Xin nâng từ vị trí gần đúng lên chính xác.
  Future<LocationPrecision> requestPrecise();

  Future<bool> openAppSettings();

  Future<bool> openLocationSettings();

  /// Luồng vị trí theo [profile]. Huỷ subscription là dừng cảm biến.
  Stream<LocationFix> positionStream(TrackingProfile profile);
}
