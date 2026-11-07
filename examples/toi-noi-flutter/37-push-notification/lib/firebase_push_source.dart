import 'dart:async';

import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:flutter/foundation.dart';

import 'push_message.dart';
import 'push_source.dart';

/// Đổi `RemoteMessage` sang [PushMessage]. Hàm thuần nên test được.
PushMessage toPushMessage(RemoteMessage message) => PushMessage(
  id: message.messageId,
  title: message.notification?.title,
  body: message.notification?.body,
  data: Map<String, Object?>.from(message.data),
);

/// Gộp trạng thái của plugin thành [PushPermission].
PushPermission toPushPermission(AuthorizationStatus status) => switch (status) {
  AuthorizationStatus.authorized ||
  AuthorizationStatus.provisional => PushPermission.granted,
  AuthorizationStatus.notDetermined => PushPermission.notAsked,
  AuthorizationStatus.denied => PushPermission.denied,
  AuthorizationStatus.deniedPermanently => PushPermission.deniedForever,
};

/// Lớp bọc mỏng quanh `FirebaseMessaging`: chưa chạy trên thiết bị.
final class FirebasePushSource implements PushSource {
  FirebasePushSource(this._messaging);

  final FirebaseMessaging _messaging;

  @override
  Stream<PushMessage> get onForeground =>
      FirebaseMessaging.onMessage.map(toPushMessage);

  @override
  Stream<PushMessage> get onOpenedFromBackground =>
      FirebaseMessaging.onMessageOpenedApp.map(toPushMessage);

  @override
  Future<PushMessage?> takeInitialMessage() async {
    final message = await _messaging.getInitialMessage();
    return message == null ? null : toPushMessage(message);
  }

  @override
  Stream<String> get onRegistered => _messaging.onRegistered;

  @override
  Future<PushPermission> permission() async => toPushPermission(
    (await _messaging.getNotificationSettings()).authorizationStatus,
  );

  @override
  Future<PushPermission> requestPermission() async => toPushPermission(
    (await _messaging.requestPermission()).authorizationStatus,
  );

  @override
  Future<void> register() async {
    // Tài liệu Firebase: trên iOS, token APNs phải có trước khi gọi API của
    // FCM. Chờ có giới hạn; hết 5 giây vẫn chưa có thì cứ gọi register và để
    // plugin báo lỗi (nếu có) cho PushRegistrar.
    if (defaultTargetPlatform == TargetPlatform.iOS) {
      for (var i = 0; i < 5; i++) {
        if (await _messaging.getAPNSToken() != null) break;
        await Future<void>.delayed(const Duration(seconds: 1));
      }
    }
    await _messaging.register();
  }

  @override
  Future<void> unregister() => _messaging.unregister();
}
