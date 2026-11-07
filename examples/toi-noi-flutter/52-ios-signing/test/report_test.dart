import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';
import 'package:toi_noi_52/toi_noi_52.dart';

import 'support.dart';

void main() {
  group('sniffProfileFile', () {
    test('plist XML, kể cả có BOM và khoảng trắng đầu', () {
      expect(
        sniffProfileFile(utf8.encode('  \n<?xml version="1.0"?>')),
        ProfileFileFormat.xmlPlist,
      );
      expect(
        sniffProfileFile([0xEF, 0xBB, 0xBF, ...utf8.encode('<plist>')]),
        ProfileFileFormat.xmlPlist,
      );
    });

    test('CMS bắt đầu bằng SEQUENCE 0x30', () {
      expect(sniffProfileFile([0x30, 0x82, 0x1A, 0x2B]), ProfileFileFormat.cms);
    });

    test('plist nhị phân và file rỗng là unknown', () {
      expect(
        sniffProfileFile(utf8.encode('bplist00')),
        ProfileFileFormat.unknown,
      );
      expect(sniffProfileFile(const []), ProfileFileFormat.unknown);
      expect(sniffProfileFile(utf8.encode('   ')), ProfileFileFormat.unknown);
    });

    test('hướng dẫn nhắc đúng lệnh security cms', () {
      expect(cmsHint, contains('security cms -D -i'));
    });
  });

  group('inspectProfile', () {
    final now = DateTime.utc(2026, 11, 7);
    final claimed = configPlist('Runner.entitlements.example').keys;

    test('profile tốt: không cảnh báo', () {
      final report = inspectProfile(
        sample('staging-adhoc'),
        now: now,
        claimed: claimed,
        udid: '00008030-000000000000000A',
      );
      expect(report.hasWarnings, isFalse);
      expect(report.toString(), contains('Loại:       ad hoc (suy ra)'));
      expect(report.toString(), contains('Thiết bị:   3'));
    });

    test('profile cũ: ba cảnh báo cùng lúc', () {
      final report = inspectProfile(
        sample('staging-adhoc-old'),
        now: now,
        claimed: claimed,
        udid: '00008030-000000000000000C',
      );
      expect(report.warnings, hasLength(3));
      expect(report.warnings[0], contains('còn 13 ngày'));
      expect(report.warnings[1], contains('aps-environment'));
      expect(report.warnings[2], contains('00008030-000000000000000C'));
    });

    test('profile hết hạn có lời nhắc tạo lại', () {
      final report = inspectProfile(
        sample('staging-adhoc-old'),
        now: DateTime.utc(2026, 12, 1),
      );
      expect(report.warnings.single, contains('đã hết hạn'));
    });

    test('App Store: báo không có danh sách thiết bị', () {
      final report = inspectProfile(sample('prod-appstore'), now: now);
      expect(report.toString(), contains('không có danh sách'));
    });
  });

  group('bin/inspect_profile.dart', () {
    Future<ProcessResult> run(List<String> args) => Process.run(
      Platform.resolvedExecutable,
      ['bin/inspect_profile.dart', ...args],
    );

    test('profile ổn thoát 0', () async {
      final r = await run([
        'samples/staging-adhoc.plist',
        '--now=2026-11-07T00:00:00Z',
      ]);
      expect(r.exitCode, 0, reason: '${r.stderr}');
      expect(r.stdout, contains('ToiNoi Customer Staging AdHoc'));
    });

    test('profile sắp hết hạn thoát 1', () async {
      final r = await run([
        'samples/staging-adhoc-old.plist',
        '--now=2026-11-07T00:00:00Z',
      ]);
      expect(r.exitCode, 1);
      expect(r.stdout, contains('CẢNH BÁO'));
    });

    test('file CMS thoát 2 kèm hướng dẫn security cms', () async {
      final dir = Directory.systemTemp.createTempSync('toi_noi_52_');
      addTearDown(() => dir.deleteSync(recursive: true));
      final file = File('${dir.path}/fake.mobileprovision')
        ..writeAsBytesSync([0x30, 0x82, 0x01, 0x00, 0x06, 0x09]);
      final r = await run([file.path]);
      expect(r.exitCode, 2);
      expect(r.stderr, contains('security cms -D -i'));
    });

    test('thiếu đối số thoát 2', () async {
      final r = await run([]);
      expect(r.exitCode, 2);
      expect(r.stderr, contains('Dùng:'));
    });
  });
}
