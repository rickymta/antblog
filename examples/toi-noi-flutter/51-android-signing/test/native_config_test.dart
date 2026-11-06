import 'dart:io';

import 'package:test/test.dart';
import 'package:toi_noi_51/gradle_signing_audit.dart';

void main() {
  final gradle = File('native-config/build.gradle.kts').readAsStringSync();

  test('build.gradle.kts của lab qua mọi kiểm tra ký bản release', () {
    expect(auditGradleSigning(gradle), isEmpty);
  });

  test('không có đường lui sang khoá debug cho bản release', () {
    expect(gradle, isNot(contains('signingConfigs.getByName("debug")')));
  });

  test('versionCode và versionName lấy từ pubspec qua flutter.*', () {
    expect(gradle, contains('versionCode = flutter.versionCode'));
    expect(gradle, contains('versionName = flutter.versionName'));
  });

  test('application ID và targetSdk theo quy ước', () {
    expect(gradle, contains('applicationId = "vn.toinoi.customer"'));
    expect(gradle, contains('targetSdk = flutter.targetSdkVersion'));
  });

  group('bộ kiểm bắt được cấu hình sai (mẫu cố ý hỏng)', () {
    // Đoạn mặc định của template Flutter: bản release ký bằng khoá debug.
    const flutterTemplateRelease = '''
    buildTypes {
        release {
            // TODO: Add your own signing config for the release build.
            signingConfig = signingConfigs.getByName("debug")
        }
    }
''';

    test('template mặc định bị báo đường lui ký debug', () {
      final problems = auditGradleSigning(flutterTemplateRelease);
      expect(problems, contains('có đường lui ký bản release bằng khoá debug'));
      expect(problems, contains('thiếu signingConfigs { create("release") }'));
    });

    test('mật khẩu viết thẳng trong Gradle bị báo', () {
      final bad = gradle.replaceFirst(
        'storePassword = keystoreProperties.getProperty("storePassword")',
        'storePassword = "hunter2"',
      );
      expect(
        auditGradleSigning(bad),
        contains('mật khẩu keystore viết thẳng trong file Gradle'),
      );
    });

    test('gõ cứng versionCode bị báo', () {
      final bad = gradle.replaceFirst(
        'versionCode = flutter.versionCode',
        'versionCode = 7',
      );
      expect(
        auditGradleSigning(bad),
        contains('versionCode phải lấy từ flutter.versionCode, không gõ số'),
      );
    });

    test('bỏ bước dừng sớm bị báo', () {
      final bad = gradle.replaceFirst(
        'dependsOn(verifyReleaseSigning)',
        '// bỏ',
      );
      expect(auditGradleSigning(bad).join(), contains('dừng sớm'));
    });

    test('chữ debug nằm trong comment thì không tính', () {
      final withComment =
          '$gradle\n// signingConfigs.getByName("debug") chỉ để minh hoạ\n';
      expect(auditGradleSigning(withComment), isEmpty);
    });
  });
}
