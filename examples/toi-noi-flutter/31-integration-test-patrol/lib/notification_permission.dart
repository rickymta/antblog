import 'package:permission_handler/permission_handler.dart';

enum NotificationPermissionStatus { granted, denied }

/// Ranh giới giữa app và hệ điều hành về quyền thông báo. `flutter test` và
/// `integration_test` thường thay bằng bản giả, vì chỉ Patrol bấm được hộp
/// thoại của hệ điều hành.
abstract interface class NotificationPermissionGateway {
  Future<NotificationPermissionStatus> status();

  /// Hỏi người dùng. Với bản thật, future này chỉ hoàn tất sau khi người dùng
  /// trả lời hộp thoại của hệ điều hành.
  Future<NotificationPermissionStatus> request();
}

/// Bản thật, dùng `permission_handler`.
final class SystemNotificationPermission
    implements NotificationPermissionGateway {
  const SystemNotificationPermission();

  @override
  Future<NotificationPermissionStatus> status() async =>
      _map(await Permission.notification.status);

  @override
  Future<NotificationPermissionStatus> request() async =>
      _map(await Permission.notification.request());

  static NotificationPermissionStatus _map(PermissionStatus status) =>
      status.isGranted || status.isProvisional
      ? NotificationPermissionStatus.granted
      : NotificationPermissionStatus.denied;
}
