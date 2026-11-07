import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_35/geolocator_location_source.dart';
import 'package:toi_noi_35/location_models.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:xml/xml.dart';

String read(String path) => File(path).readAsStringSync();

/// Nội dung file bỏ các dòng chú thích `#`, để test không bị đánh lừa bởi comment.
String code(String path) =>
    read(path)
        .split("\n")
        .where((l) => !l.trimLeft().startsWith("#"))
        .join("\n");

/// Đọc `<dict>` cấp một của Info.plist thành Map (chuỗi, mảng chuỗi, dict lồng).
Map<String, Object> plistDict(String path) {
  final doc = XmlDocument.parse(read(path));
  return _dict(doc.findAllElements('dict').first);
}

Map<String, Object> _dict(XmlElement dict) {
  final children = dict.childElements.toList();
  final result = <String, Object>{};
  for (var i = 0; i < children.length; i += 2) {
    assert(children[i].name.local == 'key');
    result[children[i].innerText] = _value(children[i + 1]);
  }
  return result;
}

Object _value(XmlElement e) => switch (e.name.local) {
  'string' => e.innerText,
  'array' => [for (final c in e.childElements) c.innerText],
  'dict' => _dict(e),
  final other => throw StateError('chưa hỗ trợ <$other>'),
};

Set<String> manifestPermissions(String path) => {
  for (final e in XmlDocument.parse(
    read(path),
  ).findAllElements('uses-permission'))
    e.getAttribute('android:name')!,
};

const customerPlist = 'native-config/customer/ios/Info.plist';
const driverPlist = 'native-config/driver/ios/Info.plist';
const customerManifest = 'native-config/customer/android/AndroidManifest.xml';
const driverManifest = 'native-config/driver/android/AndroidManifest.xml';

bool hasVietnameseDiacritics(String s) => RegExp(
  '[àáảãạăâđêôơưÀÁẢÃẠĂÂĐÊÔƠƯ]|[ềếểễệồốổỗộờớởỡợừứửữựìíỉĩịòóỏõọùúủũụỳýỷỹỵ]',
).hasMatch(s);

void main() {
  group('iOS Info.plist', () {
    test('app khách: chỉ quyền "khi dùng app", không quyền nền', () {
      final plist = plistDict(customerPlist);
      expect(plist, contains('NSLocationWhenInUseUsageDescription'));
      expect(
        plist,
        isNot(contains('NSLocationAlwaysAndWhenInUseUsageDescription')),
      );
      expect(plist, isNot(contains('NSLocationAlwaysUsageDescription')));
      expect(plist, isNot(contains('UIBackgroundModes')));
    });

    test(
      'app tài xế: thêm quyền "luôn cho phép" và UIBackgroundModes location',
      () {
        final plist = plistDict(driverPlist);
        expect(plist, contains('NSLocationWhenInUseUsageDescription'));
        expect(plist, contains('NSLocationAlwaysAndWhenInUseUsageDescription'));
        expect(plist['UIBackgroundModes'], ['location']);
      },
    );

    test('requestAlwaysAuthorization cần cả hai khoá: tài xế có đủ', () {
      // Apple: requestAlwaysAuthorization cần khoá WhenInUse cùng khoá Always;
      // từ iOS 11 khoá Always là NSLocationAlwaysAndWhenInUseUsageDescription.
      final plist = plistDict(driverPlist);
      expect(
        plist.keys,
        containsAll([
          'NSLocationWhenInUseUsageDescription',
          'NSLocationAlwaysAndWhenInUseUsageDescription',
        ]),
      );
    });

    test('mô tả bằng tiếng Việt có dấu, không rỗng, nói rõ mục đích', () {
      for (final path in [customerPlist, driverPlist]) {
        final plist = plistDict(path);
        for (final key in plist.keys.where(
          (k) => k.endsWith('UsageDescription'),
        )) {
          final text = plist[key]! as String;
          expect(text.length, greaterThan(40), reason: '$path $key');
          expect(hasVietnameseDiacritics(text), isTrue, reason: '$path $key');
          expect(text, isNot(contains('This app')), reason: '$path $key');
        }
      }
    });

    test('mô tả quyền nền của tài xế nói rõ khi nào ngừng gửi', () {
      final text =
          plistDict(
                driverPlist,
              )['NSLocationAlwaysAndWhenInUseUsageDescription']!
              as String;
      expect(text, contains('chuyến'));
      expect(text, contains('ngừng'));
    });

    test(
      'purposeKey của requestTemporaryFullAccuracy có trong cả hai plist',
      () {
        for (final path in [customerPlist, driverPlist]) {
          final dict =
              plistDict(path)['NSLocationTemporaryUsageDescriptionDictionary']!
                  as Map<String, Object>;
          expect(dict.keys, contains(preciseLocationPurposeKey), reason: path);
          expect(dict[preciseLocationPurposeKey], isNotEmpty);
        }
      },
    );
  });

  group('Android AndroidManifest.xml', () {
    test('app khách: COARSE + FINE, không nền, không foreground service', () {
      expect(manifestPermissions(customerManifest), {
        'android.permission.ACCESS_COARSE_LOCATION',
        'android.permission.ACCESS_FINE_LOCATION',
      });
    });

    test('cả hai app không khai ACCESS_BACKGROUND_LOCATION', () {
      // Có quyền này trong manifest thì geolocator_android thêm nó vào cùng
      // yêu cầu xin quyền, và Android 11+ bỏ qua yêu cầu gộp.
      for (final path in [customerManifest, driverManifest]) {
        expect(
          manifestPermissions(path),
          isNot(contains('android.permission.ACCESS_BACKGROUND_LOCATION')),
          reason: path,
        );
      }
    });

    test('app tài xế: thêm foreground service location', () {
      final driver = manifestPermissions(driverManifest);
      expect(
        driver,
        containsAll(manifestPermissions(customerManifest)),
        reason: 'tài xế là tập con mở rộng của khách',
      );
      expect(driver, contains('android.permission.FOREGROUND_SERVICE'));
      expect(
        driver,
        contains('android.permission.FOREGROUND_SERVICE_LOCATION'),
      );
    });

    test('cả hai app khai cặp COARSE + FINE cùng nhau (Android 12+)', () {
      for (final path in [customerManifest, driverManifest]) {
        final p = manifestPermissions(path);
        expect(
          p,
          contains('android.permission.ACCESS_COARSE_LOCATION'),
          reason: path,
        );
        expect(
          p,
          contains('android.permission.ACCESS_FINE_LOCATION'),
          reason: path,
        );
      }
    });
  });

  group('iOS build: SPM là đường chính, Podfile chỉ cho CocoaPods', () {
    test(
      'khách dùng BYPASS_PERMISSION_LOCATION_ALWAYS=1 khi build bằng SPM',
      () {
        final f = code('native-config/customer/ios/build-ios.example.sh');
        expect(
          f,
          contains('BYPASS_PERMISSION_LOCATION_ALWAYS=1 flutter build ios'),
        );
        final all = read('native-config/customer/ios/build-ios.example.sh');
        expect(all, contains('PERMISSION_HANDLER_INFO_PLIST'));
        expect(all, contains('DerivedData'));
      },
    );

    test('khách bỏ qua quyền Always của geolocator_apple', () {
      final f = code("native-config/customer/ios/Podfile.location-fragment.rb");
      expect(f, contains("'geolocator_apple'"));
      expect(f, contains('BYPASS_PERMISSION_LOCATION_ALWAYS=1'));
      expect(f, isNot(contains('PERMISSION_LOCATION=1')));
    });

    test(
      'tài xế (nhánh CocoaPods) bật macro vị trí của permission_handler_apple',
      () {
        final f = code("native-config/driver/ios/Podfile.location-fragment.rb");
        expect(f, contains("'permission_handler_apple'"));
        expect(f, contains("'PERMISSION_LOCATION=1'"));
        expect(f, isNot(contains('BYPASS_PERMISSION_LOCATION_ALWAYS')));
      },
    );
  });

  group('cấu hình khớp với code', () {
    test('mọi profile mà app khách có thể dùng đều không cần cấu hình nền', () {
      final customerProfiles = {
        for (final s in [...TripStatus.values, null])
          profileFor(AppRole.customer, s),
      };
      expect(customerProfiles.every((p) => !p.background), isTrue);
      expect(plistDict(customerPlist), isNot(contains('UIBackgroundModes')));
      expect(
        manifestPermissions(customerManifest),
        isNot(contains('android.permission.FOREGROUND_SERVICE_LOCATION')),
      );
    });

    test('profile nền của tài xế có đủ cấu hình native để chạy', () {
      final driverProfiles = {
        for (final s in [...TripStatus.values, null])
          profileFor(AppRole.driver, s),
      };
      expect(driverProfiles.any((p) => p.background), isTrue);
      expect(plistDict(driverPlist)['UIBackgroundModes'], contains('location'));
      expect(
        manifestPermissions(driverManifest),
        contains('android.permission.FOREGROUND_SERVICE_LOCATION'),
      );
    });

    test('không có key hay token trong native-config', () {
      for (final f in Directory(
        'native-config',
      ).listSync(recursive: true).whereType<File>()) {
        final text = f.readAsStringSync();
        expect(
          text,
          isNot(matches(RegExp('AIza[0-9A-Za-z_-]{20,}'))),
          reason: f.path,
        );
        expect(text.toLowerCase(), isNot(contains('api_key')), reason: f.path);
      }
    });
  });
}
