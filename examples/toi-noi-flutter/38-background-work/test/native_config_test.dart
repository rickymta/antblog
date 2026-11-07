import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_38/background_tasks.dart';
import 'package:workmanager/workmanager.dart' show ForegroundServiceType;
import 'package:xml/xml.dart';

const _driver = 'native-config/driver';
const _customer = 'native-config/customer';

String _read(String path) => File(path).readAsStringSync();

Set<String> _permissions(String manifestPath) =>
    XmlDocument.parse(_read(manifestPath))
        .findAllElements('uses-permission')
        .map((e) => e.getAttribute('android:name')!)
        .toSet();

/// Đọc `<dict>` của plist thành Map: string, mảng string và boolean.
Map<String, Object> _plist(String path) {
  final dict = XmlDocument.parse(_read(path)).findAllElements('dict').first;
  final elements = dict.childElements.toList();
  final result = <String, Object>{};
  for (var i = 0; i < elements.length; i += 2) {
    assert(elements[i].name.local == 'key');
    final value = elements[i + 1];
    result[elements[i].innerText] = switch (value.name.local) {
      'string' => value.innerText,
      'array' => value.childElements.map((e) => e.innerText).toList(),
      'true' => true,
      'false' => false,
      final other => throw StateError('Kiểu plist chưa hỗ trợ: $other'),
    };
  }
  return result;
}

/// Thư mục của một package trong cache pub, tra qua `package_config.json`.
String _packageDir(String name) {
  final config = jsonDecode(
    _read('.dart_tool/package_config.json'),
  ) as Map<String, Object?>;
  final packages = (config['packages']! as List<Object?>)
      .cast<Map<String, Object?>>();
  final root =
      packages.firstWhere((p) => p['name'] == name)['rootUri']! as String;
  return Uri.parse(
    Directory.current.uri.resolve('.dart_tool/').resolve(root).toString(),
  ).toFilePath();
}

void main() {
  group('Android: quyền của app tài xế và app khách', () {
    final driver = _permissions(
      '$_driver/android/app/src/main/AndroidManifest.xml',
    );
    final customer = _permissions(
      '$_customer/android/app/src/main/AndroidManifest.xml',
    );

    test(
      'app tài xế khai quyền foreground service loại location (Android 14+)',
      () {
        expect(
          driver,
          containsAll([
            'android.permission.FOREGROUND_SERVICE',
            'android.permission.FOREGROUND_SERVICE_LOCATION',
            'android.permission.ACCESS_FINE_LOCATION',
            'android.permission.ACCESS_COARSE_LOCATION',
            'android.permission.POST_NOTIFICATIONS',
          ]),
        );
      },
    );

    test('cả hai app cố ý không khai ACCESS_BACKGROUND_LOCATION', () {
      expect(
        driver,
        isNot(contains('android.permission.ACCESS_BACKGROUND_LOCATION')),
      );
      expect(
        customer,
        isNot(contains('android.permission.ACCESS_BACKGROUND_LOCATION')),
      );
    });

    test('app khách chỉ có mạng và vị trí khi đang mở', () {
      expect(customer, {
        'android.permission.INTERNET',
        'android.permission.ACCESS_COARSE_LOCATION',
        'android.permission.ACCESS_FINE_LOCATION',
      });
    });

    test('app tài xế hơn app khách đúng ba quyền: foreground service, loại location, thông báo', () {
      expect(customer.difference(driver), isEmpty);
      expect(driver.difference(customer), {
        'android.permission.FOREGROUND_SERVICE',
        'android.permission.FOREGROUND_SERVICE_LOCATION',
        'android.permission.POST_NOTIFICATIONS',
      });
    });

    test(
      'cả hai manifest có activity chính exported (bắt buộc từ Android 12)',
      () {
        for (final dir in [_driver, _customer]) {
          final doc = XmlDocument.parse(
            _read('$dir/android/app/src/main/AndroidManifest.xml'),
          );
          final main = doc.findAllElements('activity').single;
          expect(main.getAttribute('android:exported'), 'true', reason: dir);
        }
      },
    );
  });

  group('iOS: Info.plist', () {
    final driver = _plist('$_driver/ios/Runner/Info.plist.fragment.xml');
    final customer = _plist('$_customer/ios/Runner/Info.plist.fragment.xml');

    test('app tài xế có nền location và fetch', () {
      expect(driver['UIBackgroundModes'], ['location', 'fetch']);
    });

    test('identifier của BGTaskScheduler trùng với tên task trong Dart', () {
      expect(
        driver['BGTaskSchedulerPermittedIdentifiers'],
        TaskNames.iosPermittedIdentifiers,
      );
    });

    test('app tài xế chỉ xin quyền "khi đang dùng", mô tả nói rõ chỉ thu trong chuyến', () {
      final whenInUse =
          driver['NSLocationWhenInUseUsageDescription']! as String;
      expect(whenInUse, contains('Trong chuyến đang chạy'));
      expect(whenInUse, contains('Ngoài chuyến, app không thu vị trí'));
      expect(
        driver.keys.where((k) => k.startsWith('NSLocationAlways')),
        isEmpty,
      );
    });

    test(
      'app khách chỉ có mô tả vị trí khi đang dùng, không có khoá nền nào',
      () {
        expect(customer.keys, ['NSLocationWhenInUseUsageDescription']);
        expect(customer, isNot(contains('UIBackgroundModes')));
        expect(
          customer,
          isNot(contains('BGTaskSchedulerPermittedIdentifiers')),
        );
      },
    );
  });

  group('iOS: cả hai app bỏ lời gọi xin quyền Always của geolocator', () {
    for (final app in [_customer, _driver]) {
      test(
        '$app: CocoaPods đặt cờ BYPASS_PERMISSION_LOCATION_ALWAYS cho geolocator_apple',
        () {
          final podfile = _read('$app/ios/Podfile.fragment.rb');
          expect(podfile, contains('geolocator_apple'));
          expect(podfile, contains('BYPASS_PERMISSION_LOCATION_ALWAYS=1'));
        },
      );

      test(
        '$app: Swift Package Manager truyền cờ bằng biến môi trường lúc build',
        () {
          final script = _read('$app/ios/build-spm.fragment.sh');
          expect(
            script,
            contains('BYPASS_PERMISSION_LOCATION_ALWAYS=1 flutter build ios'),
          );
        },
      );
    }

    test(
      'không app nào khai mô tả quyền Always (đã bỏ lời gọi xin quyền đó)',
      () {
        for (final app in [_customer, _driver]) {
          final keys = _plist('$app/ios/Runner/Info.plist.fragment.xml').keys;
          expect(
            keys.where((k) => k.startsWith('NSLocationAlways')),
            isEmpty,
            reason: app,
          );
        }
      },
    );
  });

  group('iOS: AppDelegate của app tài xế', () {
    test('đăng ký lại handler BGTask trước khi didFinishLaunching trả về', () {
      final swift = _read('$_driver/ios/Runner/AppDelegate.swift');
      final register = swift.indexOf(
        'WorkmanagerPlugin.registerLaunchHandlers()',
      );
      final registrant = swift.indexOf(
        'WorkmanagerPlugin.setPluginRegistrantCallback',
      );
      final returns = swift.indexOf('return super.application(');
      expect(register, isNonNegative);
      expect(registrant, isNonNegative);
      expect(register, lessThan(returns));
      expect(registrant, lessThan(returns));
    });
  });

  group('điều đọc từ source của plugin (phiên bản đã pin trong pubspec.lock)', () {
    test('geolocator_android khai sẵn service loại location', () {
      final manifest = _read(
        '${_packageDir('geolocator_android')}/android/src/main/AndroidManifest.xml',
      );
      expect(manifest, contains('android:foregroundServiceType="location"'));
      expect(manifest, contains('GeolocatorLocationService'));
    });

    test('workmanager chỉ hỗ trợ foreground service loại dataSync và shortService, không có location', () {
      expect(ForegroundServiceType.values.map((t) => t.name), [
        'dataSync',
        'shortService',
      ]);
    });

    test('manifest của workmanager_android tự thêm quyền foreground service và thông báo cho app', () {
      final manifest = _read(
        '${_packageDir('workmanager_android')}/android/src/main/AndroidManifest.xml',
      );
      expect(manifest, contains('android.permission.POST_NOTIFICATIONS'));
      expect(
        manifest,
        contains('android.permission.FOREGROUND_SERVICE_SHORT_SERVICE'),
      );
      expect(manifest, isNot(contains('FOREGROUND_SERVICE_LOCATION')));
    });
  });
}
