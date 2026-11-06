import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'support.dart';

void main() {
  late Directory repo;
  late String script;

  /// Tạo repo git tạm có `lab/tool/pre-commit` và `lab/lib/a.dart` đã staged.
  Future<void> stage(String name, String content) async {
    File(p.join(repo.path, 'lab', 'lib', name)).writeAsStringSync(content);
    final (code, output) = await run('git', [
      'add',
      '--',
      'lab/lib/$name',
    ], workingDirectory: repo.path);
    expect(code, 0, reason: output);
  }

  Future<(int, String)> hook({String analyze = 'true'}) => run(
    'sh',
    [script],
    workingDirectory: p.join(repo.path, 'lab'),
    environment: {'ANALYZE_CMD': analyze},
  );

  setUp(() async {
    repo = Directory.systemTemp.createTempSync('toi_noi_32_hook');
    addTearDown(() => repo.deleteSync(recursive: true));
    Directory(p.join(repo.path, 'lab', 'tool')).createSync(recursive: true);
    Directory(p.join(repo.path, 'lab', 'lib')).createSync();
    script = p.join(repo.path, 'lab', 'tool', 'pre-commit');
    File('tool/pre-commit').copySync(script);
    final (code, output) = await run('git', [
      'init',
      '-q',
    ], workingDirectory: repo.path);
    expect(code, 0, reason: output);
  });

  test('không có file .dart staged thì bỏ qua', () async {
    final (code, output) = await hook(analyze: 'false');

    expect(code, 0);
    expect(output, contains('không có file .dart nào staged'));
  });

  test('file đã format và phân tích sạch thì cho commit', () async {
    await stage('a.dart', 'void f() {}\n');

    final (code, _) = await hook();

    expect(code, 0);
  });

  test('file chưa format bị chặn và được nêu tên', () async {
    await stage('a.dart', 'void f( ){ }\n');

    final (code, output) = await hook();

    expect(code, 1);
    expect(output, contains('lib/a.dart'));
  });

  test('tên file có dấu cách vẫn được kiểm', () async {
    await stage('co dau cach.dart', 'void f( ){ }\n');

    final (code, output) = await hook();

    expect(code, 1);
    expect(output, contains('lib/co dau cach.dart'));
  });

  test('git gọi hook từ gốc repository: vẫn tìm đúng thư mục lab', () async {
    await stage('a.dart', 'void f( ){ }\n');

    final (code, output) = await run(
      'sh',
      [script],
      workingDirectory: repo.path,
      environment: {'ANALYZE_CMD': 'true'},
    );

    expect(code, 1);
    expect(output, contains('lib/a.dart'));
  });

  test('phân tích thất bại thì chặn dù mọi file đã format', () async {
    await stage('a.dart', 'void f() {}\n');

    final (code, _) = await hook(analyze: 'false');

    expect(code, 1);
  });

  test('file không phải .dart không bị kiểm format', () async {
    await stage('notes.txt', 'void f( ){ }\n');

    final (code, output) = await hook(analyze: 'false');

    expect(code, 0);
    expect(output, contains('không có file .dart nào staged'));
  });

  test('lệnh phân tích mặc định là dart analyze --fatal-infos', () {
    expect(
      File('tool/pre-commit').readAsStringSync(),
      contains(r'${ANALYZE_CMD:-dart analyze --fatal-infos}'),
    );
  });

  // Commit thật, qua hook cài trong repo git tạm (không đụng hook của repo này).
  group('git commit thật', () {
    const gitIdentity = [
      '-c',
      'user.name=Test',
      '-c',
      'user.email=test@example.test',
      '-c',
      'commit.gpgsign=false',
    ];

    Future<(int, String)> git(
      List<String> args, {
      String? cwd,
      bool analyzeOk = true,
    }) => run(
      'git',
      [...gitIdentity, ...args],
      workingDirectory: cwd ?? repo.path,
      environment: {'ANALYZE_CMD': analyzeOk ? 'true' : 'false'},
    );

    Future<bool> hasCommit({String? cwd}) async =>
        (await git(['rev-parse', '--verify', 'HEAD'], cwd: cwd)).$1 == 0;

    setUp(() async {
      final hook = File(p.join(repo.path, '.git', 'hooks', 'pre-commit'))
        ..writeAsStringSync(
          '#!/bin/sh\n'
          'exec sh "\$(git rev-parse --show-toplevel)/lab/tool/pre-commit"\n',
        );
      await run('chmod', ['+x', hook.path]);
    });

    /// Commit nền (bỏ qua hook) chứa script và một file đã format.
    Future<void> baseline() async {
      File(p.join(repo.path, 'lab', 'lib', 'ok.dart'))
          .writeAsStringSync('void f() {}\n');
      await git(['add', '-A']);
      final (code, output) = await git([
        'commit',
        '-q',
        '--no-verify',
        '-m',
        'base',
      ]);
      expect(code, 0, reason: output);
    }

    test('file đã format: commit đi qua', () async {
      File(p.join(repo.path, 'lab', 'lib', 'a.dart'))
          .writeAsStringSync('void f() {}\n');
      await git(['add', 'lab/lib/a.dart']);

      final (code, output) = await git(['commit', '-q', '-m', 'ok']);

      expect(code, 0, reason: output);
      expect(await hasCommit(), isTrue);
    });

    test('file chưa format: commit bị chặn', () async {
      File(p.join(repo.path, 'lab', 'lib', 'a.dart'))
          .writeAsStringSync('void f( ){ }\n');
      await git(['add', 'lab/lib/a.dart']);

      final (code, output) = await git(['commit', '-q', '-m', 'bad']);

      expect(code, isNot(0));
      expect(output, contains('lib/a.dart'));
      expect(await hasCommit(), isFalse);
    });

    test('file đổi tên (R) vẫn bị kiểm', () async {
      File(p.join(repo.path, 'lab', 'lib', 'old.dart'))
          .writeAsStringSync('void f( ){ }\n');
      await git(['add', '-A']);
      await git(['commit', '-q', '--no-verify', '-m', 'base']);
      await git(['mv', 'lab/lib/old.dart', 'lab/lib/renamed.dart']);

      final (code, output) = await git(['commit', '-q', '-m', 'rename']);

      expect(code, isNot(0));
      expect(output, contains('lib/renamed.dart'));
    });

    test('tên file có dấu vẫn bị kiểm', () async {
      File(p.join(repo.path, 'lab', 'lib', 'tiền_cước.dart'))
          .writeAsStringSync('void f( ){ }\n');
      await git(['add', '-A']);

      final (code, output) = await git(['commit', '-q', '-m', 'accent']);

      expect(code, isNot(0));
      expect(output, contains('lib/tiền_cước.dart'));
    });

    test(
      'file có lỗi cú pháp được báo là lỗi cú pháp, không phải chưa format',
      () async {
        File(p.join(repo.path, 'lab', 'lib', 'a.dart'))
            .writeAsStringSync('void f( {\n');
        await git(['add', '-A']);

        final (code, output) = await git(['commit', '-q', '-m', 'syntax']);

        expect(code, isNot(0));
        expect(output, contains('lỗi cú pháp'));
        expect(output, isNot(contains('chưa format')));
      },
    );

    test('phân tích thất bại thì commit bị chặn', () async {
      File(p.join(repo.path, 'lab', 'lib', 'a.dart'))
          .writeAsStringSync('void f() {}\n');
      await git(['add', '-A']);

      final (code, _) = await git([
        'commit',
        '-q',
        '-m',
        'analyze',
      ], analyzeOk: false);

      expect(code, isNot(0));
      expect(await hasCommit(), isFalse);
    });

    test(
      'trong worktree liên kết (GIT_DIR khác) vẫn chặn file chưa format',
      () async {
        await baseline();
        final wt = p.join(repo.path, 'wt');
        final (addCode, addOut) = await git([
          'worktree',
          'add',
          '-q',
          '-b',
          'feature',
          wt,
        ]);
        expect(addCode, 0, reason: addOut);
        File(p.join(wt, 'lab', 'lib', 'a.dart'))
            .writeAsStringSync('void f( ){ }\n');
        await git(['add', 'lab/lib/a.dart'], cwd: wt);

        final (code, output) = await git(['commit', '-q', '-m', 'wt'], cwd: wt);

        expect(code, isNot(0));
        expect(output, contains('lib/a.dart'));
      },
    );
  });
}
