import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';
import 'package:toi_noi_56/toi_noi_56.dart';

import 'support.dart';

/// Máy viết bài không có tài khoản Apple Developer và không chạy được
/// `xcodebuild`, nên file trong `native-config/` chỉ được kiểm bằng cách đọc
/// khoá và giá trị.
void main() {
  group('Info.plist mẫu', () {
    test('cả hai app dùng biến build của Flutter cho version và build', () {
      for (final path in [customerPlist, driverPlist]) {
        final plist = readPlist(path);
        expect(plist['CFBundleShortVersionString'], r'$(FLUTTER_BUILD_NAME)');
        expect(plist['CFBundleVersion'], r'$(FLUTTER_BUILD_NUMBER)');
        expect(plist['ITSAppUsesNonExemptEncryption'], false);
      }
    });

    test('app khách: chỉ quyền khi dùng app, push nền, không location nền', () {
      final plist = readPlist(customerPlist);
      expect(plist, contains('NSLocationWhenInUseUsageDescription'));
      expect(
        plist,
        isNot(contains('NSLocationAlwaysAndWhenInUseUsageDescription')),
      );
      expect(plist['UIBackgroundModes'], ['remote-notification']);
    });

    test(
      'app tài xế: background mode location, không khai quyền Always (bài 38)',
      () {
        final plist = readPlist(driverPlist);
        expect(plist, contains('NSLocationWhenInUseUsageDescription'));
        expect(
          plist,
          isNot(contains('NSLocationAlwaysAndWhenInUseUsageDescription')),
        );
        expect(plist['UIBackgroundModes'], ['location', 'remote-notification']);
      },
    );
  });

  group('Runner.entitlements.example', () {
    test('có aps-environment cho Push Notifications', () {
      expect(readPlist(entitlementsFile)['aps-environment'], 'production');
    });
  });

  group('ExportOptions-staging-testflight.plist.example', () {
    final plist = readPlist(
      'native-config/ios/ExportOptions-staging-testflight.plist.example',
    );

    test(
      'method app-store-connect, không phải tên cũ app-store hay ad-hoc',
      () {
        expect(plist['method'], 'app-store-connect');
      },
    );

    test(
      'bundle ID staging ghim profile loại App Store, không phải ad hoc',
      () {
        final profiles = plist['provisioningProfiles']! as Map<String, Object?>;
        expect(profiles.keys, ['vn.toinoi.customer.staging']);
        final name = profiles['vn.toinoi.customer.staging']! as String;
        expect(name, contains('AppStore'));
        expect(name, isNot(contains('AdHoc')));
      },
    );

    test('ký thủ công bằng Apple Distribution; Team ID là giá trị giả', () {
      expect(plist['signingStyle'], 'manual');
      expect(plist['signingCertificate'], 'Apple Distribution');
      expect(plist['teamID'], 'ABCDE12345');
    });
  });

  group('test-information.json.example', () {
    final source = readText('native-config/test-information.json.example');

    test('chưa điền thì checker đỏ đúng các trường cần điền', () {
      final issues = TestInformation.fromJson(source).validateForExternal();
      expect(
        {for (final i in issues) i.field},
        {
          'feedbackEmail',
          'contactFirstName',
          'contactLastName',
          'contactEmail',
          'contactPhone',
          'demoAccountName',
          'demoAccountPassword',
        },
      );
      expect(issues.every((i) => i.code == 'info-unfilled'), isTrue);
    });

    test('mô tả beta đã viết sẵn, và nói rõ dữ liệu là dữ liệu thử', () {
      final t = TestInformation.fromJson(source);
      expect(TestInformation.isUnfilled(t.betaAppDescription), isFalse);
      expect(t.betaAppDescription, contains('dữ liệu thử'));
    });

    test('điền đủ các trường giữ chỗ thì hết lỗi', () {
      final map = jsonDecode(source) as Map<String, Object?>;
      for (final key in map.keys.toList()) {
        final v = map[key];
        if (v is String && TestInformation.isUnfilled(v)) {
          map[key] = key.contains('Email')
              ? 'x@toinoi.example'
              : key.contains('Phone')
              ? '+84000000000'
              : 'giá-trị-thử';
        }
      }
      expect(
        TestInformation.fromJson(jsonEncode(map)).validateForExternal(),
        isEmpty,
      );
    });

    test('mật khẩu demo trong repo luôn là giá trị giữ chỗ', () {
      final t = TestInformation.fromJson(source);
      expect(TestInformation.isUnfilled(t.demoAccountPassword), isTrue);
    });
  });

  group('Fastfile.beta-external.example', () {
    const path = 'native-config/ios/fastlane/Fastfile.beta-external.example';
    final text = readText(path);
    // Bỏ dòng chú thích: chúng được phép nhắc tên tham số mà code không dùng.
    final code = text
        .split('\n')
        .where((l) => !l.trimLeft().startsWith('#'))
        .join('\n');

    test(
      'gọi upload_to_testflight với nhóm external và Beta App Review info',
      () {
        expect(text, contains('upload_to_testflight('));
        expect(text, contains('distribute_external: true'));
        expect(text, contains('groups: [group]'));
        expect(text, contains('beta_app_review_info: review_info'));
        expect(text, contains('beta_app_description: beta_description'));
        expect(text, contains('beta_app_feedback_email: feedback_email'));
        expect(text, contains('demo_account_required: true'));
      },
    );

    test('không đặt skip_waiting_for_build_processing: true, vì pilot không phân phối được khi đó', () {
      expect(code, isNot(contains('skip_waiting_for_build_processing')));
    });

    test(
      'không đặt uses_non_exempt_encryption: Info.plist đã khai khoá đó',
      () {
        expect(code, isNot(contains('uses_non_exempt_encryption')));
      },
    );

    test('mọi giá trị bí mật đi qua env!, không có chuỗi gõ tay', () {
      for (final key in [
        'contact_email',
        'contact_first_name',
        'contact_last_name',
        'contact_phone',
        'demo_account_name',
        'demo_account_password',
      ]) {
        expect(
          RegExp('$key: env!\\("BETA_[A-Z_]+"\\)').hasMatch(text),
          isTrue,
          reason: key,
        );
      }
      expect(text, contains('api_key: asc_api_key'));
    });

    test(
      'mọi biến env! của file đều được liệt kê trong phần chú thích đầu file',
      () {
        final header = text
            .split('\n')
            .takeWhile((l) => l.startsWith('#') || l.trim().isEmpty)
            .join('\n');
        final used = RegExp(r'env!\("([A-Z_]+)"\)')
            .allMatches(code)
            .map((m) => m.group(1)!)
            .toSet();
        expect(used, hasLength(8));
        for (final name in used) {
          expect(header, contains(name), reason: name);
        }
        expect(header, contains('import "BetaExternal.fastfile"'));
      },
    );

    test('dừng sớm khi thiếu tham số, trước bước tải lên', () {
      final upload = text.indexOf('upload_to_testflight(');
      for (final guard in [
        'Cần ipa:',
        'Cần group:',
        'Cần changelog_file:',
        'env!("BETA_APP_DESCRIPTION")',
        'env!("BETA_DEMO_PASSWORD")',
      ]) {
        final at = text.indexOf(guard);
        expect(at, greaterThan(-1), reason: guard);
        expect(at, lessThan(upload), reason: guard);
      }
    });

    test('ruby -c báo Syntax OK (bỏ qua nếu máy không có ruby)', () {
      final ProcessResult result;
      try {
        result = Process.runSync('ruby', ['-c', path]);
      } on ProcessException {
        markTestSkipped('Máy không có ruby');
        return;
      }
      expect(result.exitCode, 0, reason: '${result.stderr}');
      expect(result.stdout, contains('Syntax OK'));
    });
  });

  group('không có bí mật thật trong native-config/', () {
    final files = Directory('native-config')
        .listSync(recursive: true)
        .whereType<File>()
        .toList();

    test('có file để quét', () {
      expect(files.length, greaterThanOrEqualTo(6));
    });

    test('không có khoá riêng, chứng chỉ hay token dạng nhận ra được', () {
      final patterns = {
        'khoá riêng PEM': RegExp('-----BEGIN [A-Z ]*PRIVATE KEY-----'),
        'chứng chỉ PEM': RegExp('-----BEGIN CERTIFICATE-----'),
        'token JWT': RegExp(r'eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.'),
      };
      for (final f in files) {
        final text = f.readAsStringSync();
        for (final e in patterns.entries) {
          expect(
            e.value.hasMatch(text),
            isFalse,
            reason: '${f.path}: ${e.key}',
          );
        }
      }
    });

    test('file mẫu chứa giá trị cần điền có đuôi .example', () {
      for (final f in files) {
        final name = f.uri.pathSegments.last;
        if (name == 'Info.plist') continue; // không có giá trị cần điền
        expect(name, endsWith('.example'), reason: f.path);
      }
    });
  });
}
