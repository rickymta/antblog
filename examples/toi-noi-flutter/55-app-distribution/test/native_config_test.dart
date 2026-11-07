import 'dart:io';

import 'package:test/test.dart';
import 'package:yaml/yaml.dart';

import 'support.dart';

void main() {
  group('đoạn dựng APK khớp với workflow dựng bản của bài 54', () {
    final apkSteps = loadYaml(readNativeConfig(apkStepsPath)) as YamlList;
    YamlMap apkStep(String name) =>
        apkSteps.singleWhere((s) => (s as YamlMap)['name'] == name) as YamlMap;

    test('đoạn dựng APK dùng cùng flavor, cấu hình và build number với bước build AAB', () {
      final aab =
          ((((buildWorkflow['jobs'] as YamlMap)['android'] as YamlMap)['steps']
                      as YamlList)
                  .singleWhere((s) => (s as YamlMap)['name'] == 'Build AAB')
              as YamlMap)['run'];
      final apk = '${apkStep('Build APK')['run']}';
      for (final shared in [
        '--flavor staging',
        '--dart-define-from-file=config/staging.json',
        r'--build-number="$BUILD_NUMBER"',
        '--obfuscate',
      ]) {
        expect('$aab', contains(shared), reason: 'AAB: $shared');
        expect(apk, contains(shared), reason: 'APK: $shared');
      }
      expect(apk, contains('flutter build apk --release'));
    });

    test(
      'đường dẫn APK theo quy tắc của công cụ Flutter cho flavor staging',
      () {
        final path = '${(apkStep('Upload APK')['with'] as YamlMap)['path']}';
        expect(
          path,
          'apps/customer/build/app/outputs/flutter-apk/app-staging-release.apk',
        );
      },
    );

    test('thư mục symbols được upload chính là thư mục --split-debug-info', () {
      final run = '${apkStep('Build APK')['run']}';
      final dir = RegExp(r'--split-debug-info=(\S+)')
          .firstMatch(run)!
          .group(1)!;
      final uploaded =
          '${(apkStep('Upload Dart symbols (APK)')['with'] as YamlMap)['path']}';
      expect(uploaded, 'apps/customer/$dir/');
    });

    test('đoạn dựng APK dùng các output mà job prepare của bài 54 có', () {
      final outputs =
          (((buildWorkflow['jobs'] as YamlMap)['prepare'] as YamlMap)['outputs']
              as YamlMap);
      expect(outputs.keys, contains('build_number'));
    });

    test('upload-artifact dùng cùng phiên bản đã ghim ở bài 54', () {
      final ios =
          ((buildWorkflow['jobs'] as YamlMap)['ios'] as YamlMap)['steps']
              as YamlList;
      final ref =
          '${(ios.singleWhere((s) => (s as YamlMap)['name'] == 'Upload IPA') as YamlMap)['uses']}';
      expect('${apkStep('Upload APK')['uses']}', ref);
    });
  });

  group('fastlane', () {
    final ios = readNativeConfig('ios/fastlane/Distribute.fastfile');
    final android = readNativeConfig('android/fastlane/Distribute.fastfile');

    test(
      'mỗi nền tảng có lane distribute_staging; iOS thêm download_udids',
      () {
        expect(ios, contains('lane :distribute_staging'));
        expect(ios, contains('lane :download_udids'));
        expect(android, contains('lane :distribute_staging'));
        expect(android, isNot(contains('download_udids')));
      },
    );

    test(
      'gọi firebase_app_distribution với nhóm, notes và file service account',
      () {
        for (final src in [ios, android]) {
          expect(src, contains('firebase_app_distribution('));
          expect(src, contains('groups: group'));
          expect(src, contains('release_notes_file:'));
          expect(src, contains('service_credentials_file: distro_credentials'));
        }
        expect(ios, contains('ipa_path: ipa'));
        expect(android, contains('android_artifact_type: "APK"'));
        expect(android, contains('android_artifact_path: apk'));
      },
    );

    test('truyền tường minh file service account, không đọc token', () {
      String code(String src) => [
        for (final l in src.split('\n'))
          if (!l.trimLeft().startsWith('#')) l,
      ].join('\n');
      for (final src in [code(ios), code(android)]) {
        expect(src, contains('ENV["GOOGLE_APPLICATION_CREDENTIALS"]'));
        expect(src, isNot(contains('firebase_cli_token')));
        expect(src, isNot(contains('FIREBASE_TOKEN')));
      }
    });

    test('chỉ đọc mục staging của app-distribution.json', () {
      for (final src in [ios, android]) {
        expect(src, contains('config.dig("apps", app, "staging", platform)'));
        expect(src, isNot(contains('"prod"')));
        expect(src, isNot(contains('"dev"')));
      }
    });

    test('download_udids dùng project_number lấy từ App ID, không dùng app: (deprecated)', () {
      // Plugin 1.0.0 đánh dấu tham số app: là deprecated cho action này.
      final udids = ios.substring(ios.indexOf('lane :download_udids'));
      expect(udids, contains('firebase_app_distribution_get_udids('));
      expect(udids, contains('project_number: project_number'));
      expect(udids, contains('app_id.split(":")[1]'));
      expect(udids, isNot(contains('      app: app_id')));
    });

    test('UDID xuất vào build/ (đã nằm trong .gitignore), không vào thư mục theo dõi', () {
      expect(ios, contains('output_file: "../build/udids.csv"'));
      final ignore = File('${labDir.path}/.gitignore').readAsStringSync();
      expect(ignore, contains('build/'));
      expect(ignore, contains('udids*.csv'));
    });

    test('Pluginfile ghim plugin firebase_app_distribution bản 1.x', () {
      for (final platform in ['ios', 'android']) {
        expect(
          readNativeConfig('$platform/fastlane/Pluginfile'),
          contains("gem 'fastlane-plugin-firebase_app_distribution', '~> 1.0'"),
        );
      }
    });

    test('cú pháp Ruby hợp lệ (ruby -c); bỏ qua nếu máy không có ruby', () {
      ProcessResult? probe;
      try {
        probe = Process.runSync('ruby', ['-v']);
      } on ProcessException {
        probe = null;
      }
      if (probe == null || probe.exitCode != 0) {
        markTestSkipped('máy không có ruby');
        return;
      }
      for (final path in [
        'ios/fastlane/Distribute.fastfile',
        'android/fastlane/Distribute.fastfile',
        'ios/fastlane/Pluginfile',
        'android/fastlane/Pluginfile',
      ]) {
        final result = Process.runSync('ruby', [
          '-c',
          nativeConfigFile(path).path,
        ]);
        expect(result.exitCode, 0, reason: '$path: ${result.stderr}');
      }
    });
  });

  group('Gradle: SDK App Distribution chỉ vào variant staging', () {
    final gradle = readNativeConfig(
      'android/app-distribution.gradle.kts.example',
    );
    final code = [
      for (final l in gradle.split('\n'))
        if (!l.trimLeft().startsWith('//')) l,
    ].join('\n');

    test(
      'bản api cho mọi variant, bản đầy đủ chỉ qua stagingImplementation',
      () {
        expect(
          code,
          contains(
            'implementation("com.google.firebase:firebase-appdistribution-api:',
          ),
        );
        final full = RegExp(r'firebase-appdistribution:[^"]+')
            .allMatches(code)
            .toList();
        expect(full, hasLength(1));
        final line = code
            .split('\n')
            .singleWhere((l) => l.contains('firebase-appdistribution:'));
        expect(line.trim(), startsWith('"stagingImplementation"('));
      },
    );

    test('hai thư viện cùng một phiên bản', () {
      final versions = RegExp(r'firebase-appdistribution(?:-api)?:([^"]+)')
          .allMatches(code)
          .map((m) => m.group(1))
          .toSet();
      expect(versions, hasLength(1));
    });

    test('flavor staging khớp tên flavor của bài 50', () {
      final gradle50 = siblingFile(
        '50-flavors-build/native-config/customer/android/app/build.gradle.kts',
      );
      expect(gradle50, contains('create("staging")'));
    });
  });

  group('không có bí mật trong native-config', () {
    final files = [
      for (final f in Directory(
        '${labDir.path}/native-config',
      ).listSync(recursive: true).whereType<File>())
        f,
    ];

    test('có file để quét', () {
      expect(files.length, greaterThanOrEqualTo(8));
    });

    test(
      'không có khoá riêng, token OAuth, API key hay file .p8/.p12/.jks',
      () {
        final patterns = <String, RegExp>{
          'PEM': RegExp('-----BEGIN'),
          'private_key': RegExp('"private_key"'),
          'Google API key': RegExp('AIza[0-9A-Za-z_-]{20,}'),
          'OAuth access token': RegExp(r'ya29\.[0-9A-Za-z_-]{20,}'),
          'refresh token': RegExp(r'1//[0-9A-Za-z_-]{20,}'),
          'JSON service account': RegExp('"type"\\s*:\\s*"service_account"'),
        };
        for (final f in files) {
          final text = f.readAsStringSync();
          for (final e in patterns.entries) {
            expect(
              text,
              isNot(matches(e.value)),
              reason: '${f.path}: ${e.key}',
            );
          }
          expect(
            f.path,
            isNot(matches(RegExp(r'\.(p8|p12|jks|keystore|mobileprovision)$'))),
          );
        }
      },
    );

    test('mọi App ID Firebase trong native-config đều là giá trị giả', () {
      final id = RegExp(r'1:\d{6,}:(?:android|ios):[A-Za-z0-9-]+');
      var seen = 0;
      for (final f in files) {
        for (final m in id.allMatches(f.readAsStringSync())) {
          seen++;
          expect(m.group(0), startsWith('1:000000000000:'), reason: f.path);
          expect(m.group(0), contains('FAKE'), reason: f.path);
        }
      }
      expect(seen, 4);
    });

    test('email chỉ thuộc tên miền ví dụ', () {
      final email = RegExp(r'[A-Za-z0-9._%+-]+@([A-Za-z0-9.-]+\.[A-Za-z]{2,})');
      for (final f in files) {
        for (final m in email.allMatches(f.readAsStringSync())) {
          expect(
            m.group(1),
            anyOf('toinoi.example', 'example.com'),
            reason: '${f.path}: ${m.group(0)}',
          );
        }
      }
    });

    test('lab không có file service account hay danh sách UDID', () {
      final stray = [
        for (final f in Directory(
          labDir.path,
        ).listSync(recursive: true).whereType<File>())
          if (!f.path.contains('/.dart_tool/') &&
              RegExp(r'service-account.*\.json$|udids.*\.csv$')
                  .hasMatch(f.path))
            f.path,
      ];
      expect(stray, isEmpty);
    });
  });
}
