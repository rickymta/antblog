import 'push_message.dart';

/// Ranh giới tới Firebase Messaging. Phần còn lại của app chỉ biết interface
/// này; test thay bằng bản giả, app thật dùng `FirebasePushSource`.
abstract interface class PushSource {
  /// Tin tới khi app đang mở (foreground). Hệ điều hành không tự hiện gì.
  Stream<PushMessage> get onForeground;

  /// Người dùng chạm vào thông báo khi app đang ở nền (chưa bị tắt hẳn).
  Stream<PushMessage> get onOpenedFromBackground;

  /// Tin đã khởi chạy app từ trạng thái bị tắt hẳn (terminated). Plugin chỉ
  /// trả một lần; các lần sau là `null`.
  Future<PushMessage?> takeInitialMessage();

  /// Định danh đăng ký push của thiết bị (FID) mỗi khi có hoặc đổi.
  Stream<String> get onRegistered;

  Future<PushPermission> permission();

  /// Hiện hộp thoại xin quyền của hệ điều hành nếu còn hỏi được.
  Future<PushPermission> requestPermission();

  /// Đăng ký thiết bị với FCM; kết quả đi về qua [onRegistered].
  Future<void> register();

  /// Huỷ đăng ký: tin gửi tới định danh cũ sẽ không tới máy này nữa.
  Future<void> unregister();
}
