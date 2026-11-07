import 'dart:async';

import 'package:dio/dio.dart';
import 'package:firebase_core/firebase_core.dart';
import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

import 'app.dart';
import 'background_handler.dart';
import 'device_api.dart';
import 'firebase_push_source.dart';
import 'local_notifier.dart';
import 'push_navigator.dart';
import 'push_registrar.dart';

const _apiBaseUrl = String.fromEnvironment(
  'API_BASE_URL',
  defaultValue: 'https://api.toinoi.example',
);

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Không truyền options: Firebase đọc cấu hình từ google-services.json
  // (Android) và GoogleService-Info.plist (iOS), hai file không nằm trong repo.
  await Firebase.initializeApp();
  FirebaseMessaging.onBackgroundMessage(firebaseMessagingBackgroundHandler);

  final source = FirebasePushSource(FirebaseMessaging.instance);
  final notifier = await FlutterLocalNotifier.create();
  final registrar = PushRegistrar(
    source: source,
    api: HttpDeviceApi(Dio(BaseOptions(baseUrl: _apiBaseUrl))),
    deviceId: generateInstallationId(),
    platform: defaultTargetPlatform == TargetPlatform.iOS ? 'ios' : 'android',
  );
  final router = buildRouter(registrar: registrar);
  final navigator = PushNavigator(
    source: source,
    notifier: notifier,
    open: router.go,
    onRejected: (reason) => debugPrint('Bỏ qua push: $reason'),
  );

  runApp(ToiNoiPushApp(router: router));
  unawaited(navigator.start());
  // Giả sử người dùng đã đăng nhập: chỉ đăng ký lại nếu quyền đã có sẵn.
  unawaited(registrar.resume());
}
