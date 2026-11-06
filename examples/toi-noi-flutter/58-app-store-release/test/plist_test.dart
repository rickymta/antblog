import 'package:test/test.dart';
import 'package:toi_noi_58/toi_noi_58.dart';

import 'support.dart';

void main() {
  group('parsePlist', () {
    test('đọc đủ kiểu: dict, array, string, integer, real, true, false', () {
      final value = parsePlist(
        '<plist version="1.0"><dict>'
        '<key>s</key><string>chữ có dấu &amp; ký hiệu &lt;&gt;</string>'
        '<key>i</key><integer>-7</integer>'
        '<key>r</key><real>1.5</real>'
        '<key>t</key><true/><key>f</key><false/>'
        '<key>a</key><array><string>x</string><string>y</string></array>'
        '<key>d</key><dict><key>k</key><string/></dict>'
        '</dict></plist>',
      );
      expect(value, {
        's': 'chữ có dấu & ký hiệu <>',
        'i': -7,
        'r': 1.5,
        't': true,
        'f': false,
        'a': ['x', 'y'],
        'd': {'k': ''},
      });
    });

    test('bỏ qua khai báo xml, DOCTYPE và comment (kể cả chữ có dấu)', () {
      final value = parsePlist(
        infoXml(usage: {'NSLocationWhenInUseUsageDescription': 'Mô tả'})
            .replaceFirst(
              '<dict>',
              '<!-- Chú thích <key>không phải khoá</key> --><dict>',
            ),
      );
      expect(value, {'NSLocationWhenInUseUsageDescription': 'Mô tả'});
    });

    test('mảng và dict rỗng viết dạng <array/> và <dict/>', () {
      expect(parsePlist('<plist><array/></plist>'), <Object?>[]);
      expect(parsePlist('<plist><dict/></plist>'), <String, Object?>{});
    });

    test('sai định dạng ném FormatException', () {
      const bad = {
        'thiếu </plist>': '<plist><dict></dict>',
        'dict không đóng':
            '<plist><dict><key>a</key><string>x</string></plist>',
        'khoá lặp': '<plist><dict><key>a</key><true/><key>a</key><false/></dict></plist>',
        'integer sai': '<plist><integer>abc</integer></plist>',
        'thẻ lạ': '<plist><blob/></plist>',
        'sau </plist> còn nội dung': '<plist><true/></plist><true/>',
        'khoá không có giá trị': '<plist><dict><key>a</key></dict></plist>',
      };
      for (final e in bad.entries) {
        expect(() => parsePlist(e.value), throwsFormatException, reason: e.key);
      }
    });

    test(
      'đọc được manifest của plugin shared_preferences_foundation 2.6.1',
      () {
        // Nội dung lấy từ ~/.pub-cache của máy kiểm (11/2026), rút gọn khoảng trắng.
        final manifest = PrivacyManifest.parse(
          '<?xml version="1.0" encoding="UTF-8"?>'
          '<plist version="1.0"><dict>'
          '<key>NSPrivacyTrackingDomains</key><array/>'
          '<key>NSPrivacyAccessedAPITypes</key><array><dict>'
          '<key>NSPrivacyAccessedAPIType</key>'
          '<string>NSPrivacyAccessedAPICategoryUserDefaults</string>'
          '<key>NSPrivacyAccessedAPITypeReasons</key><array><string>1C8F.1</string></array>'
          '</dict></array>'
          '<key>NSPrivacyCollectedDataTypes</key><array/>'
          '<key>NSPrivacyTracking</key><false/>'
          '</dict></plist>',
        );
        expect(manifest.tracking, isFalse);
        expect(
          manifest.accessedApis.single.category,
          RequiredReasonApi.userDefaults,
        );
        expect(manifest.accessedApis.single.reasons, ['1C8F.1']);
      },
    );
  });
}
