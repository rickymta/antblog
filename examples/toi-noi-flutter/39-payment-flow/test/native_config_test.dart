import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_39/payment_return.dart';
import 'package:xml/xml.dart';

const _manifest = 'native-config/android/app/src/main/AndroidManifest.xml';
const _plist = 'native-config/ios/Runner/Info.plist.fragment.xml';

XmlDocument _doc(String path) =>
    XmlDocument.parse(File(path).readAsStringSync());

/// Mỗi `<intent-filter>` thành: action, category và các `<data>`.
List<
  ({
    Set<String> actions,
    Set<String> categories,
    List<Map<String, String>> data,
  })
>
_filters() => _doc(_manifest)
    .findAllElements('intent-filter')
    .map(
      (f) => (
        actions: f
            .findElements('action')
            .map((e) => e.getAttribute('android:name')!)
            .toSet(),
        categories: f
            .findElements('category')
            .map((e) => e.getAttribute('android:name')!)
            .toSet(),
        data: f
            .findElements('data')
            .map((d) => {for (final a in d.attributes) a.name.local: a.value})
            .toList(),
      ),
    )
    .toList();

void main() {
  group('Android: toinoi://payment/return', () {
    late final ({
      Set<String> actions,
      Set<String> categories,
      List<Map<String, String>> data,
    })
    filter;

    setUpAll(() {
      filter = _filters().singleWhere((f) => f.data.isNotEmpty);
    });

    test('bộ lọc VIEW + DEFAULT + BROWSABLE', () {
      expect(filter.actions, {'android.intent.action.VIEW'});
      expect(
        filter.categories,
        containsAll([
          'android.intent.category.DEFAULT',
          'android.intent.category.BROWSABLE',
        ]),
      );
    });

    test('scheme, host và path trùng hằng của PaymentReturn', () {
      expect(filter.data, hasLength(1), reason: 'nhiều <data> bị trộn chéo');
      expect(filter.data.single, {
        'scheme': PaymentReturn.scheme,
        'host': PaymentReturn.host,
        'path': PaymentReturn.path,
      });
    });

    test('bộ lọc MAIN/LAUNCHER không lẫn <data>', () {
      final launcher = _filters().singleWhere(
        (f) => f.actions.contains('android.intent.action.MAIN'),
      );
      expect(launcher.data, isEmpty);
    });

    test(
      'activity có exported (bắt buộc từ Android 12 khi có intent-filter)',
      () {
        final activity = _doc(_manifest).findAllElements('activity').single;
        expect(activity.getAttribute('android:exported'), 'true');
      },
    );

    test('queries có CustomTabsService cho supportsLaunchMode', () {
      final actions = _doc(_manifest)
          .findAllElements('queries')
          .single
          .findAllElements('action')
          .map((e) => e.getAttribute('android:name'));
      expect(
        actions,
        contains('android.support.customtabs.action.CustomTabsService'),
      );
    });

    test('không có quyền nào ngoài INTERNET', () {
      final permissions = _doc(_manifest)
          .findAllElements('uses-permission')
          .map((e) => e.getAttribute('android:name'));
      expect(permissions, ['android.permission.INTERNET']);
    });
  });

  group('iOS: CFBundleURLTypes', () {
    test('đăng ký đúng scheme toinoi', () {
      final dict = _doc(_plist).findAllElements('dict').first;
      final keys = dict.findElements('key').map((e) => e.innerText).toList();
      expect(keys, ['CFBundleURLTypes']);
      final urlType = dict
          .findElements('array')
          .single
          .findElements('dict')
          .single;
      final schemes = urlType
          .findElements('array')
          .single
          .findElements('string')
          .map((e) => e.innerText);
      expect(schemes, [PaymentReturn.scheme]);
    });
  });
}
