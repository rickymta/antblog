import 'package:test/test.dart';
import 'package:toi_noi_52/toi_noi_52.dart';

import 'support.dart';

void main() {
  group('đọc profile mẫu', () {
    test('ad hoc: thông tin cơ bản', () {
      final p = sample('staging-adhoc');
      expect(p.name, 'ToiNoi Customer Staging AdHoc');
      expect(p.teamIds, [fakeTeamId]);
      expect(p.applicationIdentifier, '$fakeTeamId.vn.toinoi.customer.staging');
      expect(p.bundleId, 'vn.toinoi.customer.staging');
      expect(p.expirationDate, DateTime.utc(2027, 1, 20));
      expect(p.provisionedDevices, hasLength(3));
    });

    test('thiếu khoá bắt buộc thì ném FormatException', () {
      final plist = parsePlist(readLab('samples/staging-adhoc.plist'))
        ..remove('ExpirationDate');
      expect(
        () => ProvisioningProfile.fromPlist(plist),
        throwsA(
          isA<FormatException>().having(
            (e) => e.message,
            'message',
            contains('ExpirationDate'),
          ),
        ),
      );
    });

    test('TeamIdentifier rỗng bị từ chối', () {
      final plist = parsePlist(readLab('samples/staging-adhoc.plist'))
        ..['TeamIdentifier'] = <Object?>[];
      expect(() => ProvisioningProfile.fromPlist(plist), throwsFormatException);
    });
  });

  group('loại profile', () {
    test('development: có thiết bị và get-task-allow bật', () {
      expect(sample('dev').kind, ProfileKind.development);
    });
    test('ad hoc: có thiết bị, get-task-allow tắt', () {
      expect(sample('staging-adhoc').kind, ProfileKind.adHoc);
    });
    test('App Store: không có ProvisionedDevices', () {
      final p = sample('prod-appstore');
      expect(p.provisionedDevices, isNull);
      expect(p.kind, ProfileKind.appStore);
    });
    test('enterprise: ProvisionsAllDevices', () {
      final plist = parsePlist(readLab('samples/prod-appstore.plist'))
        ..['ProvisionsAllDevices'] = true;
      final p = ProvisioningProfile.fromPlist(plist);
      expect(p.kind, ProfileKind.enterprise);
      expect(p.allowsDevice('bất-kỳ'), isTrue);
    });
  });

  group('hạn dùng', () {
    final p = sample('staging-adhoc'); // hết hạn 2027-01-20T00:00:00Z
    final expiry = p.expirationDate;

    test('còn xa thì hợp lệ', () {
      expect(p.statusAt(DateTime.utc(2026, 11, 7)), ExpiryStatus.valid);
    });

    test('đúng 30 ngày trước hạn bắt đầu cảnh báo', () {
      final now = expiry.subtract(const Duration(days: 30));
      expect(p.statusAt(now), ExpiryStatus.expiringSoon);
    });

    test('31 ngày trước hạn chưa cảnh báo', () {
      final now = expiry.subtract(const Duration(days: 31));
      expect(p.statusAt(now), ExpiryStatus.valid);
    });

    test('đúng giây hết hạn đã coi là hết hạn', () {
      expect(p.statusAt(expiry), ExpiryStatus.expired);
    });

    test('một giây trước hạn vẫn còn, nhưng sắp hết', () {
      final now = expiry.subtract(const Duration(seconds: 1));
      expect(p.statusAt(now), ExpiryStatus.expiringSoon);
    });

    test('ngưỡng cảnh báo tuỳ chỉnh', () {
      final now = expiry.subtract(const Duration(days: 10));
      expect(
        p.statusAt(now, warnWithin: const Duration(days: 7)),
        ExpiryStatus.valid,
      );
    });

    test('giờ địa phương được quy về UTC trước khi so', () {
      // Cùng một thời điểm, biểu diễn bằng múi giờ khác.
      final utc = expiry.subtract(const Duration(days: 5));
      final local = DateTime.fromMillisecondsSinceEpoch(
        utc.millisecondsSinceEpoch,
      );
      expect(p.remaining(local), p.remaining(utc));
    });
  });

  group('thiết bị', () {
    final p = sample('staging-adhoc');
    test('UDID trong danh sách được phép', () {
      expect(p.allowsDevice('00008030-000000000000000A'), isTrue);
    });
    test('UDID ngoài danh sách bị từ chối', () {
      expect(p.allowsDevice('00008030-00000000000000FF'), isFalse);
    });
    test('profile App Store không chạy cục bộ trên thiết bị nào', () {
      expect(
        sample('prod-appstore').allowsDevice('00008030-000000000000000A'),
        isFalse,
      );
    });
  });

  group('allowlist entitlement', () {
    final claimed = configPlist('Runner.entitlements.example').keys;

    test('profile tạo sau khi bật capability cho phép mọi quyền app xin', () {
      expect(sample('staging-adhoc').missingEntitlements(claimed), isEmpty);
    });

    test('profile cũ thiếu push và associated domains', () {
      expect(sample('staging-adhoc-old').missingEntitlements(claimed), {
        'aps-environment',
        'com.apple.developer.associated-domains',
      });
    });

    test('allowlist được phép rộng hơn thứ app xin', () {
      expect(sample('staging-adhoc').missingEntitlements(const []), isEmpty);
    });
  });

  group('bundle ID', () {
    test('explicit chỉ khớp đúng một bundle ID', () {
      final p = sample('staging-adhoc');
      expect(p.coversBundleId('vn.toinoi.customer.staging'), isTrue);
      expect(p.coversBundleId('vn.toinoi.customer'), isFalse);
      expect(p.coversBundleId('vn.toinoi.customer.staging.extra'), isFalse);
    });

    test('wildcard khớp theo tiền tố', () {
      final plist = parsePlist(readLab('samples/dev.plist'));
      (plist['Entitlements']!
              as Map<String, Object?>)['application-identifier'] =
          '$fakeTeamId.vn.toinoi.*';
      final p = ProvisioningProfile.fromPlist(plist);
      expect(p.coversBundleId('vn.toinoi.customer.dev'), isTrue);
      expect(p.coversBundleId('vn.other.app'), isFalse);
    });
  });
}
