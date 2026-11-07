import 'dart:async';

import 'package:flutter_local_notifications/flutter_local_notifications.dart';

import 'notification_channel.dart';

/// Thông báo cục bộ: chỉ cần ba việc. Hiện một thông báo, báo khi người dùng
/// chạm vào, và cho biết thông báo nào đã khởi chạy app.
abstract interface class LocalNotifier {
  /// Hiện thông báo; [payload] trả lại nguyên văn qua [taps] khi người dùng chạm.
  Future<void> show({
    required int id,
    required String title,
    required String body,
    required String payload,
  });

  /// Payload của thông báo mà người dùng vừa chạm (app đang chạy).
  Stream<String> get taps;

  /// Payload của thông báo đã khởi chạy app từ trạng thái bị tắt hẳn, hoặc
  /// `null` nếu app không được mở bằng thông báo cục bộ.
  Future<String?> launchPayload();
}

/// Bản dùng `flutter_local_notifications`: lớp bọc mỏng, chưa chạy trên thiết bị.
final class FlutterLocalNotifier implements LocalNotifier {
  FlutterLocalNotifier._(this._plugin);

  final FlutterLocalNotificationsPlugin _plugin;
  final _taps = StreamController<String>.broadcast();

  static Future<FlutterLocalNotifier> create() async {
    final notifier = FlutterLocalNotifier._(FlutterLocalNotificationsPlugin());
    await notifier._plugin.initialize(
      settings: const InitializationSettings(
        android: AndroidInitializationSettings('@mipmap/ic_launcher'),
        // Mặc định plugin xin quyền ngay lúc initialize. Ta tắt cả ba để hộp
        // thoại chỉ hiện khi người dùng bấm "Bật thông báo".
        iOS: DarwinInitializationSettings(
          requestAlertPermission: false,
          requestSoundPermission: false,
          requestBadgePermission: false,
        ),
      ),
      onDidReceiveNotificationResponse: (response) {
        final payload = response.payload;
        if (payload != null) notifier._taps.add(payload);
      },
    );
    // Tạo kênh khi khởi động: tạo lại kênh đã có là không làm gì.
    await notifier._plugin
        .resolvePlatformSpecificImplementation<
          AndroidFlutterLocalNotificationsPlugin
        >()
        ?.createNotificationChannel(
          const AndroidNotificationChannel(
            tripUpdatesChannelId,
            tripUpdatesChannelName,
            description: tripUpdatesChannelDescription,
            importance: Importance.high,
          ),
        );
    return notifier;
  }

  @override
  Stream<String> get taps => _taps.stream;

  @override
  Future<void> show({
    required int id,
    required String title,
    required String body,
    required String payload,
  }) {
    return _plugin.show(
      id: id,
      title: title,
      body: body,
      payload: payload,
      notificationDetails: const NotificationDetails(
        android: AndroidNotificationDetails(
          tripUpdatesChannelId,
          tripUpdatesChannelName,
          channelDescription: tripUpdatesChannelDescription,
          importance: Importance.high,
          priority: Priority.high,
        ),
        iOS: DarwinNotificationDetails(),
      ),
    );
  }

  @override
  Future<String?> launchPayload() async {
    final details = await _plugin.getNotificationAppLaunchDetails();
    if (details == null || !details.didNotificationLaunchApp) return null;
    return details.notificationResponse?.payload;
  }
}
