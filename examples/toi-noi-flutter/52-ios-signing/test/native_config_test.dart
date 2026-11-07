import 'dart:io';

import 'package:test/test.dart';
import 'package:toi_noi_52/toi_noi_52.dart';

import 'support.dart';

/// Máy viết bài không chạy được `xcodebuild archive`, nên file trong
/// `native-config/` chỉ được kiểm bằng cách đọc khoá và giá trị.
void main() {
  group('Runner.entitlements.example', () {
    final plist = configPlist('Runner.entitlements.example');

    test('có aps-environment cho Push Notifications', () {
      expect(plist['aps-environment'], 'development');
    });

    test('associated domains dùng tên miền giả toinoi.example', () {
      expect(plist['com.apple.developer.associated-domains'], [
        'applinks:toinoi.example',
      ]);
    });

    test('không khai khoá push sai tên', () {
      expect(plist, isNot(contains('com.apple.developer.push-notifications')));
    });
  });

  group('Info.plist.fragment.example', () {
    test('Background Modes nằm ở UIBackgroundModes, không ở entitlements', () {
      final info = configPlist('Info.plist.fragment.example');
      expect(info['UIBackgroundModes'], ['remote-notification']);
      expect(
        configPlist('Runner.entitlements.example'),
        isNot(contains('UIBackgroundModes')),
      );
    });
  });

  group('xcconfig theo flavor', () {
    test('dev: automatic, không ghim profile', () {
      final c = xcconfig('dev');
      expect(c['PRODUCT_BUNDLE_IDENTIFIER'], 'vn.toinoi.customer.dev');
      expect(c['CODE_SIGN_STYLE'], 'Automatic');
      expect(c, isNot(contains('PROVISIONING_PROFILE_SPECIFIER')));
    });

    test('staging: manual, profile ad hoc', () {
      final c = xcconfig('staging');
      expect(c['PRODUCT_BUNDLE_IDENTIFIER'], 'vn.toinoi.customer.staging');
      expect(c['CODE_SIGN_STYLE'], 'Manual');
      expect(c['CODE_SIGN_IDENTITY'], 'Apple Distribution');
      expect(
        c['PROVISIONING_PROFILE_SPECIFIER'],
        'ToiNoi Customer Staging AdHoc',
      );
    });

    test('prod: manual, profile App Store', () {
      final c = xcconfig('prod');
      expect(c['PRODUCT_BUNDLE_IDENTIFIER'], 'vn.toinoi.customer');
      expect(c['CODE_SIGN_STYLE'], 'Manual');
      expect(c['PROVISIONING_PROFILE_SPECIFIER'], 'ToiNoi Customer AppStore');
    });

    test('cả ba flavor cùng một team và cùng file entitlements', () {
      for (final flavor in ['dev', 'staging', 'prod']) {
        final c = xcconfig(flavor);
        expect(c['DEVELOPMENT_TEAM'], fakeTeamId, reason: flavor);
        expect(c['CODE_SIGN_ENTITLEMENTS'], 'Runner/Runner.entitlements');
      }
    });

    test('flavor manual luôn có profile; flavor automatic thì không', () {
      for (final flavor in ['dev', 'staging', 'prod']) {
        final c = xcconfig(flavor);
        final manual = c['CODE_SIGN_STYLE'] == 'Manual';
        expect(
          c['PROVISIONING_PROFILE_SPECIFIER'],
          manual ? isNotEmpty : isNull,
          reason: flavor,
        );
      }
    });
  });

  group('ExportOptions', () {
    final adhoc = configPlist('ExportOptions-adhoc.plist.example');
    final appStore = configPlist('ExportOptions-appstore.plist.example');

    test('ad hoc dùng tên hiện hành release-testing', () {
      expect(adhoc['method'], 'release-testing');
    });

    test('App Store dùng tên hiện hành app-store-connect', () {
      expect(appStore['method'], 'app-store-connect');
    });

    test('cả hai qua lint không có vấn đề', () {
      expect(lintExportOptions(adhoc, expectedTeamId: fakeTeamId), isEmpty);
      expect(lintExportOptions(appStore, expectedTeamId: fakeTeamId), isEmpty);
    });

    test('profile trong ExportOptions khớp xcconfig và đúng bundle ID', () {
      final staging = xcconfig('staging');
      expect(adhoc['provisioningProfiles'], {
        staging['PRODUCT_BUNDLE_IDENTIFIER']:
            staging['PROVISIONING_PROFILE_SPECIFIER'],
      });
      final prod = xcconfig('prod');
      expect(appStore['provisioningProfiles'], {
        prod['PRODUCT_BUNDLE_IDENTIFIER']:
            prod['PROVISIONING_PROFILE_SPECIFIER'],
      });
    });

    test('profile mẫu khớp tên và bundle ID mà ExportOptions trỏ tới', () {
      final adhocProfile = sample('staging-adhoc');
      final map = adhoc['provisioningProfiles']! as Map<String, Object?>;
      expect(map[adhocProfile.bundleId], adhocProfile.name);
      final storeProfile = sample('prod-appstore');
      final storeMap =
          appStore['provisioningProfiles']! as Map<String, Object?>;
      expect(storeMap[storeProfile.bundleId], storeProfile.name);
    });
  });

  group('lintExportOptions', () {
    Map<String, Object?> base() => {
      'method': 'release-testing',
      'teamID': fakeTeamId,
      'signingStyle': 'manual',
      'provisioningProfiles': {'vn.toinoi.customer.staging': 'P'},
    };

    List<String> codes(Map<String, Object?> plist, {String? team}) =>
        lintExportOptions(
          plist,
          expectedTeamId: team,
        ).map((i) => i.code).toList();

    test('tên cũ được gợi ý tên mới', () {
      for (final entry in deprecatedExportMethods.entries) {
        final issues = lintExportOptions(base()..['method'] = entry.key);
        expect(issues.single.code, 'deprecated-method');
        expect(issues.single.message, contains(entry.value));
      }
    });

    test('mọi tên cũ đều trỏ tới một tên hiện hành', () {
      expect(currentExportMethods, containsAll(deprecatedExportMethods.values));
      expect(
        currentExportMethods.intersection(deprecatedExportMethods.keys.toSet()),
        isEmpty,
      );
    });

    test('method lạ và method thiếu', () {
      expect(codes(base()..['method'] = 'adhoc'), ['unknown-method']);
      expect(codes(base()..remove('method')), ['missing-method']);
    });

    test('teamID thiếu, rỗng hoặc lệch xcconfig', () {
      expect(codes(base()..remove('teamID')), ['missing-team']);
      expect(codes(base()..['teamID'] = ''), ['missing-team']);
      expect(codes(base(), team: 'ZZZZZ99999'), ['team-mismatch']);
    });

    test('manual mà không có profile', () {
      expect(codes(base()..remove('provisioningProfiles')), [
        'manual-without-profiles',
      ]);
      expect(codes(base()..['provisioningProfiles'] = <String, Object?>{}), [
        'manual-without-profiles',
      ]);
    });

    test('automatic không cần provisioningProfiles', () {
      final plist = base()
        ..['signingStyle'] = 'automatic'
        ..remove('provisioningProfiles');
      expect(codes(plist), isEmpty);
    });
  });

  group('không có bí mật thật', () {
    final files = [
      ...Directory('native-config').listSync(recursive: true),
      ...Directory('samples').listSync(recursive: true),
    ].whereType<File>().toList();

    test('có file để quét', () {
      expect(files.length, greaterThanOrEqualTo(11));
    });

    test('mọi file mẫu của native-config có đuôi .example', () {
      for (final f in files.where((f) => f.path.contains('native-config'))) {
        expect(f.path, endsWith('.example'));
      }
    });

    test('không có khoá riêng, chứng chỉ hay file profile nhị phân', () {
      for (final f in files) {
        final text = f.readAsStringSync();
        expect(text, isNot(contains('-----BEGIN')), reason: f.path);
        expect(f.path, isNot(matches(r'\.(p12|p8|cer|mobileprovision)$')));
      }
    });

    test('mọi Team ID trong App ID đều là ABCDE12345 giả', () {
      final appId = RegExp(r'\b([A-Z0-9]{10})\.vn\.toinoi\b');
      for (final f in files) {
        for (final m in appId.allMatches(f.readAsStringSync())) {
          expect(m.group(1), fakeTeamId, reason: f.path);
        }
      }
    });

    test('mọi UDID mẫu là số giả toàn số 0', () {
      final udid = RegExp(r'\b0000[0-9A-F]{4}-([0-9A-F]{16})\b');
      for (final f in files) {
        for (final m in udid.allMatches(f.readAsStringSync())) {
          expect(m.group(1), matches(r'^0{15}[0-9A-F]$'), reason: f.path);
        }
      }
    });
  });
}
