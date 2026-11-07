import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_59/app_version.dart';

void main() {
  group('AppVersion.parse', () {
    test('đọc x.y.z+build', () {
      final v = AppVersion.parse('1.6.0+87');
      expect((v.major, v.minor, v.patch, v.build), (1, 6, 0, 87));
    });

    test('thiếu +build thì build là 0', () {
      expect(AppVersion.parse('1.4.0').build, 0);
    });

    test('bỏ khoảng trắng hai đầu', () {
      expect(AppVersion.parse(' 1.4.0+5 '), const AppVersion(1, 4, 0, 5));
    });

    for (final bad in [
      '',
      '1.4',
      '1.4.0.1',
      '1.4.0-beta',
      '1.4.0-beta+3',
      'v1.4.0',
      '-1.4.0',
      '1.-4.0',
      '1.4.0+',
      '1.4.0+-3',
      '1.4.0+x',
      '1.4.0 +3',
      '１.4.0', // chữ số toàn chiều
      '1.4.0\n+3', // xuống dòng nằm giữa
      '1234567890.0.0', // quá 9 chữ số, tránh tràn khi đổi sang số
    ]) {
      test('từ chối "${bad.replaceAll('\n', r'\n')}"', () {
        expect(() => AppVersion.parse(bad), throwsFormatException);
      });
    }

    test('build lớn như versionCode vẫn đọc được', () {
      expect(AppVersion.parse('1.0.0+2100000000').build, 2100000000);
    });
  });

  group('so sánh', () {
    test('so số chứ không so chuỗi: 1.9.0 < 1.10.0', () {
      expect(AppVersion.parse('1.9.0') < AppVersion.parse('1.10.0'), isTrue);
    });

    test('thứ tự major, minor, patch, build', () {
      final ordered = [
        '0.9.9+999',
        '1.0.0',
        '1.0.0+1',
        '1.0.1',
        '1.1.0',
        '2.0.0',
      ].map(AppVersion.parse).toList();
      final shuffled = [...ordered.reversed]..sort();
      expect(shuffled, ordered);
    });

    test('build quyết định khi x.y.z bằng nhau', () {
      expect(
        AppVersion.parse('1.4.0+59') < AppVersion.parse('1.4.0+60'),
        isTrue,
      );
    });

    test('mốc 1.4.0 (build 0) không lớn hơn bất kỳ 1.4.0+n nào', () {
      expect(AppVersion.parse('1.4.0') <= AppVersion.parse('1.4.0+60'), isTrue);
    });

    test('== và hashCode theo giá trị; 1.4.0 bằng 1.4.0+0', () {
      expect(AppVersion.parse('1.4.0'), AppVersion.parse('1.4.0+0'));
      expect(
        AppVersion.parse('1.4.0').hashCode,
        AppVersion.parse('1.4.0+0').hashCode,
      );
    });

    test(
      'hotfix 1.6.1+88 mới hơn 1.6.0+87; bản cũ đưa lại không "mới hơn"',
      () {
        final broken = AppVersion.parse('1.6.0+87');
        expect(AppVersion.parse('1.6.1+88') > broken, isTrue);
        // Rollback "dựng lại 1.5.3 cũ": build thấp hơn, store sẽ coi là hạ cấp.
        expect(AppVersion.parse('1.5.3+80') > broken, isFalse);
      },
    );

    test('toString luôn có +build', () {
      expect(AppVersion.parse('1.4.0').toString(), '1.4.0+0');
    });
  });
}
