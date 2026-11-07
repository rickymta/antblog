import 'package:test/test.dart';
import 'package:toi_noi_52/toi_noi_52.dart';

String wrap(String body) =>
    '<?xml version="1.0" encoding="UTF-8"?><plist version="1.0">$body</plist>';

void main() {
  group('parsePlist', () {
    test('đọc đủ các kiểu mà profile dùng', () {
      final map = parsePlist(
        wrap('''
<dict>
  <key>s</key><string>chuỗi có dấu: Tới Nơi</string>
  <key>t</key><true/>
  <key>f</key><false/>
  <key>i</key><integer>42</integer>
  <key>r</key><real>1.5</real>
  <key>d</key><date>2026-11-07T08:30:00Z</date>
  <key>b</key><data>
    RkFL
    RQ==
  </data>
  <key>a</key><array><string>x</string><string>y</string></array>
  <key>m</key><dict><key>k</key><string>v</string></dict>
</dict>'''),
      );
      expect(map['s'], 'chuỗi có dấu: Tới Nơi');
      expect(map['t'], isTrue);
      expect(map['f'], isFalse);
      expect(map['i'], 42);
      expect(map['r'], 1.5);
      expect(map['d'], DateTime.utc(2026, 11, 7, 8, 30));
      expect(map['b'], 'RkFLRQ==', reason: 'xuống dòng trong data bị bỏ');
      expect(map['a'], ['x', 'y']);
      expect(map['m'], {'k': 'v'});
    });

    test('giữ thứ tự và bỏ qua chú thích XML', () {
      final map = parsePlist(
        wrap(
          '<dict><!-- chú thích --><key>b</key><true/><key>a</key><true/></dict>',
        ),
      );
      expect(map.keys, ['b', 'a']);
    });

    test('từ chối XML hỏng', () {
      expect(() => parsePlist('<plist><dict>'), throwsFormatException);
    });

    test('từ chối gốc không phải dict', () {
      expect(
        () => parsePlist(wrap('<array><string>x</string></array>')),
        throwsFormatException,
      );
    });

    test('từ chối key không kèm giá trị', () {
      expect(
        () => parsePlist(wrap('<dict><key>a</key></dict>')),
        throwsFormatException,
      );
    });

    test('từ chối kiểu lạ', () {
      expect(
        () => parsePlist(wrap('<dict><key>a</key><blob/></dict>')),
        throwsFormatException,
      );
    });

    test('từ chối ngày sai định dạng', () {
      expect(
        () => parsePlist(wrap('<dict><key>a</key><date>hôm qua</date></dict>')),
        throwsFormatException,
      );
    });
  });

  group('parseXcconfig', () {
    test('đọc KEY = value, bỏ chú thích, dòng trống và #include', () {
      final map = parseXcconfig('''
// chú thích
#include "Generated.xcconfig"

A = 1
B =   hai từ  // chú thích cuối dòng
C = "có nháy"
D[sdk=iphoneos*] = 4
''');
      expect(map, {
        'A': '1',
        'B': 'hai từ',
        'C': 'có nháy',
        'D[sdk=iphoneos*]': '4',
      });
    });

    test('dòng sau thắng dòng trước', () {
      expect(parseXcconfig('A = 1\nA = 2'), {'A': '2'});
    });

    test('từ chối dòng không có dấu =', () {
      expect(() => parseXcconfig('A 1'), throwsFormatException);
    });
  });
}
