import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:yaml/yaml.dart';

String _read(String path) => File(path).readAsStringSync();

const _androidApp = 'native-config/android/app';

void main() {
  late YamlMap patrol;
  late String packageName;

  setUpAll(() {
    final pubspec = loadYaml(_read('pubspec.yaml')) as YamlMap;
    patrol = pubspec['patrol'] as YamlMap;
    packageName = (patrol['android'] as YamlMap)['package_name'] as String;
  });

  group('khối patrol: trong pubspec.yaml', () {
    test('có tên app, package Android và bundle id iOS', () {
      expect(patrol['app_name'], 'Toi Noi');
      expect(packageName, 'vn.toinoi.toi_noi_31');
      expect((patrol['ios'] as YamlMap)['bundle_id'], 'vn.toinoi.toiNoi31');
    });

    test(
      'dùng thư mục test mặc định patrol_test/ (không đặt test_directory)',
      () {
        expect(patrol.containsKey('test_directory'), isFalse);
        expect(Directory('patrol_test').existsSync(), isTrue);
      },
    );
  });

  group('Android: Gradle và test runner', () {
    late String gradle;
    setUp(() => gradle = _read('$_androidApp/build.gradle.kts'));

    test('applicationId và namespace khớp package_name của patrol', () {
      expect(gradle, contains('applicationId = "$packageName"'));
      expect(gradle, contains('namespace = "$packageName"'));
    });

    test('dùng PatrolJUnitRunner, xoá dữ liệu app giữa các test, có Orchestrator', () {
      expect(
        gradle,
        contains(
          'testInstrumentationRunner = "pl.leancode.patrol.PatrolJUnitRunner"',
        ),
      );
      expect(
        gradle,
        contains(
          'testInstrumentationRunnerArguments["clearPackageData"] = "true"',
        ),
      );
      expect(gradle, contains('execution = "ANDROIDX_TEST_ORCHESTRATOR"'));
      expect(gradle, contains('androidTestUtil("androidx.test:orchestrator:'));
    });

    test('MainActivityTest.java nằm đúng thư mục theo package và khai đúng package', () {
      final dir = packageName.replaceAll('.', '/');
      final java = _read(
        '$_androidApp/src/androidTest/java/$dir/MainActivityTest.java',
      );
      expect(java, contains('package $packageName;'));
      expect(java, contains('instrumentation.setUp(MainActivity.class);'));
      expect(java, contains('instrumentation.runDartTest(dartTestName);'));
    });
  });

  group('Android: quyền và HTTP không mã hoá', () {
    late String mainManifest;
    late String debugManifest;
    late String networkConfig;
    setUp(() {
      mainManifest = _read('$_androidApp/src/main/AndroidManifest.xml');
      debugManifest = _read('$_androidApp/src/debug/AndroidManifest.xml');
      networkConfig = _read(
        '$_androidApp/src/debug/res/xml/network_security_config.xml',
      );
    });

    test('manifest chính khai POST_NOTIFICATIONS để hộp thoại quyền hiện', () {
      expect(mainManifest, contains('android.permission.POST_NOTIFICATIONS'));
    });

    test('manifest chính (bản release) không mở cleartext dưới bất cứ hình thức nào', () {
      expect(mainManifest, isNot(contains('usesCleartextTraffic')));
      expect(mainManifest, isNot(contains('networkSecurityConfig')));
      expect(
        Directory('$_androidApp/src/main/res/xml').existsSync(),
        isFalse,
        reason: 'network_security_config chỉ nằm trong src/debug',
      );
    });

    test('manifest debug trỏ tới network_security_config và có INTERNET', () {
      expect(
        debugManifest,
        contains(
          'android:networkSecurityConfig="@xml/network_security_config"',
        ),
      );
      expect(debugManifest, contains('android.permission.INTERNET'));
      expect(debugManifest, isNot(contains('usesCleartextTraffic')));
    });

    test('cleartext chỉ mở cho 10.0.2.2, không có base-config rộng', () {
      expect(networkConfig, contains('cleartextTrafficPermitted="true"'));
      expect(
        networkConfig,
        contains('<domain includeSubdomains="false">10.0.2.2</domain>'),
      );
      expect(networkConfig, isNot(contains('<base-config')));
      final domains = RegExp(r'<domain\b[^>]*>([^<]+)</domain>')
          .allMatches(networkConfig)
          .map((m) => m.group(1))
          .toList();
      expect(domains, ['10.0.2.2']);
    });
  });

  group('iOS', () {
    test('RunnerUITests.m dùng macro runner của patrol cho đúng target', () {
      final m = _read('native-config/ios/RunnerUITests/RunnerUITests.m');
      expect(m, contains('@import patrol;'));
      expect(m, contains('PATROL_INTEGRATION_TEST_IOS_RUNNER(RunnerUITests)'));
    });

    test('ATS: chỉ mở mạng cục bộ, không tắt ATS toàn app', () {
      final plist = _read(
        'native-config/ios/Runner/Info.plist.ats-fragment.xml',
      );
      expect(
        plist,
        contains('<key>NSAllowsLocalNetworking</key>\n    <true/>'),
      );
      expect(plist, isNot(contains('NSAllowsArbitraryLoads')));
    });
  });

  group('không có bí mật', () {
    test('file cấu hình native không chứa khoá, token hay keystore', () {
      final files = Directory('native-config')
          .listSync(recursive: true)
          .whereType<File>();
      expect(files, isNotEmpty);
      final pattern = RegExp(
        r'(storePassword|keyPassword|keyAlias|api[_-]?key|BEGIN [A-Z ]*PRIVATE KEY|Bearer\s+[A-Za-z0-9._-]{20,})',
        caseSensitive: false,
      );
      for (final file in files) {
        expect(
          pattern.hasMatch(file.readAsStringSync()),
          isFalse,
          reason: file.path,
        );
      }
    });
  });
}
