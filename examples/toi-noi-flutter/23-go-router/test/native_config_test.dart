import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_23/deep_links.dart';
import 'package:xml/xml.dart';

/// Các file trong `native-config/` là bản mẫu của phần cấu hình native. Test
/// đọc chính các file đó: đổi một giá trị mà quên file còn lại là test đỏ.
///
/// Test không chứng minh hệ điều hành chấp nhận cấu hình (cần thiết bị và
/// tên miền thật), chỉ chứng minh các file nhất quán với nhau và với code Dart.
const _customerId = 'vn.toinoi.customer';

String _read(String path) => File('native-config/$path').readAsStringSync();

XmlDocument _xml(String path) => XmlDocument.parse(_read(path));

/// Bộ lọc `<intent-filter>` đã đọc ra.
typedef IntentFilter = ({
  bool autoVerify,
  Set<String> actions,
  Set<String> categories,
  List<Map<String, String>> data,
});

String? _attr(XmlElement e, String name) => e.getAttribute('android:$name');

List<IntentFilter> _intentFilters(XmlDocument manifest) {
  final activity = manifest
      .findAllElements('activity')
      .singleWhere((a) => _attr(a, 'name') == '.MainActivity');
  return [
    for (final f in activity.findElements('intent-filter'))
      (
        autoVerify: _attr(f, 'autoVerify') == 'true',
        actions: {for (final a in f.findElements('action')) _attr(a, 'name')!},
        categories: {
          for (final c in f.findElements('category')) _attr(c, 'name')!,
        },
        data: [
          for (final d in f.findElements('data'))
            {for (final a in d.attributes) a.name.local: a.value},
        ],
      ),
  ];
}

/// Android gộp mọi `<data>` trong một filter rồi so từng phần của URI
/// (scheme, host, path). Đây là bản rút gọn cho scheme, host và pathPrefix.
bool filterMatches(IntentFilter filter, Uri uri) {
  if (!filter.actions.contains('android.intent.action.VIEW')) return false;
  if (!filter.categories.contains('android.intent.category.BROWSABLE')) {
    return false;
  }
  final schemes = {for (final d in filter.data) ?d['scheme']};
  final hosts = {for (final d in filter.data) ?d['host']};
  final prefixes = {for (final d in filter.data) ?d['pathPrefix']};
  if (!schemes.contains(uri.scheme)) return false;
  if (hosts.isNotEmpty && !hosts.contains(uri.host)) return false;
  if (prefixes.isNotEmpty && !prefixes.any(uri.path.startsWith)) return false;
  return true;
}

bool isSha256Fingerprint(String s) =>
    RegExp(r'^([0-9A-F]{2}:){31}[0-9A-F]{2}$').hasMatch(s);

bool isAppId(String s) =>
    RegExp(r'^[A-Z0-9]{10}\.[A-Za-z0-9.\-]+$').hasMatch(s);

/// Đọc plist dạng `<dict>` phẳng thành Map (đủ cho hai file mẫu).
Object? _plistValue(XmlElement e) => switch (e.name.local) {
  'string' => e.innerText,
  'true' => true,
  'false' => false,
  'array' => [for (final c in e.childElements) _plistValue(c)],
  'dict' => _plistDict(e),
  final other => throw FormatException('thẻ plist chưa hỗ trợ: $other'),
};

Map<String, Object?> _plistDict(XmlElement dict) {
  final children = dict.childElements.toList();
  return {
    for (var i = 0; i < children.length; i += 2)
      children[i].innerText: _plistValue(children[i + 1]),
  };
}

Map<String, Object?> _plist(String path) =>
    _plistDict(_xml(path).findAllElements('dict').first);

void main() {
  group('AndroidManifest.xml.example', () {
    final manifest = _xml('android/AndroidManifest.xml.example');
    final filters = _intentFilters(manifest);

    test('activity xuất ra ngoài và dùng singleTop', () {
      final activity = manifest.findAllElements('activity').single;
      expect(_attr(activity, 'exported'), 'true');
      expect(_attr(activity, 'launchMode'), 'singleTop');
    });

    test('filter LAUNCHER tách riêng, không lẫn <data>', () {
      final launcher = filters.singleWhere(
        (f) => f.categories.contains('android.intent.category.LAUNCHER'),
      );
      expect(launcher.actions, {'android.intent.action.MAIN'});
      expect(launcher.data, isEmpty);
      expect(launcher.autoVerify, isFalse);
    });

    test('custom scheme: scheme và host khớp DeepLinks, không autoVerify', () {
      final custom = filters.singleWhere(
        (f) => f.data.any((d) => d['scheme'] == DeepLinks.scheme),
      );
      expect(custom.autoVerify, isFalse);
      expect(custom.actions, {'android.intent.action.VIEW'});
      expect(custom.categories, {
        'android.intent.category.DEFAULT',
        'android.intent.category.BROWSABLE',
      });
      expect(custom.data, [
        {'scheme': DeepLinks.scheme, 'host': 'trip'},
      ]);
    });

    test('App Link: https, đúng tên miền, pathPrefix /trip/, autoVerify', () {
      final web = filters.singleWhere((f) => f.autoVerify);
      expect(web.actions, {'android.intent.action.VIEW'});
      expect(web.categories, {
        'android.intent.category.DEFAULT',
        'android.intent.category.BROWSABLE',
      });
      expect(web.data, [
        {'scheme': 'https', 'host': DeepLinks.webHost, 'pathPrefix': '/trip/'},
      ]);
    });

    test('không tắt xử lý deep link của Flutter', () {
      final disabled = manifest
          .findAllElements('meta-data')
          .where(
            (m) =>
                _attr(m, 'name') == 'flutter_deeplinking_enabled' &&
                _attr(m, 'value') == 'false',
          );
      expect(disabled, isEmpty);
    });

    test('link mà manifest nhận thì code Dart hiểu, và ngược lại', () {
      bool androidAccepts(Uri uri) => filters.any((f) => filterMatches(f, uri));

      final accepted = [
        DeepLinks.customTrip('T-1002'),
        DeepLinks.webTrip('T-1002'),
        Uri.parse('https://toinoi.example/trip/T-1002?ref=mail'),
      ];
      for (final uri in accepted) {
        expect(androidAccepts(uri), isTrue, reason: '$uri');
        expect(DeepLinks.resolve(uri), startsWith('/trip/'), reason: '$uri');
      }

      final rejected = [
        Uri.parse('https://toinoi.example/pricing'),
        Uri.parse('https://evil.example/trip/T-1002'),
        Uri.parse('http://toinoi.example/trip/T-1002'),
        Uri.parse('toinoi://payment/confirm'),
      ];
      for (final uri in rejected) {
        expect(androidAccepts(uri), isFalse, reason: '$uri');
      }
    });
  });

  group('iOS', () {
    test('Info.plist: CFBundleURLSchemes chứa scheme của DeepLinks', () {
      final plist = _plist('ios/Info.plist.example');
      final types = plist['CFBundleURLTypes']! as List<Object?>;
      final schemes = [
        for (final t in types)
          ...((t! as Map<String, Object?>)['CFBundleURLSchemes']!
              as List<Object?>),
      ];
      expect(schemes, [DeepLinks.scheme]);
      expect(plist['FlutterDeepLinkingEnabled'], anyOf(isNull, isTrue));
    });

    test(
      'entitlements: applinks:<tên miền>, không có scheme hay đường dẫn',
      () {
        final plist = _plist('ios/Runner.entitlements.example');
        final domains =
            plist['com.apple.developer.associated-domains']! as List<Object?>;
        expect(domains, ['applinks:${DeepLinks.webHost}']);
      },
    );
  });

  group('file đặt trên web server (/.well-known/)', () {
    final aasa = jsonDecode(
      _read('web/.well-known/apple-app-site-association.example'),
    ) as Map<String, Object?>;
    final links = jsonDecode(
      _read('web/.well-known/assetlinks.json.example'),
    ) as List<Object?>;

    test('apple-app-site-association: appID và components', () {
      final details =
          ((aasa['applinks']! as Map<String, Object?>)['details']!
                  as List<Object?>)
              .cast<Map<String, Object?>>();
      expect(details, hasLength(1));
      final appIds = (details.single['appIDs']! as List<Object?>)
          .cast<String>();
      expect(appIds, ['ABCDE12345.$_customerId']);
      expect(appIds.every(isAppId), isTrue);
      expect(details.single['components'], [
        {'/': '/trip/*'},
      ]);
    });

    test('assetlinks.json: quan hệ, package và vân tay chứng chỉ', () {
      final entry = links.single! as Map<String, Object?>;
      expect(entry['relation'], ['delegate_permission/common.handle_all_urls']);
      final target = entry['target']! as Map<String, Object?>;
      expect(target['namespace'], 'android_app');
      expect(target['package_name'], _customerId);
      final prints = (target['sha256_cert_fingerprints']! as List<Object?>)
          .cast<String>();
      expect(prints, isNotEmpty);
      expect(prints.every(isSha256Fingerprint), isTrue);
    });

    test('hai file nói về cùng một app và cùng một nhánh đường dẫn', () {
      final details =
          ((aasa['applinks']! as Map<String, Object?>)['details']!
                  as List<Object?>)
              .cast<Map<String, Object?>>();
      final appId = (details.single['appIDs']! as List<Object?>).single!
          .toString();
      final target =
          (links.single! as Map<String, Object?>)['target']!
              as Map<String, Object?>;
      expect(appId.endsWith('.${target['package_name']}'), isTrue);

      // /trip/* của iOS và pathPrefix /trip/ của Android cùng phủ /trip/{id}.
      final component = (details.single['components']! as List<Object?>)
          .cast<Map<String, Object?>>()
          .single['/']!
          .toString();
      final prefix = _intentFilters(_xml('android/AndroidManifest.xml.example'))
          .singleWhere((f) => f.autoVerify)
          .data
          .single['pathPrefix']!;
      expect(component, '$prefix*');
    });
  });

  group('bộ kiểm tra có bắt được giá trị sai', () {
    test('vân tay: thiếu byte, chữ thường, sai ký tự', () {
      final good = List.filled(32, 'AB').join(':');
      expect(isSha256Fingerprint(good), isTrue);
      expect(isSha256Fingerprint(List.filled(31, 'AB').join(':')), isFalse);
      expect(isSha256Fingerprint(good.toLowerCase()), isFalse);
      expect(isSha256Fingerprint(good.replaceFirst('AB', 'GZ')), isFalse);
      expect(isSha256Fingerprint(good.replaceAll(':', '')), isFalse);
    });

    test('appID: phải có Team ID 10 ký tự', () {
      expect(isAppId('ABCDE12345.vn.toinoi.customer'), isTrue);
      expect(isAppId('vn.toinoi.customer'), isFalse);
      expect(isAppId('abcde12345.vn.toinoi.customer'), isFalse);
    });

    test(
      'filterMatches: thiếu BROWSABLE thì không nhận link từ trình duyệt',
      () {
        const filter = (
          autoVerify: true,
          actions: {'android.intent.action.VIEW'},
          categories: {'android.intent.category.DEFAULT'},
          data: [
            {
              'scheme': 'https',
              'host': 'toinoi.example',
              'pathPrefix': '/trip/',
            },
          ],
        );
        expect(filterMatches(filter, DeepLinks.webTrip('T-1')), isFalse);
      },
    );
  });
}
