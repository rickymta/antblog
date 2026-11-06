import 'package:toi_noi_31/notification_permission.dart';

/// Quyền thông báo giả: trả lời ngay, không có hộp thoại nào.
final class FakeNotificationPermission
    implements NotificationPermissionGateway {
  FakeNotificationPermission({
    this.current = NotificationPermissionStatus.denied,
    this.answer = NotificationPermissionStatus.granted,
  });

  NotificationPermissionStatus current;

  /// Điều người dùng "bấm" khi được hỏi.
  final NotificationPermissionStatus answer;

  int requests = 0;

  @override
  Future<NotificationPermissionStatus> status() async => current;

  @override
  Future<NotificationPermissionStatus> request() async {
    requests++;
    return current = answer;
  }
}
