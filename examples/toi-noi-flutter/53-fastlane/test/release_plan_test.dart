import 'package:test/test.dart';
import 'package:toi_noi_53/release_plan.dart';

void main() {
  group('applicationId theo curriculum §3.2', () {
    test('sáu cặp app và flavor', () {
      expect(
        {
          for (final app in AppKind.values)
            for (final flavor in Flavor.values)
              '${app.name}/${flavor.name}': applicationId(app, flavor),
        },
        {
          'customer/dev': 'vn.toinoi.customer.dev',
          'customer/staging': 'vn.toinoi.customer.staging',
          'customer/prod': 'vn.toinoi.customer',
          'driver/dev': 'vn.toinoi.driver.dev',
          'driver/staging': 'vn.toinoi.driver.staging',
          'driver/prod': 'vn.toinoi.driver',
        },
      );
    });
  });

  group('lanesFor', () {
    test('dev không có lane: không đi qua cửa hàng', () {
      expect(lanesFor(Flavor.dev), isEmpty);
    });

    test('staging và prod cùng đẩy lên TestFlight và Play internal', () {
      for (final flavor in [Flavor.staging, Flavor.prod]) {
        final lanes = lanesFor(flavor);
        expect(lanes.map((s) => '${s.platform.name} ${s.lane}'), [
          'ios beta',
          'android internal',
        ], reason: flavor.name);
      }
    });

    test('track của Play luôn là internal, lane iOS không có track', () {
      for (final flavor in Flavor.values) {
        for (final spec in lanesFor(flavor)) {
          expect(
            spec.track,
            spec.platform == StorePlatform.android ? 'internal' : isNull,
          );
        }
      }
    });

    test(
      'lane quản trị sync_certs không nằm trong lane phân phối của flavor nào',
      () {
        for (final flavor in Flavor.values) {
          expect(
            lanesFor(flavor).map((s) => s.lane),
            isNot(contains('sync_certs')),
          );
        }
      },
    );
  });

  group('requiredEnv và missingEnv', () {
    test('iOS cần API key và match; Android chỉ cần khoá Play', () {
      expect(requiredEnv(StorePlatform.ios), {
        'ASC_KEY_ID',
        'ASC_ISSUER_ID',
        'ASC_KEY_CONTENT',
        'MATCH_GIT_URL',
        'MATCH_PASSWORD',
      });
      expect(requiredEnv(StorePlatform.android), {'PLAY_JSON_KEY_DATA'});
    });

    test('xác thực repo match là tuỳ chọn: khai trong .env.example nhưng không bắt buộc', () {
      expect(matchRepoAuthEnv, {
        'MATCH_GIT_BASIC_AUTHORIZATION',
        'MATCH_GIT_PRIVATE_KEY',
      });
      for (final platform in StorePlatform.values) {
        expect(requiredEnv(platform).intersection(matchRepoAuthEnv), isEmpty);
      }
    });

    test('không có mật khẩu Apple ID trong biến cần thiết', () {
      final all = {
        ...requiredEnv(StorePlatform.ios),
        ...requiredEnv(StorePlatform.android),
      };
      expect(all, isNot(contains('FASTLANE_PASSWORD')));
      expect(all, isNot(contains('FASTLANE_USER')));
    });

    test('biến rỗng hoặc toàn khoảng trắng tính là thiếu', () {
      final missing = missingEnv(lanesFor(Flavor.staging), {
        'ASC_KEY_ID': 'k',
        'ASC_ISSUER_ID': '   ',
        'ASC_KEY_CONTENT': '',
        'MATCH_GIT_URL': 'u',
        'PLAY_JSON_KEY_DATA': 'j',
      });
      expect(missing, {'ASC_ISSUER_ID', 'ASC_KEY_CONTENT', 'MATCH_PASSWORD'});
    });

    test('đủ biến thì không thiếu gì; dev không cần biến nào', () {
      final full = {
        for (final n in {
          ...requiredEnv(StorePlatform.ios),
          ...requiredEnv(StorePlatform.android),
        })
          n: 'x',
      };
      expect(missingEnv(lanesFor(Flavor.prod), full), isEmpty);
      expect(missingEnv(lanesFor(Flavor.dev), {}), isEmpty);
    });
  });

  group('planRelease', () {
    test('staging customer: hai lệnh, mỗi lệnh chạy trong thư mục của nó', () {
      final plan = planRelease(
        flavor: Flavor.staging,
        app: AppKind.customer,
        buildNumber: 42,
      );
      expect(plan.map((i) => i.workingDirectory), ['ios', 'android']);
      expect(plan.map((i) => i.command), [
        'bundle exec fastlane ios beta app:customer flavor:staging build_number:42',
        'bundle exec fastlane android internal app:customer flavor:staging build_number:42',
      ]);
    });

    test('only chọn một cửa hàng: CI iOS trên macOS, Android trên ubuntu', () {
      final ios = planRelease(
        flavor: Flavor.prod,
        app: AppKind.driver,
        buildNumber: 7,
        only: StorePlatform.ios,
      );
      expect(ios, hasLength(1));
      expect(ios.single.spec.lane, 'beta');
      expect(ios.single.toString(), startsWith('(cd ios && '));
    });

    test('dev cho kế hoạch rỗng, không ném lỗi', () {
      expect(
        planRelease(flavor: Flavor.dev, app: AppKind.customer, buildNumber: 1),
        isEmpty,
      );
    });

    test('build number 0 hoặc âm bị từ chối, cả với dev', () {
      for (final n in [0, -1]) {
        expect(
          () => planRelease(
            flavor: Flavor.staging,
            app: AppKind.customer,
            buildNumber: n,
          ),
          throwsArgumentError,
        );
        expect(
          () => planRelease(
            flavor: Flavor.dev,
            app: AppKind.customer,
            buildNumber: n,
          ),
          throwsArgumentError,
        );
      }
    });

    test('build number 1 là biên hợp lệ', () {
      expect(
        planRelease(flavor: Flavor.prod, app: AppKind.customer, buildNumber: 1),
        hasLength(2),
      );
    });
  });
}
