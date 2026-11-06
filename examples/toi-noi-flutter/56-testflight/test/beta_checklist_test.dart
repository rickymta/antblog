import 'package:test/test.dart';
import 'package:toi_noi_56/toi_noi_56.dart';

import 'support.dart';

Matcher hasIssue(String code, {String? subject, Severity? severity}) =>
    predicate<List<CheckIssue>>(
      (issues) => issues.any(
        (i) =>
            i.code == code &&
            (subject == null || i.subject == subject) &&
            (severity == null || i.severity == severity),
      ),
      'có phát hiện $code${subject == null ? '' : ' về $subject'}',
    );

void main() {
  group('Info.plist mẫu', () {
    test('app khách đạt, không cảnh báo nào', () {
      final issues = customerChecklist(last: '42').check(
        readPlist(customerPlist),
        entitlements: readPlist(entitlementsFile),
      );
      expect(issues, isEmpty);
    });

    test('app tài xế đạt, không cảnh báo nào', () {
      final issues = driverChecklist(last: '42').check(
        readPlist(driverPlist),
        entitlements: readPlist(entitlementsFile),
      );
      expect(issues, isEmpty);
    });

    test(
      'không đòi khoá mô tả cho thông báo: hộp thoại push không có khoá đó',
      () {
        final plist = readPlist(customerPlist);
        expect(
          plist.keys.where((k) => k.toLowerCase().contains('notification')),
          isEmpty,
        );
        expect(customerChecklist().check(plist).ok, isTrue);
      },
    );
  });

  group('mô tả quyền vị trí', () {
    test('thiếu WhenInUse là lỗi, cho cả hai app', () {
      for (final (checklist, path) in [
        (customerChecklist(), customerPlist),
        (driverChecklist(), driverPlist),
      ]) {
        final plist = editable(path)
          ..remove('NSLocationWhenInUseUsageDescription');
        expect(
          checklist.check(plist),
          hasIssue(
            'purpose-missing',
            subject: 'NSLocationWhenInUseUsageDescription',
            severity: Severity.error,
          ),
        );
      }
    });

    test('chuỗi rỗng, toàn khoảng trắng hoặc sai kiểu là lỗi', () {
      for (final bad in <Object?>['', '   \n', 42, true]) {
        final plist = editable(customerPlist)
          ..['NSLocationWhenInUseUsageDescription'] = bad;
        expect(
          customerChecklist().check(plist),
          hasIssue('purpose-blank', severity: Severity.error),
          reason: 'giá trị $bad',
        );
      }
    });

    test('dấu vết giữ chỗ chỉ là cảnh báo, build vẫn qua', () {
      for (final text in [
        'TODO viết sau',
        'Lorem ipsum',
        r'$(APP_NAME) cần vị trí',
      ]) {
        final plist = editable(customerPlist)
          ..['NSLocationWhenInUseUsageDescription'] = text;
        final issues = customerChecklist().check(plist);
        expect(
          issues,
          hasIssue('purpose-placeholder', severity: Severity.warning),
          reason: text,
        );
        expect(issues.ok, isTrue);
      }
    });

    test('app khách khai quyền nền bị cảnh báo, bằng Always hoặc bằng background mode', () {
      final withAlways = editable(customerPlist)
        ..['NSLocationAlwaysAndWhenInUseUsageDescription'] = 'Cần vị trí nền';
      expect(
        customerChecklist().check(withAlways),
        hasIssue('customer-background-location', severity: Severity.warning),
      );

      final withMode = editable(customerPlist)
        ..['UIBackgroundModes'] = ['remote-notification', 'location'];
      expect(
        customerChecklist().check(withMode),
        hasIssue('customer-background-location', severity: Severity.warning),
      );
    });

    test('app tài xế thiếu UIBackgroundModes location là lỗi', () {
      final plist = editable(driverPlist)
        ..['UIBackgroundModes'] = ['remote-notification'];
      expect(
        driverChecklist().check(plist),
        hasIssue(
          'background-mode-missing',
          subject: 'location',
          severity: Severity.error,
        ),
      );
    });

    test('app tài xế không có UIBackgroundModes: thiếu cả location lẫn remote-notification', () {
      final plist = editable(driverPlist)..remove('UIBackgroundModes');
      final issues = driverChecklist().check(plist);
      expect(issues, hasIssue('background-mode-missing', subject: 'location'));
      expect(
        issues,
        hasIssue('background-mode-missing', subject: 'remote-notification'),
      );
    });

    test('khoá Always của tài xế, nếu có, không được để trống', () {
      final plist = editable(driverPlist)
        ..['NSLocationAlwaysAndWhenInUseUsageDescription'] = ' ';
      expect(
        driverChecklist().check(plist),
        hasIssue(
          'purpose-blank',
          subject: 'NSLocationAlwaysAndWhenInUseUsageDescription',
        ),
      );
    });

    test(
      'tài xế không khai Always vẫn qua nếu có background mode location',
      () {
        final plist = editable(driverPlist)
          ..remove('NSLocationAlwaysAndWhenInUseUsageDescription');
        expect(driverChecklist().check(plist).ok, isTrue);
      },
    );
  });

  group('push', () {
    test('thiếu remote-notification trong UIBackgroundModes là lỗi', () {
      final plist = editable(customerPlist)..['UIBackgroundModes'] = <String>[];
      expect(
        customerChecklist().check(plist),
        hasIssue(
          'background-mode-missing',
          subject: 'remote-notification',
          severity: Severity.error,
        ),
      );
    });

    test('UIBackgroundModes sai kiểu bị báo, không ném lỗi', () {
      for (final bad in <Object?>[
        'location',
        1,
        <Object?>['location', 1],
      ]) {
        final plist = editable(driverPlist)..['UIBackgroundModes'] = bad;
        expect(
          driverChecklist().check(plist),
          hasIssue('background-modes-type', severity: Severity.error),
          reason: '$bad',
        );
      }
    });

    test(
      'entitlements thiếu aps-environment là lỗi; không truyền thì không kiểm',
      () {
        final plist = readPlist(customerPlist);
        expect(
          customerChecklist().check(plist, entitlements: <String, Object?>{}),
          hasIssue('push-entitlement-missing', subject: 'aps-environment'),
        );
        expect(
          customerChecklist().check(
            plist,
            entitlements: {'aps-environment': 'production'},
          ),
          isEmpty,
        );
        expect(customerChecklist().check(plist), isEmpty);
      },
    );
  });

  group('export compliance', () {
    test('thiếu khoá: cảnh báo, và lời cảnh báo nhắc Missing Compliance', () {
      final plist = editable(customerPlist)
        ..remove('ITSAppUsesNonExemptEncryption');
      final issues = customerChecklist().check(plist);
      expect(
        issues,
        hasIssue(
          'encryption-key-missing',
          subject: 'ITSAppUsesNonExemptEncryption',
          severity: Severity.warning,
        ),
      );
      expect(issues.single.message, contains('Missing Compliance'));
      expect(issues.ok, isTrue);
    });

    test('chuỗi "NO" thay vì <false/> là lỗi', () {
      for (final bad in <Object?>['NO', 'false', 0]) {
        final plist = editable(customerPlist)
          ..['ITSAppUsesNonExemptEncryption'] = bad;
        expect(
          customerChecklist().check(plist),
          hasIssue('encryption-key-type', severity: Severity.error),
          reason: '$bad',
        );
      }
    });

    test('true phải kèm mã compliance; có mã thì qua', () {
      final plist = editable(customerPlist)
        ..['ITSAppUsesNonExemptEncryption'] = true;
      expect(
        customerChecklist().check(plist),
        hasIssue('encryption-code-missing', severity: Severity.error),
      );
      plist['ITSEncryptionExportComplianceCode'] = 'MA_CUA_APPLE_CAP';
      expect(customerChecklist().check(plist), isEmpty);
    });

    test('false mà vẫn có mã compliance: cảnh báo mâu thuẫn', () {
      final plist = editable(customerPlist)
        ..['ITSEncryptionExportComplianceCode'] = 'MA_CUA_APPLE_CAP';
      final issues = customerChecklist().check(plist);
      expect(
        issues,
        hasIssue('encryption-code-unused', severity: Severity.warning),
      );
      expect(issues.ok, isTrue);
    });
  });

  group('CFBundleShortVersionString và CFBundleVersion', () {
    test('không có biến build thì không đọc được: lỗi cho cả hai khoá', () {
      final issues = customerChecklist(settings: const {})
          .check(readPlist(customerPlist));
      expect(
        issues,
        hasIssue('version-unresolved', subject: 'CFBundleShortVersionString'),
      );
      expect(
        issues,
        hasIssue('version-unresolved', subject: 'CFBundleVersion'),
      );
    });

    test('biến có giá trị rỗng cũng coi là chưa cấp', () {
      final issues = customerChecklist(
        settings: const {
          'FLUTTER_BUILD_NAME': '1.4.0',
          'FLUTTER_BUILD_NUMBER': '',
        },
      ).check(readPlist(customerPlist));
      expect(
        issues,
        hasIssue('version-unresolved', subject: 'CFBundleVersion'),
      );
    });

    test('khoá thiếu hoặc không phải chuỗi là lỗi version-missing', () {
      final plist = editable(customerPlist)
        ..remove('CFBundleVersion')
        ..['CFBundleShortVersionString'] = 7;
      final issues = customerChecklist().check(plist);
      expect(issues, hasIssue('version-missing', subject: 'CFBundleVersion'));
      expect(
        issues,
        hasIssue('version-missing', subject: 'CFBundleShortVersionString'),
      );
    });

    test('version phải đúng ba số: 1.4, 1.4.0.1, 1.x.0 đều sai', () {
      for (final name in ['1.4', '1.4.0.1', '1.x.0', '', 'v1.4.0']) {
        final settings = {...settings143, 'FLUTTER_BUILD_NAME': name};
        final issues = customerChecklist(settings: settings)
            .check(readPlist(customerPlist));
        expect(
          issues.any((i) => i.subject == 'CFBundleShortVersionString'),
          isTrue,
          reason: '"$name"',
        );
      }
    });

    test('build number sai dạng: chữ, rỗng, bốn thành phần, số quá lớn', () {
      for (final build in [
        'abc',
        '4a',
        '-1',
        '1..2',
        '1.2.3.4',
        '99999999999999999999',
      ]) {
        final settings = {...settings143, 'FLUTTER_BUILD_NUMBER': build};
        expect(
          customerChecklist(settings: settings).check(readPlist(customerPlist)),
          hasIssue('build-format', subject: 'CFBundleVersion'),
          reason: build,
        );
      }
    });

    test('build number một đến ba thành phần đều hợp lệ', () {
      for (final build in ['1', '43', '1.2', '1.2.3', '043']) {
        final settings = {...settings143, 'FLUTTER_BUILD_NUMBER': build};
        expect(
          customerChecklist(settings: settings).check(readPlist(customerPlist)),
          isEmpty,
          reason: build,
        );
      }
    });

    test(
      'phải lớn hơn bản đã tải lên: bằng và nhỏ hơn bị lỗi, lớn hơn qua',
      () {
        final plist = readPlist(customerPlist);
        expect(
          customerChecklist(last: '42').check(plist),
          isEmpty,
          reason: '43 > 42',
        );
        expect(
          customerChecklist(last: '43').check(plist),
          hasIssue('build-not-increasing', severity: Severity.error),
          reason: 'bằng nhau',
        );
        expect(
          customerChecklist(last: '44').check(plist),
          hasIssue('build-not-increasing'),
          reason: 'nhỏ hơn',
        );
      },
    );

    test('so theo số, không theo chữ: 10 lớn hơn 9, 1.10 lớn hơn 1.9', () {
      for (final (now, last, ok) in [
        ('10', '9', true),
        ('9', '10', false),
        ('1.10', '1.9', true),
        ('1.9', '1.10', false),
        ('2', '1.99.99', true),
        ('1.0.0', '1', false),
        ('043', '43', false),
      ]) {
        final settings = {...settings143, 'FLUTTER_BUILD_NUMBER': now};
        final issues = customerChecklist(
          settings: settings,
          last: last,
        ).check(readPlist(customerPlist));
        expect(
          issues.any((i) => i.code == 'build-not-increasing'),
          !ok,
          reason: '$now so với $last',
        );
      }
    });

    test('chưa có bản nào tải lên thì không kiểm tăng dần', () {
      expect(customerChecklist().check(readPlist(customerPlist)), isEmpty);
    });

    test('lastUploadedBuild sai dạng là lỗi lập trình: ném ArgumentError', () {
      expect(
        () => customerChecklist(last: 'abc').check(readPlist(customerPlist)),
        throwsArgumentError,
      );
    });

    test('Info.plist ghi sẵn số thật thay vì biến vẫn được kiểm', () {
      final plist = editable(customerPlist)
        ..['CFBundleVersion'] = '41'
        ..['CFBundleShortVersionString'] = '1.4.0';
      expect(
        customerChecklist(last: '41', settings: const {}).check(plist),
        hasIssue('build-not-increasing'),
      );
    });
  });

  group('parseBuild, compareBuilds, resolveBuildSettings', () {
    test('parseBuild đệm số 0 và từ chối dạng sai', () {
      expect(parseBuild('7'), [7, 0, 0]);
      expect(parseBuild('1.2'), [1, 2, 0]);
      expect(parseBuild('1.2.3'), [1, 2, 3]);
      expect(parseBuild(''), isNull);
      expect(parseBuild('1.2.3.4'), isNull);
      expect(parseBuild(' 1'), isNull);
    });

    test('compareBuilds theo từng thành phần', () {
      expect(compareBuilds([1, 2, 3], [1, 2, 3]), 0);
      expect(compareBuilds([1, 2, 4], [1, 2, 3]), greaterThan(0));
      expect(compareBuilds([0, 9, 9], [1, 0, 0]), lessThan(0));
    });

    test('resolveBuildSettings thay cả hai dạng cú pháp và báo biến thiếu', () {
      const s = {'A': '1', 'B': '2'};
      expect(resolveBuildSettings(r'$(A).${B}', s), '1.2');
      expect(resolveBuildSettings('thuần', s), 'thuần');
      expect(resolveBuildSettings(r'$(A).$(C)', s), isNull);
    });
  });

  test('CheckIssues.ok bỏ qua cảnh báo và đếm lỗi', () {
    const warn = CheckIssue('w', Severity.warning, 's', 'm');
    const err = CheckIssue('e', Severity.error, 's', 'm');
    expect([warn].ok, isTrue);
    expect([warn, err].ok, isFalse);
    expect([warn, err].errors, [err]);
  });
}
