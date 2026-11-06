import 'package:flutter/foundation.dart';

/// Một tin push, đã bóc khỏi kiểu của plugin để phần còn lại của app (và test)
/// không phải biết `RemoteMessage`.
@immutable
final class PushMessage {
  const PushMessage({
    this.id,
    this.title,
    this.body,
    this.data = const <String, Object?>{},
  });

  final String? id;
  final String? title;
  final String? body;

  /// Với FCM, `data` luôn là map chuỗi sang chuỗi; ta vẫn nhận `Object?` để
  /// kiểm kiểu một lần ở [PushRouter] thay vì tin vào nơi gửi.
  final Map<String, Object?> data;
}

/// Quyền hiển thị thông báo, gộp từ trạng thái của iOS và Android 13+.
enum PushPermission {
  /// Chưa hỏi người dùng.
  notAsked,

  /// Được phép, kể cả chế độ provisional của iOS (thông báo lặng lẽ).
  granted,

  /// Từ chối, hệ điều hành có thể còn hỏi lại.
  denied,

  /// Từ chối và hệ điều hành sẽ không hỏi nữa: chỉ còn cách mở Cài đặt.
  deniedForever;

  bool get isGranted => this == PushPermission.granted;
}
