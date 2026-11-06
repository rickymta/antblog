import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'support.dart';

const _longRun = Timeout(Duration(minutes: 5));

/// Sao chép fixture [name] vào thư mục tạm và chạy `dart pub get` ở đó.
Future<Directory> _tempFixture(String name) async {
  final dir = Directory.systemTemp.createTempSync('toi_noi_32_$name');
  addTearDown(() => dir.deleteSync(recursive: true));
  copyDirectory(Directory('fixtures/$name'), dir);
  final (code, output) = await run('dart', [
    'pub',
    'get',
  ], workingDirectory: dir.path);
  expect(code, 0, reason: output);
  return dir;
}

void main() {
  group('dart format theo language version', () {
    // Chữ ký dài đủ để formatter phải ngắt dòng.
    const source = '''
Future<int> quote(String request, {required String intentId, required Duration timeout}) async {
  return 1;
}
''';

    Future<String> formatted(String sdkConstraint) async {
      final dir = Directory.systemTemp.createTempSync('toi_noi_32_fmt');
      addTearDown(() => dir.deleteSync(recursive: true));
      File(
        p.join(dir.path, 'pubspec.yaml'),
      ).writeAsStringSync('name: fmt\nenvironment:\n  sdk: $sdkConstraint\n');
      Directory(p.join(dir.path, 'lib')).createSync();
      final file = File(p.join(dir.path, 'lib', 'quote.dart'))
        ..writeAsStringSync(source);
      expect(
        (await run('dart', ['pub', 'get'], workingDirectory: dir.path)).$1,
        0,
      );
      final (code, output) = await run('dart', [
        'format',
        '--output=show',
        file.path,
      ]);
      expect(code, 0, reason: output);
      return output;
    }

    test('sdk ^3.6.0 giữ kiểu cũ: tham số dồn một dòng', () async {
      final out = await formatted('^3.6.0');

      expect(out, contains('Future<int> quote(String request,\n'));
      expect(
        out,
        contains(
          '    {required String intentId, required Duration timeout}) async {',
        ),
      );
    }, timeout: _longRun);

    // 3.7 là ranh giới: từ đây formatter dùng kiểu mới.
    for (final sdk in ['^3.7.0', '^3.13.3']) {
      test(
        'sdk $sdk dùng kiểu mới: mỗi tham số một dòng, có dấu phẩy cuối',
        () async {
          final out = await formatted(sdk);

          expect(out, contains('Future<int> quote(\n  String request, {\n'));
          expect(out, contains('  required Duration timeout,\n}) async {'));
        },
        timeout: _longRun,
      );
    }

    test(
      'đổi sdk trong pubspec mà chưa pub get thì kiểu format chưa đổi',
      () async {
        final dir = Directory.systemTemp.createTempSync('toi_noi_32_stale');
        addTearDown(() => dir.deleteSync(recursive: true));
        final pubspec = File(p.join(dir.path, 'pubspec.yaml'))
          ..writeAsStringSync('name: fmt\nenvironment:\n  sdk: ^3.6.0\n');
        Directory(p.join(dir.path, 'lib')).createSync();
        final file = File(p.join(dir.path, 'lib', 'quote.dart'))
          ..writeAsStringSync(source);
        await run('dart', ['pub', 'get'], workingDirectory: dir.path);

        pubspec.writeAsStringSync('name: fmt\nenvironment:\n  sdk: ^3.13.3\n');
        final (_, stale) = await run('dart', [
          'format',
          '--output=show',
          file.path,
        ]);
        await run('dart', ['pub', 'get'], workingDirectory: dir.path);
        final (_, fresh) = await run('dart', [
          'format',
          '--output=show',
          file.path,
        ]);

        expect(stale, contains('Future<int> quote(String request,\n'));
        expect(fresh, contains('Future<int> quote(\n  String request, {\n'));
      },
      timeout: _longRun,
    );

    test('--set-exit-if-changed thoát 1 khi file chưa format', () async {
      final dir = Directory.systemTemp.createTempSync('toi_noi_32_exit');
      addTearDown(() => dir.deleteSync(recursive: true));
      final file = File(p.join(dir.path, 'a.dart'))
        ..writeAsStringSync('void f( ){ }\n');

      final (code, _) = await run('dart', [
        'format',
        '--output=none',
        '--set-exit-if-changed',
        file.path,
      ]);
      expect(code, 1);
      expect(
        file.readAsStringSync(),
        'void f( ){ }\n',
        reason: 'output=none không ghi file',
      );

      await run('dart', ['format', file.path]);
      final (afterCode, _) = await run('dart', [
        'format',
        '--output=none',
        '--set-exit-if-changed',
        file.path,
      ]);
      expect(afterCode, 0);
    }, timeout: _longRun);
  });

  group('dart fix', () {
    test(
      '--dry-run chỉ báo, --apply sửa 4 lint và để lại một cách viết dở',
      () async {
        final dir = await _tempFixture('fixable');
        final file = File(p.join(dir.path, 'lib', 'fixable.dart'));
        final original = file.readAsStringSync();

        final before = await analyze(dir.path);
        expect(
          before.keys,
          unorderedEquals([
            'directives_ordering@lib/fixable.dart:2',
            'prefer_final_locals@lib/fixable.dart:10',
            'prefer_single_quotes@lib/fixable.dart:10',
            'unnecessary_this@lib/fixable.dart:10',
          ]),
        );

        final (dryCode, dryOut) = await run('dart', [
          'fix',
          '--dry-run',
        ], workingDirectory: dir.path);
        expect(dryCode, 0);
        expect(dryOut, contains('4 proposed fixes in 1 file'));
        expect(
          file.readAsStringSync(),
          original,
          reason: 'dry-run không ghi file',
        );

        final (applyCode, applyOut) = await run('dart', [
          'fix',
          '--apply',
        ], workingDirectory: dir.path);
        expect(applyCode, 0);
        expect(applyOut, contains('4 fixes made in 1 file'));

        final fixed = file.readAsStringSync();
        expect(
          fixed,
          startsWith("import 'dart:async';\nimport 'dart:math';\n"),
        );
        expect(fixed, contains("final text = '\${vnd} VND';"));
        expect((await analyze(dir.path)).diagnostics, isEmpty);
      },
      timeout: _longRun,
    );
  });

  group('// ignore có lý do', () {
    test('document_ignores và unnecessary_ignore là lint mức info', () async {
      final dir = await _tempFixture('ignores');

      final result = await analyze(dir.path);
      expect(
        result.keys,
        unorderedEquals([
          'document_ignores@lib/ignores.dart:1',
          'unnecessary_ignore@lib/ignores.dart:11',
        ]),
      );
      expect(result.exitCode, 0, reason: 'info không làm analyze thất bại');

      final strict = await analyze(dir.path, flags: ['--fatal-infos']);
      expect(
        strict.exitCode,
        1,
        reason: '--fatal-infos biến info thành thất bại',
      );
    }, timeout: _longRun);

    test('ignore có lý do (dòng trên hoặc cùng dòng) vẫn chặn đúng lint đó', () async {
      final dir = await _tempFixture('ignores');

      final result = await analyze(dir.path);
      final lines = {for (final d in result.diagnostics) d.line};
      // Dòng 4-6 (lý do ở trên) và 8 (lý do cùng dòng) không có chẩn đoán nào,
      // kể cả unused_element: ignore đã chặn nó.
      expect(lines.intersection({4, 5, 6, 8}), isEmpty);
      expect(
        result.diagnostics.where((d) => d.code == 'unused_element'),
        isEmpty,
      );
    }, timeout: _longRun);
  });
}
