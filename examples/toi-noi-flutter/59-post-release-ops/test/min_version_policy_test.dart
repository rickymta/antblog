import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_59/app_version.dart';
import 'package:toi_noi_59/min_version_policy.dart';
import 'package:toi_noi_59/store_link.dart';
import 'package:toi_noi_59/store_platform.dart';

Map<String, Object?> androidRule({
  String min = '1.4.0+60',
  String? rec = '1.6.0+87',
  String latest = '1.6.0+87',
  Object? url,
  Object? message,
}) => {
  'android': {
    'minSupported': min,
    'recommended': ?rec,
    'latestPublished': latest,
    'storeUrl': ?url,
    'message': ?message,
  },
};

UpdateDecision evaluate(
  String current,
  Map<String, Object?> json, {
  StorePlatform platform = StorePlatform.android,
}) => MinVersionPolicy.evaluate(
  current: AppVersion.parse(current),
  platform: platform,
  config: VersionPolicyConfig.fromJson(json),
);

void main() {
  group('ba mức quyết định', () {
    final config = androidRule();

    test('dưới minSupported là force', () {
      expect(evaluate('1.3.9+999', config), isA<UpdateForce>());
    });

    test('đúng minSupported không bị force, nhưng được nhắc', () {
      expect(evaluate('1.4.0+60', config), isA<UpdateRecommend>());
    });

    test('cùng x.y.z, build thấp hơn một đơn vị là force', () {
      expect(evaluate('1.4.0+59', config), isA<UpdateForce>());
    });

    test('giữa minSupported và recommended là recommend', () {
      final d = evaluate('1.5.2+75', config) as UpdateRecommend;
      expect(d.recommended, AppVersion.parse('1.6.0+87'));
    });

    test('đúng recommended là ok', () {
      final d = evaluate('1.6.0+87', config);
      expect(d, isA<UpdateOk>());
      expect((d as UpdateOk).reason, OkReason.upToDate);
    });

    test('mới hơn latestPublished (bản đang thử nội bộ) vẫn ok', () {
      expect(evaluate('1.7.0+90', config), isA<UpdateOk>());
    });

    test('không có recommended thì chỉ còn force hoặc ok', () {
      final c = androidRule(rec: null);
      expect(evaluate('1.5.0+70', c), isA<UpdateOk>());
      expect(evaluate('1.3.0+10', c), isA<UpdateForce>());
    });
  });

  group('minSupported không ghi build', () {
    final config = androidRule(min: '1.4.0', rec: null, latest: '1.6.0+87');

    test('mọi 1.4.0+n đều đạt', () {
      expect(evaluate('1.4.0+0', config), isA<UpdateOk>());
      expect(evaluate('1.4.0+1', config), isA<UpdateOk>());
    });

    test('1.3.9 dù build rất cao vẫn bị force', () {
      expect(evaluate('1.3.9+9999', config), isA<UpdateForce>());
    });
  });

  group('theo nền tảng', () {
    final json = <String, Object?>{
      ...androidRule(),
      'ios': {'minSupported': '1.5.0+70', 'latestPublished': '1.6.0+88'},
    };

    test('mỗi nền tảng dùng quy tắc của nó', () {
      expect(evaluate('1.4.5+65', json), isA<UpdateRecommend>());
      expect(
        evaluate('1.4.5+65', json, platform: StorePlatform.ios),
        isA<UpdateForce>(),
      );
    });

    test('nền tảng không có quy tắc thì ok với lý do riêng', () {
      final d = evaluate('0.0.1', androidRule(), platform: StorePlatform.ios);
      expect((d as UpdateOk).reason, OkReason.noRuleForPlatform);
    });
  });

  group('thông điệp', () {
    test('dùng thông điệp của server, bỏ khoảng trắng', () {
      final d = evaluate('1.0.0', androidRule(message: '  Cập nhật gấp  '));
      expect((d as UpdateForce).message, 'Cập nhật gấp');
    });

    test('rỗng hoặc sai kiểu thì dùng thông điệp mặc định', () {
      for (final m in ['   ', 42, null]) {
        final d = evaluate('1.0.0', androidRule(message: m)) as UpdateForce;
        expect(d.message, MinVersionPolicy.defaultForceMessage);
      }
    });

    test('recommend có thông điệp mặc định riêng', () {
      final d = evaluate('1.5.0+70', androidRule()) as UpdateRecommend;
      expect(d.message, MinVersionPolicy.defaultRecommendMessage);
    });
  });

  group('cấu hình sai bị từ chối (FormatException)', () {
    void rejects(String why, Object? json) {
      test(why, () {
        expect(() => VersionPolicyConfig.fromJson(json), throwsFormatException);
      });
    }

    rejects(
      'minSupported lớn hơn latestPublished',
      androidRule(min: '9.0.0', latest: '1.6.0+87'),
    );
    rejects(
      'recommended thấp hơn minSupported',
      androidRule(min: '1.5.0', rec: '1.4.0'),
    );
    rejects('recommended cao hơn latestPublished', androidRule(rec: '2.0.0'));
    rejects('thiếu minSupported', {
      'android': {'latestPublished': '1.6.0'},
    });
    rejects('thiếu latestPublished', {
      'android': {'minSupported': '1.4.0'},
    });
    rejects('minSupported sai kiểu', {
      'android': {'minSupported': 140, 'latestPublished': '1.6.0'},
    });
    rejects('chuỗi phiên bản hỏng', androidRule(min: 'một.hai.ba'));
    rejects('nền tảng không phải object', {'android': 'xin chào'});
    rejects('gốc không phải object', ['android']);
    rejects('null', null);

    test('chuỗi JSON thật qua jsonDecode cũng đi đúng đường', () {
      final decoded = jsonDecode(
        '{"android":{"minSupported":"1.4.0","latestPublished":"1.6.0"}}',
      );
      expect(VersionPolicyConfig.fromJson(decoded).rules, hasLength(1));
    });
  });

  group('StoreLink.resolve', () {
    const good =
        'https://play.google.com/store/apps/details?id=vn.toinoi.customer';

    test('nhận link https của đúng cửa hàng', () {
      expect(StoreLink.resolve(good, StorePlatform.android), Uri.parse(good));
    });

    test('thay bằng link dựng sẵn khi thiếu hoặc đáng ngờ', () {
      final fallback = StoreLink.androidDefault;
      for (final raw in <String?>[
        null,
        '',
        'http://play.google.com/store/apps/details?id=x',
        'https://evil.example/store',
        'https://play.google.com.evil.example/x',
        'https://user:pw@play.google.com/store',
        'intent://scan#Intent;end',
        'market://details?id=vn.toinoi.customer',
        'javascript:alert(1)',
        'https://apps.apple.com/app/id1', // sai cửa hàng cho Android
        '::không phải uri::',
      ]) {
        expect(
          StoreLink.resolve(raw, StorePlatform.android),
          fallback,
          reason: '$raw',
        );
      }
    });

    test('iOS chỉ nhận apps.apple.com', () {
      expect(
        StoreLink.resolve(
          'https://apps.apple.com/app/id123',
          StorePlatform.ios,
        ),
        Uri.parse('https://apps.apple.com/app/id123'),
      );
      expect(StoreLink.resolve(good, StorePlatform.ios), StoreLink.iosDefault);
    });

    test('link đáng ngờ trong cấu hình không làm hỏng quyết định force', () {
      final d = evaluate(
        '1.0.0',
        androidRule(url: 'https://evil.example/x'),
      ) as UpdateForce;
      expect(d.storeUri, StoreLink.androidDefault);
    });
  });
}
