import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_59/app_version.dart';
import 'package:toi_noi_59/feature_flags.dart';
import 'package:toi_noi_59/min_version_policy.dart';
import 'package:toi_noi_59/store_platform.dart';
import 'package:yaml/yaml.dart';

String read(String name) => File('native-config/$name').readAsStringSync();

void main() {
  group('min-version.example.json', () {
    final json =
        jsonDecode(read('min-version.example.json')) as Map<String, Object?>;
    final config = VersionPolicyConfig.fromJson(json);

    test('đọc được và có đủ hai nền tảng', () {
      expect(config.rules.keys, unorderedEquals(StorePlatform.values));
    });

    test('thứ tự min <= latest; build của hai cửa hàng khác nhau', () {
      for (final rule in config.rules.values) {
        expect(rule.minSupported <= rule.latestPublished, isTrue);
      }
      expect(
        config.rules[StorePlatform.android]!.latestPublished.build,
        isNot(config.rules[StorePlatform.ios]!.latestPublished.build),
      );
    });

    test('mẫu thật sự chặn bản cũ và cho qua bản mới', () {
      UpdateDecision at(String v, StorePlatform p) => MinVersionPolicy.evaluate(
        current: AppVersion.parse(v),
        platform: p,
        config: config,
      );
      expect(at('1.3.9+59', StorePlatform.android), isA<UpdateForce>());
      expect(at('1.6.0+87', StorePlatform.android), isA<UpdateOk>());
      expect(at('1.6.0+88', StorePlatform.ios), isA<UpdateOk>());
    });

    test(
      'link cửa hàng trong mẫu được chấp nhận, không bị thay bằng mặc định',
      () {
        for (final platform in StorePlatform.values) {
          final entry = json[platform.name]! as Map<String, Object?>;
          expect(
            config.rules[platform]!.storeUri.toString(),
            entry['storeUrl'],
            reason: platform.name,
          );
        }
      },
    );
  });

  group('remote-config-template.example.json khớp với mặc định trong app', () {
    final template = jsonDecode(
      read('remote-config-template.example.json'),
    ) as Map<String, Object?>;
    final params = template['parameters']! as Map<String, Object?>;

    Map<String, Object?> param(String key) =>
        params[key]! as Map<String, Object?>;

    test('đúng bộ khoá với danh mục cờ, không thừa không thiếu', () {
      expect(params.keys, unorderedEquals(Flags.all.map((f) => f.key)));
    });

    test('defaultValue của mẫu parse ra đúng defaultValue trong app', () {
      for (final flag in Flags.all) {
        final raw =
            (param(flag.key)['defaultValue']! as Map<String, Object?>)['value'];
        expect(flag.parse(raw), flag.defaultValue, reason: flag.key);
      }
    });

    test('valueType khớp kiểu của cờ', () {
      expect(param('payment_vixanh_enabled')['valueType'], 'BOOLEAN');
      expect(param('driver_new_routing_enabled')['valueType'], 'BOOLEAN');
      expect(param('quote_refresh_seconds')['valueType'], 'NUMBER');
    });

    test('mỗi tham số có mô tả (Firebase giới hạn 256 ký tự)', () {
      for (final key in params.keys) {
        final d = param(key)['description']! as String;
        expect(d, isNotEmpty);
        expect(d.length, lessThanOrEqualTo(256));
      }
    });
  });

  group('shorebird.yaml.example', () {
    final yaml = loadYaml(read('shorebird.yaml.example')) as YamlMap;
    final uuid = RegExp(
      r'^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$',
    );

    test('có app_id và một app_id cho mỗi flavor của bài 25', () {
      expect(yaml['app_id'], matches(uuid));
      final flavors = yaml['flavors'] as YamlMap;
      expect(flavors.keys, unorderedEquals(['dev', 'staging', 'prod']));
      expect(flavors.values.toSet(), hasLength(3));
      for (final id in flavors.values) {
        expect(id, matches(uuid));
      }
    });

    test(
      'app_id cấp trên là của flavor đầu tiên (dev), theo tài liệu Shorebird',
      () {
        final flavors = yaml['flavors'] as YamlMap;
        expect(flavors.keys.first, 'dev');
        expect(yaml['app_id'], flavors['dev']);
      },
    );

    test('là UUID giả rõ ràng, không phải id thật', () {
      expect(yaml['app_id'].toString(), startsWith('00000000-'));
    });
  });

  test('không có khoá hay token thật trong native-config', () {
    final secretShapes = RegExp(
      r'AIza[0-9A-Za-z_-]{20,}|-----BEGIN [A-Z ]*PRIVATE KEY-----|'
      r'ghp_[0-9A-Za-z]{20,}|[0-9]{10,}:[0-9A-Za-z_-]{30,}',
    );
    for (final file in Directory('native-config').listSync(recursive: true)) {
      if (file is! File) continue;
      expect(
        secretShapes.hasMatch(file.readAsStringSync()),
        isFalse,
        reason: file.path,
      );
    }
  });

  test('.gitignore neo /android/ và /ios/ để không nuốt native-config/', () {
    final lines = File('.gitignore').readAsLinesSync();
    expect(lines, containsAll(['/android/', '/ios/']));
    expect(lines, isNot(contains('android/')));
    expect(lines, isNot(contains('native-config/')));
  });
}
