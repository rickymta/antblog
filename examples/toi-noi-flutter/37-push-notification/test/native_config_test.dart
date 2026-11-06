import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_37/notification_channel.dart';
import 'package:toi_noi_37/push_router.dart';
import 'package:xml/xml.dart';

/// Máy viết bài không build được Gradle hay Xcode, nên file trong
/// `native-config/` chỉ được kiểm bằng cách đọc giá trị. Test này chặn các lỗi
/// cấu hình hay gặp và chặn việc commit khoá thật.
String read(String path) => File('native-config/$path').readAsStringSync();

/// Từ điển của một plist phẳng: khoá -> giá trị chuỗi/bool/danh sách chuỗi.
Map<String, Object> plist(String text) {
  final dict = XmlDocument.parse(text).findAllElements('dict').first;
  final out = <String, Object>{};
  final children = dict.childElements.toList();
  for (var i = 0; i + 1 < children.length; i += 2) {
    final value = children[i + 1];
    out[children[i].innerText] = switch (value.name.local) {
      'true' => true,
      'false' => false,
      'array' => value.childElements.map((e) => e.innerText).toList(),
      _ => value.innerText,
    };
  }
  return out;
}

void main() {
  group('AndroidManifest.xml.example', () {
    final doc = XmlDocument.parse(read('android/AndroidManifest.xml.example'));

    Map<String, String> metaData() => {
      for (final e in doc.findAllElements('meta-data'))
        e.getAttribute('android:name')!: e.getAttribute('android:value')!,
    };

    test('khai quyền POST_NOTIFICATIONS (Android 13+)', () {
      final permissions = doc
          .findAllElements('uses-permission')
          .map((e) => e.getAttribute('android:name'));
      expect(permissions, contains('android.permission.POST_NOTIFICATIONS'));
    });

    test('kênh mặc định của FCM trùng hằng số trong Dart', () {
      expect(
        metaData()['com.google.firebase.messaging.default_notification_channel_id'],
        tripUpdatesChannelId,
      );
    });

    test('bật định danh FID', () {
      expect(metaData()['firebase_messaging_installation_id_enabled'], 'true');
    });

    test('không mở HTTP không mã hoá', () {
      expect(
        read('android/AndroidManifest.xml.example'),
        isNot(contains('usesCleartextTraffic')),
      );
    });
  });

  group('google-services.json.example', () {
    final json = jsonDecode(
      read('android/google-services.json.example'),
    ) as Map<String, Object?>;
    final clients = (json['client']! as List).cast<Map<String, Object?>>();

    test('có ba app Android, một cho mỗi flavor của app khách', () {
      final packages = clients
          .map(
            (c) =>
                ((c['client_info']! as Map)['android_client_info']!
                    as Map)['package_name'],
          )
          .toList();
      expect(packages, [
        'vn.toinoi.customer.dev',
        'vn.toinoi.customer.staging',
        'vn.toinoi.customer',
      ]);
    });

    test('mọi khoá và mã dự án là giá trị giả rõ ràng', () {
      final info = json['project_info']! as Map;
      expect(info['project_id'], 'toi-noi-example');
      expect(info['project_number'], matches(RegExp(r'^0+$')));
      for (final c in clients) {
        final key = ((c['api_key']! as List).single as Map)['current_key'];
        expect(key, contains('FAKE'));
        final appId = (c['client_info']! as Map)['mobilesdk_app_id'] as String;
        expect(appId, contains('FAKE'));
      }
    });
  });

  group('iOS', () {
    test('GoogleService-Info.plist.example là giá trị giả', () {
      final p = plist(read('ios/GoogleService-Info.plist.example'));
      expect(p['API_KEY'], contains('FAKE'));
      expect(p['GCM_SENDER_ID'], matches(RegExp(r'^0+$')));
      expect(p['GOOGLE_APP_ID'], contains('FAKE'));
      expect(p['BUNDLE_ID'], 'vn.toinoi.customer.dev');
    });

    test(
      'Info.plist: bật FID và hai background mode theo hướng dẫn Firebase',
      () {
        final p = plist(read('ios/Info.plist.fragment.example'));
        expect(p['FirebaseMessagingInstallationIdEnabled'], isTrue);
        expect(
          p['UIBackgroundModes'],
          containsAll(['remote-notification', 'fetch']),
        );
      },
    );

    test('entitlements có aps-environment hợp lệ', () {
      final p = plist(read('ios/Runner.entitlements.example'));
      expect(p['aps-environment'], isIn(['development', 'production']));
    });
  });

  group('đoạn Gradle', () {
    final text = read('android/gradle-google-services.kts.example');

    test('thêm đúng plugin google-services', () {
      expect(text, contains('id("com.google.gms.google-services") version'));
      expect(text, contains('apply false'));
    });

    test('đáp ứng yêu cầu của flutter_local_notifications 23', () {
      expect(text, contains('compileSdk = 37'));
      expect(text, contains('isCoreLibraryDesugaringEnabled = true'));
      expect(
        text,
        contains('coreLibraryDesugaring("com.android.tools:desugar_jdk_libs'),
      );
      expect(text, contains('id("com.android.application") version "9.1.1"'));
    });
  });

  group('AppDelegate.swift', () {
    final text = read('ios/AppDelegate.swift');

    test('cấu hình delegate của thông báo trước khi gọi super', () {
      final firebase = text.indexOf(
        'FLTFirebaseMessagingPlugin.configureNotificationCenterDelegate()',
      );
      final local = text.indexOf(
        'UNUserNotificationCenter.current().delegate = self',
      );
      final superCall = text.indexOf('return super.application(');
      expect(local, isNonNegative);
      expect(firebase, greaterThan(local));
      expect(superCall, greaterThan(firebase));
    });

    test('vẫn đăng ký plugin theo UIScene', () {
      expect(text, contains('FlutterImplicitEngineDelegate'));
      expect(
        text,
        contains(
          'GeneratedPluginRegistrant.register(with: engineBridge.pluginRegistry)',
        ),
      );
    });
  });

  group('tin mẫu của backend (fcm-send.json.example)', () {
    final text = read('backend/fcm-send.json.example');
    final message =
        (jsonDecode(text) as Map<String, Object?>)['message']!
            as Map<String, Object?>;
    final data = (message['data']! as Map).cast<String, Object?>();

    test('nhắm bằng fid, không dùng trường token đã deprecated', () {
      expect(message['fid'], contains('FAKE'));
      expect(message, isNot(contains('token')));
    });

    test('data là map chuỗi sang chuỗi và PushRouter hiểu được', () {
      expect(data.values.every((v) => v is String), isTrue);
      expect(const PushRouter().decide(data), const OpenTrip('T-1002'));
    });

    test('data chỉ có hai khoá: loại tin và mã chuyến', () {
      expect(data.keys, unorderedEquals(['type', 'tripId']));
    });

    test('nội dung hiển thị không mang tên, số điện thoại hay giá', () {
      final shown = jsonEncode(message['notification']);
      expect(shown, isNot(matches(RegExp(r'\d{3,}'))));
      expect(shown, isNot(contains('VND')));
    });

    test('channel_id trùng kênh trong app', () {
      final android = message['android']! as Map;
      expect(
        (android['notification']! as Map)['channel_id'],
        tripUpdatesChannelId,
      );
    });

    test('nhỏ hơn giới hạn 4096 byte của tài liệu FCM', () {
      expect(utf8.encode(jsonEncode(message)).length, lessThan(4096));
    });
  });

  group('không có bí mật thật trong lab', () {
    // Các dạng khoá có định dạng nhận ra được. Chuỗi "AIza" + 35 ký tự là
    // dạng của Google API key; "APA91b" là dạng của FCM registration token cũ.
    final patterns = <String, RegExp>{
      'Google API key': RegExp(r'AIza[0-9A-Za-z_\-]{35}'),
      'khoá riêng PEM': RegExp(r'-----BEGIN [A-Z ]*PRIVATE KEY-----'),
      'private_key trong JSON': RegExp(r'"private_key"\s*:'),
      'OAuth access token': RegExp(r'ya29\.[0-9A-Za-z_\-]{20,}'),
      'FCM token cũ': RegExp(r'[A-Za-z0-9_\-]{11}:APA91b[A-Za-z0-9_\-]{50,}'),
      'OAuth client id': RegExp(
        r'\d{12}-[a-z0-9]{32}\.apps\.googleusercontent\.com',
      ),
    };

    final files = [
      for (final dir in ['lib', 'test', 'native-config'])
        ...Directory(dir)
            .listSync(recursive: true)
            .whereType<File>()
            .where((f) => !f.path.endsWith('native_config_test.dart')),
      File('README.md'),
      File('pubspec.yaml'),
    ];

    test('đã quét được file', () => expect(files.length, greaterThan(15)));

    for (final entry in patterns.entries) {
      test('không có ${entry.key}', () {
        final hits = [
          for (final f in files)
            if (entry.value.hasMatch(f.readAsStringSync())) f.path,
        ];
        expect(hits, isEmpty);
      });
    }

    test('bộ quét bắt được khoá giống thật (kiểm chính bộ quét)', () {
      final fakeKey = 'AIza${'a' * 35}';
      expect(patterns['Google API key']!.hasMatch('"$fakeKey"'), isTrue);
      expect(
        patterns['khoá riêng PEM']!.hasMatch('-----BEGIN PRIVATE KEY-----'),
        isTrue,
      );
    });

    test('không có file cấu hình thật hay khoá APNs, chỉ có bản .example', () {
      final names = Directory('.')
          .listSync(recursive: true)
          .whereType<File>()
          .where((f) => !f.path.contains('.dart_tool'))
          .map((f) => f.uri.pathSegments.last)
          .toSet();
      expect(names, isNot(contains('google-services.json')));
      expect(names, isNot(contains('GoogleService-Info.plist')));
      expect(names, isNot(contains('firebase_options.dart')));
      expect(names.where((n) => n.endsWith('.p8')), isEmpty);
    });

    test('.gitignore chặn file thật và khoá APNs', () {
      final ignore = File('.gitignore').readAsStringSync().split('\n');
      for (final entry in [
        'google-services.json',
        'GoogleService-Info.plist',
        'firebase_options.dart',
        '*.p8',
        '/android/',
        '/ios/',
      ]) {
        expect(ignore, contains(entry));
      }
    });
  });
}
