import 'dart:io';

import 'package:test/test.dart';
import 'package:toi_noi_56/src/cli.dart';

import 'support.dart';

({int code, String out, String err}) run(List<String> args) {
  final out = StringBuffer();
  final err = StringBuffer();
  final code = runBetaCheck(args, out, err);
  return (code: code, out: out.toString(), err: err.toString());
}

void main() {
  late Directory tmp;

  setUp(() => tmp = Directory.systemTemp.createTempSync('toi_noi_56_cli_'));
  tearDown(() => tmp.deleteSync(recursive: true));

  test('bản mẫu của app khách đạt: thoát 0', () {
    final r = run([
      'customer',
      customerPlist,
      '--build-name=1.4.0',
      '--build-number=43',
      '--last-build=42',
      '--entitlements=$entitlementsFile',
    ]);
    expect(r.code, 0);
    expect(r.out, contains('Không có lỗi (0 cảnh báo).'));
    expect(r.err, isEmpty);
  });

  test('build number không tăng: thoát 1 và in mã lỗi', () {
    final r = run([
      'driver',
      driverPlist,
      '--build-name=1.4.0',
      '--build-number=42',
      '--last-build=42',
    ]);
    expect(r.code, 1);
    expect(r.out, contains('build-not-increasing'));
    expect(r.out, contains('1 lỗi, 0 cảnh báo.'));
  });

  test('chỉ có cảnh báo thì vẫn thoát 0', () {
    final plist = File('${tmp.path}/Info.plist')
      ..writeAsStringSync(
        readText(customerPlist).replaceAll(
          RegExp(r'\s*<key>ITSAppUsesNonExemptEncryption</key>\s*<false/>'),
          '',
        ),
      );
    final r = run([
      'customer',
      plist.path,
      '--build-name=1.4.0',
      '--build-number=43',
    ]);
    expect(r.code, 0);
    expect(r.out, contains('encryption-key-missing'));
    expect(r.out, contains('(1 cảnh báo)'));
  });

  test('không truyền biến build: lỗi version-unresolved, thoát 1', () {
    final r = run(['customer', customerPlist]);
    expect(r.code, 1);
    expect(r.out, contains('version-unresolved'));
  });

  test('entitlements thiếu aps-environment: thoát 1', () {
    final ent = File('${tmp.path}/e.plist')
      ..writeAsStringSync(
        '<?xml version="1.0"?><plist version="1.0"><dict/></plist>',
      );
    final r = run([
      'customer',
      customerPlist,
      '--build-name=1.4.0',
      '--build-number=43',
      '--entitlements=${ent.path}',
    ]);
    expect(r.code, 1);
    expect(r.out, contains('push-entitlement-missing'));
  });

  group('dùng sai: thoát 2', () {
    test('thiếu đối số', () {
      expect(run([]).code, 2);
      expect(run(['customer']).code, 2);
      expect(run(['customer', customerPlist, 'thừa']).code, 2);
    });

    test('vai trò lạ', () {
      final r = run(['admin', customerPlist]);
      expect(r.code, 2);
      expect(r.err, contains('customer hoặc driver'));
    });

    test('tuỳ chọn thiếu giá trị hoặc không biết', () {
      expect(run(['customer', customerPlist, '--build-number']).code, 2);
      final r = run(['customer', customerPlist, '--ban-build=3']);
      expect(r.code, 2);
      expect(r.err, contains('--ban-build'));
    });

    test('file không tồn tại', () {
      final r = run(['customer', '${tmp.path}/khong-co.plist']);
      expect(r.code, 2);
      expect(r.err, contains('khong-co.plist'));
    });

    test('file không phải plist', () {
      final bad = File('${tmp.path}/bad.plist')
        ..writeAsStringSync('không phải xml');
      final r = run(['customer', bad.path]);
      expect(r.code, 2);
      expect(r.err, contains('plist'));
    });

    test('--last-build sai dạng', () {
      final r = run([
        'customer',
        customerPlist,
        '--build-name=1.4.0',
        '--build-number=43',
        '--last-build=abc',
      ]);
      expect(r.code, 2);
      expect(r.err, contains('không hợp lệ'));
    });
  });
}
