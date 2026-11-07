import 'package:firebase_core/firebase_core.dart';
import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:flutter/foundation.dart';

import 'push_router.dart';

/// Chạy khi có tin lúc app ở nền hoặc bị tắt. Theo tài liệu Firebase hàm
/// phải là top-level, không được ẩn danh, và trên Flutter 3.3 trở lên phải có
/// `@pragma('vm:entry-point')` để không bị loại khi build release.
///
/// Trên Android hàm chạy trong isolate riêng: không có state, provider hay
/// UI của app. Vì vậy chỉ làm việc ngắn và độc lập.
@pragma('vm:entry-point')
Future<void> firebaseMessagingBackgroundHandler(RemoteMessage message) async {
  await Firebase.initializeApp();
  debugPrint(PushRouter.safeSummary(message.data));
}
