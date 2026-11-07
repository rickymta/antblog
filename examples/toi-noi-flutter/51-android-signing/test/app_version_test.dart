import 'dart:io';

import 'package:test/test.dart';
import 'package:toi_noi_51/app_version.dart';
import 'package:toi_noi_51/version_check_cli.dart';

void main() {
  group('AppVersion.parse', () {
    test('tách versionName và versionCode từ x.y.z+build', () {
      final v = AppVersion.parse('1.4.2+37');
      expect(v.versionName, '1.4.2');
      expect(v.versionCode, 37);
      expect(v.toString(), '1.4.2+37');
    });

    test('giữ hậu tố tiền phát hành trong versionName', () {
      expect(AppVersion.parse('2.0.0-beta.1+120').versionName, '2.0.0-beta.1');
    });

    test('build là số, không phải số phiên bản: 1.0.0+9 nhỏ hơn 1.0.0+10', () {
      expect(AppVersion.parse('1.0.0+10').versionCode, greaterThan(9));
    });

    for (final bad in [
      '1.4.2', // thiếu +build
      '1.4+37', // thiếu số vá
      '1.4.2+', // build rỗng
      '1.4.2+abc', // build không phải số
      '1.4.2+3.7',
      '1.4.2+-5',
      '',
    ]) {
      test('từ chối "$bad"', () {
        expect(() => AppVersion.parse(bad), throwsFormatException);
      });
    }

    test('build phải từ 1 tới 2.100.000.000', () {
      expect(() => AppVersion.parse('1.0.0+0'), throwsFormatException);
      expect(AppVersion.parse('1.0.0+2100000000').versionCode, maxVersionCode);
      expect(() => AppVersion.parse('1.0.0+2100000001'), throwsFormatException);
    });
  });

  group('AppVersion.fromPubspec', () {
    test('đọc trường version', () {
      const yaml = 'name: toi_noi_customer\nversion: 3.1.0+58\n';
      expect(AppVersion.fromPubspec(yaml).versionCode, 58);
    });

    test('không có version thì lỗi', () {
      expect(
        () => AppVersion.fromPubspec('name: toi_noi_customer\n'),
        throwsFormatException,
      );
    });

    test('version là số (1.0) thay vì chuỗi x.y.z+build thì lỗi', () {
      expect(
        () => AppVersion.fromPubspec('version: 1.0\n'),
        throwsFormatException,
      );
    });
  });

  group('versionCode tăng đơn điệu', () {
    test('lớn hơn bản trước thì qua', () {
      expect(
        () => AppVersion.parse('1.4.3+38').ensureIncreasesFrom(37),
        returnsNormally,
      );
    });

    test('build giảm thì ném VersionCodeRegression, không phải trùng', () {
      expect(
        () => AppVersion.parse('1.5.0+30').ensureIncreasesFrom(37),
        throwsA(
          isA<VersionCodeRegression>()
              .having((e) => e.previous, 'previous', 37)
              .having((e) => e.candidate, 'candidate', 30)
              .having((e) => e.isDuplicate, 'isDuplicate', isFalse),
        ),
      );
    });

    test('build trùng bản trước cũng bị từ chối', () {
      expect(
        () => AppVersion.parse('1.4.2+37').ensureIncreasesFrom(37),
        throwsA(
          isA<VersionCodeRegression>().having(
            (e) => e.isDuplicate,
            'dup',
            isTrue,
          ),
        ),
      );
    });

    test('versionName tăng nhưng build giảm vẫn là lỗi', () {
      // Semver tăng không cứu được versionCode: Play chỉ nhìn versionCode.
      expect(
        () => AppVersion.parse('9.0.0+5').ensureIncreasesFrom(37),
        throwsA(isA<VersionCodeRegression>()),
      );
    });

    test('thông báo nói rõ giảm hay trùng', () {
      expect(
        VersionCodeRegression(previous: 37, candidate: 30).toString(),
        contains('giảm từ 37 xuống 30'),
      );
      expect(
        VersionCodeRegression(previous: 37, candidate: 37).toString(),
        contains('đã được dùng'),
      );
    });
  });

  group('APK tách ABI', () {
    test('Flutter cộng hệ số ABI * 1000 vào versionCode', () {
      final v = AppVersion.parse('1.4.2+37');
      expect(v.splitVersionCode(Abi.armeabiV7a), 1037);
      expect(v.splitVersionCode(Abi.arm64V8a), 2037);
      expect(v.splitVersionCode(Abi.x86_64), 4037);
    });

    test('AAB và APK gộp dùng đúng build, không cộng hệ số', () {
      expect(AppVersion.parse('1.4.2+37').versionCode, 37);
    });

    test('build từ 1000 trở lên làm hai ABI trùng versionCode', () {
      final a = AppVersion.parse('1.0.0+1037').splitVersionCode(Abi.armeabiV7a);
      final b = AppVersion.parse('1.0.0+37').splitVersionCode(Abi.arm64V8a);
      expect(a, b);
    });

    test('vượt giới hạn Play thì ném StateError', () {
      expect(
        () =>
            AppVersion.parse('1.0.0+2100000000').splitVersionCode(Abi.arm64V8a),
        throwsStateError,
      );
    });
  });

  group('check_version_code (CLI)', () {
    late Directory dir;
    late File pubspec;

    setUp(() {
      dir = Directory.systemTemp.createTempSync('tn51-cli');
      pubspec = File('${dir.path}/pubspec.yaml')
        ..writeAsStringSync('name: x\nversion: 1.4.2+37\n');
    });
    tearDown(() => dir.deleteSync(recursive: true));

    int run(List<String> args, StringBuffer out, StringBuffer err) =>
        runVersionCheck(args, out, err);

    test('build tăng: mã 0 và in versionCode', () {
      final out = StringBuffer(), err = StringBuffer();
      expect(run([pubspec.path, '--previous', '36'], out, err), 0);
      expect(out.toString(), contains('versionCode 37'));
      expect(err.toString(), isEmpty);
    });

    test('build giảm: mã 1', () {
      final out = StringBuffer(), err = StringBuffer();
      expect(run([pubspec.path, '--previous', '50'], out, err), 1);
      expect(err.toString(), contains('giảm từ 50 xuống 37'));
    });

    test('dùng sai hoặc file lỗi: mã 2', () {
      final out = StringBuffer(), err = StringBuffer();
      expect(run([pubspec.path], out, err), 2);
      expect(run([pubspec.path, '--previous', 'abc'], out, err), 2);
      expect(
        run(['${dir.path}/khong-co.yaml', '--previous', '1'], out, err),
        2,
      );
      pubspec.writeAsStringSync('version: 1.0.0\n');
      expect(run([pubspec.path, '--previous', '1'], out, err), 2);
    });
  });
}
