import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_25/config/app_config.dart';
import 'package:toi_noi_25/config/flavor.dart';

import 'support/config_files.dart';
import 'support/secret_scan.dart';

void main() {
  test('có đúng ba file, mỗi flavor một file', () {
    final jsonFiles =
        Directory('config')
            .listSync()
            .map((entity) => entity.uri.pathSegments.last)
            .where((name) => name.endsWith('.json'))
            .toList()
          ..sort();
    expect(jsonFiles, ['dev.json', 'prod.json', 'staging.json']);
  });

  for (final name in flavorFileNames) {
    group('config/$name.json', () {
      final json = readRawConfig('config/$name.json');

      test('có đúng bộ khoá của AppConfig, không thừa không thiếu', () {
        expect(json.keys.toSet(), AppConfig.keys.toSet());
      });

      test('FLAVOR trùng tên file', () {
        expect(json[AppConfig.keyFlavor], name);
      });

      test(
        'kiểu JSON đúng ý: bool là bool, timeout là số, còn lại là chuỗi',
        () {
          expect(json[AppConfig.keyLogHttp], isA<bool>());
          expect(json[AppConfig.keyConnectTimeoutMs], isA<int>());
          expect(json[AppConfig.keyReceiveTimeoutMs], isA<int>());
          expect(json[AppConfig.keyApiBaseUrl], isA<String>());
          // null sẽ thành chuỗi "null" khi qua công cụ Flutter.
          expect(json.values, everyElement(isNotNull));
        },
      );

      test('qua kiểm tra của AppConfig.fromMap', () {
        expect(() => AppConfig.fromMap(asDefines(json)), returnsNormally);
      });

      test('không chứa thứ giống bí mật', () {
        expect(findSecretLike(json), isEmpty);
      });
    });
  }

  test('prod: https, không trỏ máy cục bộ, không bật log HTTP', () {
    final config = AppConfig.fromMap(readFlavorDefines('prod'));
    expect(config.apiBaseUrl.scheme, 'https');
    expect(isLocalHost(config.apiBaseUrl.host), isFalse);
    expect(config.apiBaseUrl.host, isNot(contains('localhost')));
    expect(config.logHttp, isFalse);
  });

  test('staging: https, không trỏ máy cục bộ, khác host của prod', () {
    final staging = AppConfig.fromMap(readFlavorDefines('staging'));
    final prod = AppConfig.fromMap(readFlavorDefines('prod'));
    expect(staging.apiBaseUrl.scheme, 'https');
    expect(isLocalHost(staging.apiBaseUrl.host), isFalse);
    expect(staging.apiBaseUrl.host, isNot(prod.apiBaseUrl.host));
  });

  test('dev trỏ về máy dev', () {
    final dev = AppConfig.fromMap(readFlavorDefines('dev'));
    expect(dev.flavor, Flavor.dev);
    expect(isLocalHost(dev.apiBaseUrl.host), isTrue);
  });

  test('ba file khác nhau ở địa chỉ API', () {
    final hosts = {
      for (final name in flavorFileNames)
        AppConfig.fromMap(readFlavorDefines(name)).apiBaseUrl.host,
    };
    expect(hosts, hasLength(3));
  });

  group('config/local.json.example', () {
    test('là mẫu dev hợp lệ cho điện thoại thật qua địa chỉ LAN', () {
      final config = AppConfig.fromMap(
        asDefines(readRawConfig('config/local.json.example')),
      );
      expect(config.flavor, Flavor.dev);
      expect(isLocalHost(config.apiBaseUrl.host), isTrue);
    });

    test('config/local.json bị .gitignore chặn', () {
      expect(
        File('.gitignore').readAsLinesSync(),
        contains('config/local.json'),
      );
    });
  });
}
