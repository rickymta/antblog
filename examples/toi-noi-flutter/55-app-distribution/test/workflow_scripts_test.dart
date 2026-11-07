import 'dart:io';

import 'package:test/test.dart';

import 'support.dart';

/// Giá trị giả của secret, ghép từ mảnh để không giống bí mật thật.
final String fakeServiceAccount = '{"type": "fake", "note": "${'x' * 12}"}';

/// Sandbox giống một runner: repo git, `runner_temp/`, `bin/` chứa lệnh giả.
class Sandbox {
  Sandbox() : dir = Directory.systemTemp.createTempSync('toi_noi_55_') {
    for (final sub in ['runner_temp', 'bin', 'repo']) {
      Directory('${dir.path}/$sub').createSync(recursive: true);
    }
  }

  final Directory dir;

  String get repo => '${dir.path}/repo';
  String get temp => '${dir.path}/runner_temp';
  String get log => '${dir.path}/stub.log';

  File get githubOutput => File('${dir.path}/github_output');
  File get stepSummary => File('${dir.path}/step_summary');

  /// Ghi lệnh giả vào `bin/<name>` (bash).
  void stub(String name, String body) {
    final file = File('${dir.path}/bin/$name')
      ..writeAsStringSync('#!/bin/bash\n$body\n');
    Process.runSync('chmod', ['+x', file.path]);
  }

  Map<String, String> env([Map<String, String> extra = const {}]) => {
    'PATH':
        '${dir.path}/bin:${Platform.environment['PATH'] ?? '/usr/bin:/bin'}',
    'HOME': '${dir.path}/home',
    'RUNNER_TEMP': temp,
    'GITHUB_OUTPUT': githubOutput.path,
    'GITHUB_STEP_SUMMARY': stepSummary.path,
    'GITHUB_REPOSITORY': 'toinoi-example/toi-noi-app',
    'STUB_LOG': log,
    ...extra,
  };

  RunResult run(String script, [Map<String, String> extra = const {}]) =>
      runScript(script, env: env(extra), workingDirectory: Directory(repo));

  String git(List<String> args) {
    final result = Process.runSync(
      'git',
      [
        '-c',
        'user.name=Test',
        '-c',
        'user.email=test@example.com',
        '-c',
        'commit.gpgsign=false',
        ...args,
      ],
      workingDirectory: repo,
      environment: {
        'GIT_CONFIG_GLOBAL': '/dev/null',
        'GIT_CONFIG_SYSTEM': '/dev/null',
      },
    );
    if (result.exitCode != 0) {
      throw StateError('git ${args.join(' ')}: ${result.stderr}');
    }
    return (result.stdout as String).trim();
  }

  void commit(String subject) =>
      git(['commit', '--allow-empty', '-m', subject]);

  void delete() => dir.deleteSync(recursive: true);
}

String scriptOf(String job, String step) => '${stepNamed(job, step)['run']}';

void main() {
  final skip = canRunScripts ? false : 'cần bash, git và jq';

  group('job notes: bước Write release notes', () {
    late Sandbox box;
    late String headSha;
    const step = 'Write release notes';

    /// `dart` giả: ghi tham số, và `dart run` chép stdin ra stdout.
    void stubDart() => box.stub('dart', r'''
echo "$@" >> "$STUB_LOG"
if [ "$1" = "run" ]; then cat; fi
''');

    Map<String, String> envFor({String build = '42'}) => {
      'BUILD_NUMBER': build,
      'BUILD_SHA': headSha,
    };

    RunResult runStep([Map<String, String>? env]) =>
        box.run(scriptOf('notes', step), env ?? envFor());

    setUp(() {
      box = Sandbox();
      stubDart();
      Directory('${box.repo}/tool/release_notes').createSync(recursive: true);
      Directory('${box.repo}/apps/customer').createSync(recursive: true);
      File('${box.repo}/apps/customer/pubspec.yaml')
          .writeAsStringSync('name: customer\nversion: 1.4.0+7\n');
      box.git(['init', '-q']);
      box.commit('feat: một');
      box.commit('fix: hai');
      box.git(['tag', 'staging-v1.0.0']);
      box.commit('feat: ba');
      box.commit('chore: bốn');
      headSha = box.git(['rev-parse', 'HEAD']);
    });
    tearDown(() => box.delete());

    List<String> commitsFed() =>
        File('${box.temp}/commits.tsv')
            .readAsLinesSync()
            .map((l) => l.split('\t').last)
            .toList();

    test('lấy commit từ tag staging-v* gần nhất tới commit đã dựng', () {
      final r = runStep();
      expect(r.exitCode, 0, reason: r.log);
      expect(commitsFed(), ['chore: bốn', 'feat: ba']);
    });

    test('commit đã dựng nằm trên tag mới: vẫn lấy từ tag trước đó', () {
      box.git(['tag', 'staging-v1.1.0']);
      final r = runStep();
      expect(r.exitCode, 0, reason: r.log);
      expect(commitsFed(), ['chore: bốn', 'feat: ba']);
    });

    test('loại commit merge khỏi log', () {
      box.git(['checkout', '-q', '-b', 'side', 'staging-v1.0.0']);
      box.commit('feat: nhánh phụ');
      box.git(['checkout', '-q', '-']);
      box.git(['merge', '--no-ff', '-q', '-m', 'Merge side', 'side']);
      headSha = box.git(['rev-parse', 'HEAD']);
      final r = runStep();
      expect(r.exitCode, 0, reason: r.log);
      expect(commitsFed(), isNot(contains('Merge side')));
      expect(commitsFed(), contains('feat: nhánh phụ'));
    });

    test('chưa có tag nào: lấy tối đa 30 commit cuối', () {
      box.git(['tag', '-d', 'staging-v1.0.0']);
      for (var i = 0; i < 35; i++) {
        box.commit('fix: sửa $i');
      }
      headSha = box.git(['rev-parse', 'HEAD']);
      final r = runStep();
      expect(r.exitCode, 0, reason: r.log);
      expect(commitsFed(), hasLength(30));
      expect(commitsFed().first, 'fix: sửa 34');
    });

    test(
      'truyền đúng app, version từ pubspec và build number của job prepare',
      () {
        final r = runStep();
        expect(r.exitCode, 0, reason: r.log);
        final calls = File(box.log).readAsLinesSync();
        expect(calls, contains('pub get'));
        expect(
          calls,
          contains(
            'run bin/release_notes.dart --app customer --version 1.4.0 --build 42',
          ),
        );
      },
    );

    test('ghi notes ra RUNNER_TEMP và vào step summary', () {
      final r = runStep();
      expect(r.exitCode, 0, reason: r.log);
      final notes = File('${box.temp}/release-notes.txt').readAsStringSync();
      expect(notes, contains('feat: ba'));
      expect(box.stepSummary.readAsStringSync(), notes);
    });

    test('build number không phải số nguyên: dừng, không sinh notes', () {
      for (final bad in ['', 'abc', '4 2', r'$(id)', '-1']) {
        final r = runStep(envFor(build: bad));
        expect(r.exitCode, isNot(0), reason: '"$bad"');
        expect(r.stderr, contains('positive integer'));
      }
      expect(File('${box.temp}/release-notes.txt').existsSync(), isFalse);
    });

    test('pubspec không có version dạng x.y.z: dừng với thông báo rõ', () {
      File('${box.repo}/apps/customer/pubspec.yaml')
          .writeAsStringSync('name: customer\nversion: 1.4\n');
      final r = runStep();
      expect(r.exitCode, isNot(0));
      expect(r.stderr, contains('cannot read version'));
    });
  }, skip: skip);

  group('job distribute: bước Distribute to the internal tester group', () {
    late Sandbox box;
    const step = 'Distribute to the internal tester group';
    const apk = 'dist/apk/app-staging-release.apk';
    const ipa = 'dist/ipa/Runner.ipa';
    const notes = 'dist/notes/release-notes.txt';

    /// `firebase` giả: ghi tham số, đường dẫn khoá, quyền và nội dung khoá ở
    /// thời điểm được gọi; thất bại khi `$STUB_FAIL_ON` khớp file nhị phân.
    void stubFirebase() => box.stub('firebase', r'''
echo "ARGS $*" >> "$STUB_LOG"
echo "CRED $GOOGLE_APPLICATION_CREDENTIALS" >> "$STUB_LOG"
echo "MODE $(stat -f %Lp "$GOOGLE_APPLICATION_CREDENTIALS" 2>/dev/null || stat -c %a "$GOOGLE_APPLICATION_CREDENTIALS")" >> "$STUB_LOG"
echo "BODY $(cat "$GOOGLE_APPLICATION_CREDENTIALS")" >> "$STUB_LOG"
if [ "${STUB_FAIL_ON:-}" = "$2" ]; then exit 1; fi
''');

    void touch(String relative) => File('${box.repo}/$relative')
      ..createSync(recursive: true)
      ..writeAsStringSync('x');

    Map<String, String> envFor() => {
      'FIREBASE_SERVICE_ACCOUNT_JSON': fakeServiceAccount,
    };

    setUp(() {
      box = Sandbox();
      stubFirebase();
      touch(apk);
      touch(ipa);
      touch(notes);
      File('${box.repo}/app-distribution.json')
          .writeAsStringSync(readNativeConfig('app-distribution.example.json'));
    });
    tearDown(() => box.delete());

    List<String> calls(String prefix) => [
      for (final l in File(box.log).readAsLinesSync())
        if (l.startsWith('$prefix ')) l.substring(prefix.length + 1),
    ];

    test(
      'gọi firebase hai lần: APK với App ID Android, IPA với App ID iOS',
      () {
        final r = box.run(scriptOf('distribute', step), envFor());
        expect(r.exitCode, 0, reason: r.log);
        expect(calls('ARGS'), [
          'appdistribution:distribute $apk '
              '--app 1:000000000000:android:FAKE-STAGING-REPLACE-ME '
              '--groups qa-noi-bo --release-notes-file $notes',
          'appdistribution:distribute $ipa '
              '--app 1:000000000000:ios:FAKE-STAGING-REPLACE-ME '
              '--groups qa-noi-bo --release-notes-file $notes',
        ]);
      },
    );

    test(
      'khoá service account có đúng nội dung, quyền 600, nằm trong RUNNER_TEMP',
      () {
        final r = box.run(scriptOf('distribute', step), envFor());
        expect(r.exitCode, 0, reason: r.log);
        for (final cred in calls('CRED')) {
          expect(cred, '${box.temp}/firebase-service-account.json');
        }
        expect(calls('MODE'), ['600', '600']);
        expect(calls('BODY'), [fakeServiceAccount, fakeServiceAccount]);
      },
    );

    test('giá trị secret không xuất hiện trong log của job', () {
      final r = box.run(scriptOf('distribute', step), envFor());
      expect(r.exitCode, 0, reason: r.log);
      expect(r.log, isNot(contains(fakeServiceAccount)));
      expect(r.log, isNot(contains('x' * 12)));
    });

    test('script không tự đặt FIREBASE_TOKEN cho firebase', () {
      box.stub('firebase', r'''
echo "TOKEN [${FIREBASE_TOKEN:-unset}]" >> "$STUB_LOG"
''');
      final r = box.run(scriptOf('distribute', step), envFor());
      expect(r.exitCode, 0, reason: r.log);
      expect(calls('TOKEN'), ['[unset]', '[unset]']);
    });

    test(
      'secret trống hoặc không đặt: dừng, không ghi khoá, không gọi firebase',
      () {
        for (final env in [
          <String, String>{},
          {'FIREBASE_SERVICE_ACCOUNT_JSON': ''},
        ]) {
          final r = box.run(scriptOf('distribute', step), env);
          expect(r.exitCode, isNot(0));
          expect(r.stderr, contains('missing FIREBASE_SERVICE_ACCOUNT_JSON'));
        }
        expect(File(box.log).existsSync(), isFalse);
        expect(
          File('${box.temp}/firebase-service-account.json').existsSync(),
          isFalse,
        );
      },
    );

    test('thiếu file IPA, APK hoặc notes: dừng trước khi ghi khoá', () {
      for (final missing in [apk, ipa, notes]) {
        File('${box.repo}/$missing').deleteSync();
        final r = box.run(scriptOf('distribute', step), envFor());
        expect(r.exitCode, isNot(0), reason: missing);
        expect(r.stderr, contains('missing'));
        expect(
          File('${box.temp}/firebase-service-account.json').existsSync(),
          isFalse,
        );
        touch(missing);
      }
      expect(File(box.log).existsSync(), isFalse);
    });

    test('APK thất bại thì dừng ngay, không phân phối IPA; khoá còn lại cho bước dọn', () {
      final r = box.run(scriptOf('distribute', step), {
        ...envFor(),
        'STUB_FAIL_ON': apk,
      });
      expect(r.exitCode, isNot(0));
      expect(calls('ARGS'), hasLength(1));
      expect(
        File('${box.temp}/firebase-service-account.json').existsSync(),
        isTrue,
      );

      final cleanup = box.run(
        scriptOf('distribute', 'Remove service account key'),
      );
      expect(cleanup.exitCode, 0, reason: cleanup.log);
      expect(
        File('${box.temp}/firebase-service-account.json').existsSync(),
        isFalse,
      );
    });

    test('app-distribution.json thiếu khoá: dừng trước khi gọi firebase', () {
      File('${box.repo}/app-distribution.json')
          .writeAsStringSync('{"apps": {}}');
      final r = box.run(scriptOf('distribute', step), envFor());
      expect(r.exitCode, isNot(0));
      expect(File(box.log).existsSync(), isFalse);
    });

    test('bước dọn chạy được khi chưa có khoá (idempotent)', () {
      final r = box.run(scriptOf('distribute', 'Remove service account key'));
      expect(r.exitCode, 0, reason: r.log);
    });
  }, skip: skip);
}
