import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';
import 'package:toi_noi_55/toi_noi_55.dart';

import 'support.dart';

const _okArgs = ['--app', 'customer', '--version', '1.4.0', '--build', '42'];

List<String> gitLog() => [
  for (final c in sampleCommits()) '${c.sha}\t${c.subject}',
];

void main() {
  group('runCli', () {
    test('đọc log dạng sha<TAB>subject và trả notes cho app khách', () {
      final result = runCli(_okArgs, gitLog());
      expect(result.notes.text, startsWith('Tới Nơi khách · staging 1.4.0 '));
      expect(result.notes.included, 4);
      expect(result.summary, contains('4 dòng'));
      expect(result.summary, contains('8 commit nội bộ bị bỏ'));
      // Tóm tắt cho log CI không chứa nội dung commit.
      expect(result.summary, isNot(contains('đếm ngược')));
    });

    test('bỏ qua dòng trống ở cuối log', () {
      final result = runCli(_okArgs, [...gitLog(), '', '  ']);
      expect(result.notes.included, 4);
    });

    test('--max được áp dụng', () {
      final result = runCli([..._okArgs, '--max', '120'], gitLog());
      expect(result.notes.length, lessThanOrEqualTo(120));
    });

    test('--flavor dev hoặc prod bị từ chối: chỉ staging được phân phối', () {
      for (final flavor in ['dev', 'prod']) {
        expect(
          () => runCli([..._okArgs, '--flavor', flavor], gitLog()),
          throwsA(
            isA<UsageError>().having(
              (e) => e.message,
              'message',
              contains('chỉ staging'),
            ),
          ),
          reason: flavor,
        );
      }
    });

    test(
      'thiếu tham số bắt buộc, app lạ, số không hợp lệ đều là UsageError',
      () {
        final bad = <List<String>>[
          ['--app', 'customer', '--version', '1.4.0'],
          ['--version', '1.4.0', '--build', '42'],
          ['--app', 'admin', '--version', '1.4.0', '--build', '42'],
          ['--app', 'customer', '--version', '1.4.0', '--build', 'abc'],
          [..._okArgs, '--max', 'nhiều'],
          [..._okArgs, '--max', '10'],
          ['--app', 'customer', '--version', 'v1', '--build', '42'],
          ['--app', 'customer', '--version', '1.4.0', '--build', '0'],
          [..._okArgs, '--khong-co'],
        ];
        for (final args in bad) {
          expect(
            () => runCli(args, gitLog()),
            throwsA(isA<UsageError>()),
            reason: args.join(' '),
          );
        }
      },
    );
  });

  group('bin/release_notes.dart (chạy tiến trình thật)', () {
    Future<ProcessResult> run(List<String> args, String stdinText) async {
      final process = await Process.start(Platform.resolvedExecutable, [
        'run',
        'bin/release_notes.dart',
        ...args,
      ], workingDirectory: labDir.path);
      process.stdin.write(stdinText);
      await process.stdin.close();
      final out = process.stdout.transform(utf8.decoder).join();
      final err = process.stderr.transform(utf8.decoder).join();
      final code = await process.exitCode;
      return ProcessResult(process.pid, code, await out, await err);
    }

    test(
      'stdin là git log, stdout là notes UTF-8, stderr là tóm tắt',
      () async {
        final result = await run(_okArgs, '${gitLog().join('\n')}\n');
        expect(result.exitCode, 0);
        expect(result.stdout, contains('Tính năng mới'));
        expect(
          result.stdout,
          contains('Cước 2,001 km hiển thị đúng 12.005 VND'),
        );
        expect(result.stderr, contains('Release notes: 4 dòng'));
      },
    );

    test('flavor prod thoát với mã 64 và không in notes', () async {
      final result = await run([..._okArgs, '--flavor', 'prod'], '');
      expect(result.exitCode, 64);
      expect(result.stdout, isEmpty);
      expect(result.stderr, contains('chỉ staging'));
    });
  });
}
