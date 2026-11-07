import 'dart:convert';

import 'package:test/test.dart';
import 'package:toi_noi_53/release_plan.dart'
    show AppKind, Flavor, applicationId;
import 'package:toi_noi_55/toi_noi_55.dart';

import 'support.dart';

/// Bản sao của file mẫu sau khi [change] sửa cây JSON; dùng cho ca sai.
String mutatedConfig(void Function(Map<String, Object?> root) change) {
  final root = jsonDecode(
    readNativeConfig('app-distribution.example.json'),
  ) as Map<String, Object?>;
  change(root);
  return jsonEncode(root);
}

Map<String, Object?> at(Map<String, Object?> m, String key) =>
    m[key]! as Map<String, Object?>;

void main() {
  final config = DistributionConfig.parse(
    readNativeConfig('app-distribution.example.json'),
  );

  group('file mẫu', () {
    test('có đúng 4 mục: hai app, hai nền tảng, chỉ staging', () {
      expect(config.targets, hasLength(4));
      expect({for (final t in config.targets) t.app}, AppKind.values.toSet());
      expect({
        for (final t in config.targets) t.platform,
      }, DistributionPlatform.values.toSet());
    });

    test('App ID là giá trị giả rõ ràng, số dự án toàn số 0', () {
      for (final t in config.targets) {
        expect(t.isPlaceholder, isTrue, reason: t.firebaseAppId);
        expect(t.firebaseAppId, startsWith('1:000000000000:'));
        expect(t.firebaseAppId, contains('FAKE'));
        expect(t.firebaseAppId, endsWith('REPLACE-ME'));
      }
    });

    test('App ID của app khách khớp với file .example của bài 50', () {
      final customerAndroid = config.target(
        AppKind.customer,
        Flavor.staging,
        DistributionPlatform.android,
      );
      final customerIos = config.target(
        AppKind.customer,
        Flavor.staging,
        DistributionPlatform.ios,
      );
      final android50 = siblingFile(
        '50-flavors-build/native-config/customer/android/app/src/staging/google-services.json.example',
      );
      final ios50 = siblingFile(
        '50-flavors-build/native-config/customer/ios/Runner/Firebase/staging/GoogleService-Info.plist.example',
      );
      expect(android50, contains(customerAndroid.firebaseAppId));
      expect(ios50, contains(customerIos.firebaseAppId));
    });

    test('Application ID là giá trị của curriculum §3.2', () {
      for (final t in config.targets) {
        expect(t.applicationId, applicationId(t.app, Flavor.staging));
        expect(t.applicationId, endsWith('.staging'));
      }
      expect(
        config
            .target(
              AppKind.driver,
              Flavor.staging,
              DistributionPlatform.android,
            )
            .applicationId,
        'vn.toinoi.driver.staging',
      );
    });

    test('nhóm tester nội bộ là qa-noi-bo', () {
      expect(config.groupAlias, 'qa-noi-bo');
      for (final t in config.targets) {
        expect(t.groupAlias, 'qa-noi-bo');
      }
    });
  });

  group('chỉ staging được phân phối', () {
    test('target() từ chối dev và prod, nói rõ vì sao', () {
      for (final flavor in [Flavor.dev, Flavor.prod]) {
        expect(
          () =>
              config.target(AppKind.customer, flavor, DistributionPlatform.ios),
          throwsA(
            isA<DistributionException>().having(
              (e) => e.message,
              'message',
              contains(flavor.name),
            ),
          ),
          reason: flavor.name,
        );
      }
    });

    test('file cấu hình có khoá dev hoặc prod bị từ chối khi đọc', () {
      for (final flavor in ['dev', 'prod']) {
        final source = mutatedConfig((root) {
          final customer = at(at(root, 'apps'), 'customer');
          customer[flavor] = customer['staging'];
        });
        expect(
          () => DistributionConfig.parse(source),
          throwsA(
            isA<DistributionException>().having(
              (e) => e.message,
              'message',
              allOf(contains('chỉ flavor staging'), contains(flavor)),
            ),
          ),
          reason: flavor,
        );
      }
    });

    test('app chưa khai báo nền tảng thì target() báo rõ', () {
      final onlyAndroid = DistributionConfig.parse(
        mutatedConfig((root) {
          final staging = at(at(at(root, 'apps'), 'driver'), 'staging');
          staging.remove('ios');
        }),
      );
      expect(
        () => onlyAndroid.target(
          AppKind.driver,
          Flavor.staging,
          DistributionPlatform.ios,
        ),
        throwsA(isA<DistributionException>()),
      );
    });
  });

  group('kiểm tra khi đọc cấu hình', () {
    void expectRejected(
      void Function(Map<String, Object?> root) change,
      String fragment,
    ) {
      expect(
        () => DistributionConfig.parse(mutatedConfig(change)),
        throwsA(
          isA<DistributionException>().having(
            (e) => e.message,
            'message',
            contains(fragment),
          ),
        ),
        reason: fragment,
      );
    }

    test('JSON hỏng', () {
      expect(
        () => DistributionConfig.parse('{'),
        throwsA(isA<DistributionException>()),
      );
    });

    test('App ID sai dạng hoặc sai nền tảng', () {
      Map<String, Object?> android(Map<String, Object?> root) =>
          at(at(at(at(root, 'apps'), 'customer'), 'staging'), 'android');
      expectRejected(
        (r) => android(r)['appId'] = 'khong-phai-app-id',
        'không đúng dạng',
      );
      expectRejected(
        (r) => android(r)['appId'] = '1:000000000000:ios:FAKE-STAGING',
        'không đúng dạng',
      );
    });

    test('App ID thuộc hai dự án khác nhau', () {
      expectRejected(
        (r) =>
            at(at(at(at(r, 'apps'), 'customer'), 'staging'), 'ios')['appId'] =
                '1:111111111111:ios:FAKE-STAGING-REPLACE-ME',
        'dự án khác',
      );
    });

    test('Application ID lệch với bảng của curriculum', () {
      expectRejected(
        (r) => at(
          at(at(at(r, 'apps'), 'customer'), 'staging'),
          'android',
        )['packageName'] = 'vn.toinoi.customer',
        'curriculum §3.2',
      );
    });

    test('alias nhóm sai dạng', () {
      for (final bad in ['QA Nội Bộ', 'qa_noi_bo', '-qa', 'qa-', '']) {
        expect(
          () => DistributionConfig.parse(
            mutatedConfig(
              (r) => at(at(r, 'testerGroups'), 'internal')['alias'] = bad,
            ),
          ),
          throwsA(isA<DistributionException>()),
          reason: '"$bad"',
        );
      }
    });

    test('app hoặc nền tảng lạ, và apps rỗng', () {
      expectRejected(
        (r) => at(r, 'apps')['admin'] = <String, Object?>{},
        'admin',
      );
      expectRejected(
        (r) => at(at(at(r, 'apps'), 'customer'), 'staging')['web'] =
            <String, Object?>{},
        'web',
      );
      expectRejected((r) => r['apps'] = <String, Object?>{}, 'không có mục');
    });
  });

  group('cliArguments: tham số của firebase appdistribution:distribute', () {
    final target = config.target(
      AppKind.customer,
      Flavor.staging,
      DistributionPlatform.android,
    );

    test('đúng lệnh, cờ và thứ tự theo trang CLI của App Distribution', () {
      expect(
        config.cliArguments(
          target,
          binaryPath: 'app-staging-release.apk',
          releaseNotesFile: 'notes.txt',
        ),
        [
          'appdistribution:distribute',
          'app-staging-release.apk',
          '--app',
          '1:000000000000:android:FAKE-STAGING-REPLACE-ME',
          '--groups',
          'qa-noi-bo',
          '--release-notes-file',
          'notes.txt',
        ],
      );
    });

    test('Android nhận apk và aab, iOS chỉ nhận ipa', () {
      final ios = config.target(
        AppKind.customer,
        Flavor.staging,
        DistributionPlatform.ios,
      );
      for (final file in ['a.apk', 'a.AAB']) {
        expect(
          () => config.cliArguments(
            target,
            binaryPath: file,
            releaseNotesFile: 'n',
          ),
          returnsNormally,
          reason: file,
        );
      }
      expect(
        () => config.cliArguments(
          target,
          binaryPath: 'a.ipa',
          releaseNotesFile: 'n',
        ),
        throwsA(isA<DistributionException>()),
      );
      expect(
        () => config.cliArguments(
          ios,
          binaryPath: 'a.apk',
          releaseNotesFile: 'n',
        ),
        throwsA(isA<DistributionException>()),
      );
      expect(
        () => config.cliArguments(
          ios,
          binaryPath: 'Runner.ipa',
          releaseNotesFile: 'n',
        ),
        returnsNormally,
      );
    });

    test('không có cờ xác thực: không --token, không FIREBASE_TOKEN', () {
      final args = config.cliArguments(
        target,
        binaryPath: 'a.apk',
        releaseNotesFile: 'n',
      );
      expect(args, isNot(contains('--token')));
      expect(args.join(' '), isNot(contains('FIREBASE_TOKEN')));
    });
  });
}
