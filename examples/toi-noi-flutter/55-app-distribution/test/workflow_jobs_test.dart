import 'dart:convert';

import 'package:test/test.dart';
import 'package:toi_noi_53/release_plan.dart' show AppKind, Flavor;
import 'package:toi_noi_54/release_audit.dart'
    show ReleaseWorkflow, findRepoRoot;
import 'package:toi_noi_55/toi_noi_55.dart';
import 'package:yaml/yaml.dart';

import 'support.dart';

final _pinnedRef = RegExp(r'@(v\d+\.\d+\.\d+|[0-9a-f]{40})$');

const _distributeStep = 'Distribute to the internal tester group';
const _downloads = ['Download APK', 'Download IPA', 'Download release notes'];

String _run(String job, String step) => '${stepNamed(job, step)['run']}';

ReleaseWorkflow _audit(String text) =>
    ReleaseWorkflow(text, repoRoot: findRepoRoot(labDir));

/// Các kiểm tra của bài 54 áp dụng được cho đoạn dán (không gồm kiểm tra
/// riêng của job build như keychain, keystore, build number, lệnh build).
List<String> _findings(ReleaseWorkflow wf) => [
  for (final f in [
    ...wf.base.checkFlutterPin(),
    ...wf.base.checkActionRefs(),
    ...wf.base.checkNoSecrets(),
    ...wf.checkLeastPrivilege(),
    ...wf.checkTriggers(),
    ...wf.checkSecretScope(),
    ...wf.checkNoExpressionInRun(),
    ...wf.checkSecretLeaks(),
    ...wf.checkSecretGuards(),
    ...wf.checkHygiene(),
  ])
    f.toString(),
];

void main() {
  group('đoạn dán vào workflow bài 54: vị trí và trigger', () {
    test('chỉ thêm hai job notes và distribute', () {
      expect(snippetJobs.keys.toSet(), {'notes', 'distribute'});
    });

    test('mọi job trong needs đều có trong workflow sau khi dán', () {
      final jobs = (mergedWorkflow['jobs'] as YamlMap).keys.toSet();
      expect(
        jobs,
        containsAll(['prepare', 'android', 'ios', 'notes', 'distribute']),
      );
      for (final name in ['notes', 'distribute']) {
        final needs = jobOf(name)['needs'];
        final list = needs is YamlList ? needs.toList() : [needs];
        for (final n in list) {
          expect(jobs, contains(n), reason: '$name needs $n');
        }
      }
    });

    test('distribute đợi cả hai job build, job prepare và job notes', () {
      expect((jobOf('distribute')['needs'] as YamlList).toSet(), {
        'prepare',
        'android',
        'ios',
        'notes',
      });
    });

    test('trigger không đổi: chỉ chạy tay và tag staging-v*', () {
      final before = buildWorkflow['on'] as YamlMap;
      final after = mergedWorkflow['on'] as YamlMap;
      expect(after.keys.toSet(), {'workflow_dispatch', 'push'});
      expect('$after', '$before');
      expect(((after['push'] as YamlMap)['tags'] as YamlList).toList(), [
        'staging-v*',
      ]);
    });

    test('đoạn dán không tự thêm trigger, không tải chéo run, không lấy mã nơi khác', () {
      for (final forbidden in [
        'workflow_run',
        'workflow_dispatch',
        'pull_request',
        'run-id',
        'github-token',
        'repository:',
        'GITHUB_PATH',
      ]) {
        final code = [
          for (final l in jobsSnippetText.split('\n'))
            if (!l.trimLeft().startsWith('#')) l,
        ].join('\n');
        expect(code, isNot(contains(forbidden)), reason: forbidden);
      }
    });

    test(
      'mọi checkout lấy đúng ref của run (không có ref, repository, token)',
      () {
        for (final job in ['notes', 'distribute']) {
          final checkouts = stepsOf(job)
              .where((s) => '${s['uses']}'.startsWith('actions/checkout'));
          expect(checkouts, hasLength(1), reason: job);
          for (final c in checkouts) {
            final with_ = c['with'] as YamlMap;
            expect(with_.keys, isNot(contains('ref')));
            expect(with_.keys, isNot(contains('repository')));
            expect(with_.keys, isNot(contains('token')));
            expect(with_['persist-credentials'], isFalse);
          }
        }
      },
    );

    test('mọi action ghim phiên bản ba số hoặc SHA đầy đủ', () {
      for (final job in ['notes', 'distribute']) {
        for (final s in stepsOf(job)) {
          if (s['uses'] == null) continue;
          expect('${s['uses']}', matches(_pinnedRef), reason: '${s['uses']}');
        }
      }
    });
  });

  group('đoạn dán: job có secret không chạy mã của repository', () {
    test('distribute nằm trong environment staging, runner ghim nhãn', () {
      final job = jobOf('distribute');
      expect(job['environment'], 'staging');
      expect(job['runs-on'], 'ubuntu-24.04');
      expect(job['timeout-minutes'], isA<int>());
    });

    test('notes không có secret và không có environment', () {
      expect('${jobOf('notes')}', isNot(contains('secrets.')));
      expect(jobOf('notes').keys, isNot(contains('environment')));
    });

    test('distribute chỉ chạy lệnh của hệ thống, npm install và firebase', () {
      // dart, flutter, script trong repository đều có thể bị đổi bởi commit
      // đã được tag; ta không để chúng chạy cùng job với khoá service account.
      final forbidden = RegExp(
        r'\b(dart|flutter|bundle|fastlane|gradlew|npm\s+run|npx)\b|(^|\s)(\./|source\s|\.\s)|tool/',
      );
      for (final s in stepsOf('distribute')) {
        final run = '${s['run'] ?? ''}';
        for (final line in logicalLines(run)) {
          expect(
            line,
            isNot(matches(forbidden)),
            reason: '${s['name']}: $line',
          );
        }
        final uses = '${s['uses'] ?? ''}';
        expect(uses, isNot(matches(RegExp(r'^\./'))), reason: uses);
      }
    });

    test('mã repository (dart run) chỉ chạy ở job notes, không có secret', () {
      expect(
        _run('notes', 'Write release notes'),
        contains('dart run bin/release_notes.dart'),
      );
    });

    test('bộ audit workflow của bài 54 không tìm thấy gì sau khi dán', () {
      expect(_findings(_audit(buildWorkflowText)), isEmpty);
      expect(_findings(_audit(mergedWorkflowText)), isEmpty);
    });

    test('audit bài 54 bắt được ca sai trên bản đã dán', () {
      final withPr = mergedWorkflowText.replaceFirst(
        'workflow_dispatch:',
        'pull_request:\n  workflow_dispatch:',
      );
      expect(withPr, isNot(mergedWorkflowText));
      expect(_findings(_audit(withPr)).join('\n'), contains('trigger'));

      final jobLevelSecret = mergedWorkflowText.replaceFirst(
        "      FIREBASE_CLI_VERSION: '15.35.0'",
        "      FIREBASE_CLI_VERSION: '15.35.0'\n      LEAK: \${{ secrets.FIREBASE_SERVICE_ACCOUNT_JSON }}",
      );
      expect(jobLevelSecret, isNot(mergedWorkflowText));
      expect(
        _findings(_audit(jobLevelSecret)).join('\n'),
        contains('secret-scope'),
      );

      final noGuard = mergedWorkflowText.replaceFirst(
        ': "\${FIREBASE_SERVICE_ACCOUNT_JSON:?missing FIREBASE_SERVICE_ACCOUNT_JSON}"',
        ':',
      );
      expect(noGuard, isNot(mergedWorkflowText));
      expect(_findings(_audit(noGuard)).join('\n'), contains('secret-guard'));
    });
  });

  group('đoạn dán: bí mật và xác thực', () {
    test('secret duy nhất là FIREBASE_SERVICE_ACCOUNT_JSON, chỉ trong env của bước run', () {
      final found = <String>{};
      for (final s in stepsOf('distribute')) {
        for (final e in s.entries) {
          if (!'${e.value}'.contains('secrets.')) continue;
          expect(e.key, 'env', reason: '${s['name']}: secret ngoài env');
          expect(s['run'], isNotNull);
          found.add(
            RegExp(r'secrets\.([A-Z_]+)').firstMatch('${e.value}')!.group(1)!,
          );
        }
      }
      expect(found, {'FIREBASE_SERVICE_ACCOUNT_JSON'});
      final others = {
        for (final k in jobOf('distribute').keys)
          if (k != 'steps') k: jobOf('distribute')[k],
      };
      expect('$others', isNot(contains('secrets.')));
    });

    test('không có biểu thức \${{ }} trong thân run', () {
      for (final job in ['notes', 'distribute']) {
        for (final s in stepsOf(job)) {
          expect(
            '${s['run'] ?? ''}',
            isNot(contains(r'${{')),
            reason: '$job: ${s['name']}',
          );
        }
      }
    });

    test('không dùng FIREBASE_TOKEN, login:ci hay --token (đã deprecated)', () {
      // Trang CLI của Firebase ghi login:ci là deprecated và FIREBASE_TOKEN không
      // còn được khuyên dùng; CLI cũng cảnh báo sẽ gỡ (kiểm tại 11/2026).
      for (final forbidden in ['FIREBASE_TOKEN', 'login:ci', '--token']) {
        expect(jobsSnippetText, isNot(contains(forbidden)), reason: forbidden);
      }
    });

    test(
      'xác thực bằng service account qua GOOGLE_APPLICATION_CREDENTIALS',
      () {
        final run = _run('distribute', _distributeStep);
        expect(run, contains('umask 077'));
        expect(
          run,
          contains(
            r'printf '
            "'%s'"
            r' "$FIREBASE_SERVICE_ACCOUNT_JSON" > "$CREDENTIALS_FILE"',
          ),
        );
        expect(
          run,
          contains(
            r'export GOOGLE_APPLICATION_CREDENTIALS="$CREDENTIALS_FILE"',
          ),
        );
        expect(run, contains(r'${FIREBASE_SERVICE_ACCOUNT_JSON:?'));
      },
    );

    test('file khoá ghi vào RUNNER_TEMP và bước cuối xoá đúng file đó, if: always()', () {
      final write = _run('distribute', _distributeStep);
      final last = stepsOf('distribute').last;
      expect(last['name'], 'Remove service account key');
      expect(last['if'], 'always()');
      final path = RegExp(r'CREDENTIALS_FILE="([^"]+)"')
          .firstMatch(write)!
          .group(1)!;
      expect(path, startsWith(r'$RUNNER_TEMP/'));
      expect('${last['run']}', contains(path));
    });

    test('không in, cat hay đẩy khoá ra GITHUB_ENV', () {
      final run = _run('distribute', _distributeStep);
      for (final line in logicalLines(run)) {
        if (!line.contains('FIREBASE_SERVICE_ACCOUNT_JSON') &&
            !line.contains('CREDENTIALS_FILE')) {
          continue;
        }
        expect(
          line,
          isNot(matches(RegExp(r'\b(echo|cat|head|tail)\b'))),
          reason: line,
        );
        expect(line, isNot(contains('GITHUB_ENV')), reason: line);
      }
      expect(run, isNot(matches(RegExp(r'\bset\s+-[a-zA-Z]*x'))));
    });
  });

  group('đoạn dán: lệnh firebase và artifact', () {
    final config = DistributionConfig.parse(
      readNativeConfig('app-distribution.example.json'),
    );
    final run = _run('distribute', _distributeStep);
    final distribute = [
      for (final l in logicalLines(run))
        if (l.startsWith('firebase appdistribution:distribute')) l,
    ];

    test('CLI ghim phiên bản cụ thể và cài bằng chính biến đó', () {
      final env = jobOf('distribute')['env'] as YamlMap;
      expect(env['FIREBASE_CLI_VERSION'], matches(RegExp(r'^\d+\.\d+\.\d+$')));
      expect(
        _run('distribute', 'Install Firebase CLI'),
        r'npm install --global "firebase-tools@$FIREBASE_CLI_VERSION"',
      );
    });

    test(
      'hai lệnh distribute: APK trước, IPA sau, cùng nhóm và cùng notes',
      () {
        expect(distribute, hasLength(2));
        expect(distribute[0], contains(r'"$APK_FILE" --app "$ANDROID_APP_ID"'));
        expect(distribute[1], contains(r'"$IPA_FILE" --app "$IOS_APP_ID"'));
        for (final l in distribute) {
          expect(l, contains(r'--groups "$GROUP"'));
          expect(l, contains(r'--release-notes-file "$NOTES_FILE"'));
        }
      },
    );

    test('cờ và thứ tự khớp DistributionConfig.cliArguments', () {
      final target = config.target(
        AppKind.customer,
        Flavor.staging,
        DistributionPlatform.android,
      );
      final expected = config.cliArguments(
        target,
        binaryPath: 'x.apk',
        releaseNotesFile: 'n.txt',
      );
      final actual = distribute.first.split(RegExp(r'\s+')).skip(1).toList();
      expect(actual.first, expected.first);
      for (final flag in expected.where((a) => a.startsWith('--'))) {
        expect(actual, contains(flag));
        expect(actual.indexOf(flag), expected.indexOf(flag), reason: flag);
      }
    });

    test(
      'đường jq trỏ tới khoá có thật trong app-distribution.example.json',
      () {
        final json = jsonDecode(
          readNativeConfig('app-distribution.example.json'),
        ) as Map<String, Object?>;
        final paths = RegExp(r"jq -er '\.([a-zA-Z.]+)'")
            .allMatches(run)
            .map((m) => m.group(1)!)
            .toList();
        expect(paths, [
          'apps.customer.staging.android.appId',
          'apps.customer.staging.ios.appId',
          'testerGroups.internal.alias',
        ]);
        for (final path in paths) {
          Object? node = json;
          for (final key in path.split('.')) {
            node = (node as Map<String, Object?>)[key];
          }
          expect(node, isA<String>(), reason: path);
        }
      },
    );

    test('chỉ staging: không flavor dev hoặc prod trong artifact, đường jq và lệnh', () {
      expect(jobsSnippetText, isNot(contains('.dev.')));
      expect(jobsSnippetText, isNot(contains('.prod.')));
      expect(jobsSnippetText, isNot(contains('--flavor')));
      for (final step in _downloads) {
        expect(
          '${(stepNamed('distribute', step)['with'] as YamlMap)['name']}',
          contains('-staging-'),
          reason: step,
        );
      }
    });

    String uploadName(YamlMap jobs, String job, String step) {
      final steps = (jobs[job] as YamlMap)['steps'] as YamlList;
      final s =
          steps.singleWhere((s) => (s as YamlMap)['name'] == step) as YamlMap;
      return '${(s['with'] as YamlMap)['name']}';
    }

    String downloadName(String step) =>
        '${(stepNamed('distribute', step)['with'] as YamlMap)['name']}';

    test(
      'tên artifact tải về khớp tên đã upload: IPA (bài 54), APK, notes',
      () {
        expect(
          downloadName('Download IPA'),
          uploadName(buildWorkflow['jobs'] as YamlMap, 'ios', 'Upload IPA'),
        );
        final apkSteps = loadYaml(readNativeConfig(apkStepsPath)) as YamlList;
        final apkUpload = apkSteps.singleWhere(
          (s) => (s as YamlMap)['name'] == 'Upload APK',
        ) as YamlMap;
        expect(
          downloadName('Download APK'),
          '${(apkUpload['with'] as YamlMap)['name']}',
        );
        expect(
          downloadName('Download release notes'),
          uploadName(snippetJobs, 'notes', 'Upload release notes'),
        );
      },
    );

    test('tên artifact có build number do job prepare cấp', () {
      final outputs =
          ((buildWorkflow['jobs'] as YamlMap)['prepare'] as YamlMap)['outputs']
              as YamlMap;
      expect(outputs.keys, contains('build_number'));
      for (final step in _downloads) {
        expect(
          downloadName(step),
          endsWith(r'${{ needs.prepare.outputs.build_number }}'),
        );
      }
    });

    test('file notes được upload và file được đọc có cùng tên', () {
      final upload =
          stepNamed('notes', 'Upload release notes')['with'] as YamlMap;
      expect(upload['path'], r'${{ runner.temp }}/release-notes.txt');
      expect(run, contains('NOTES_FILE="dist/notes/release-notes.txt"'));
    });

    test('đường dẫn APK, IPA khớp thư mục tải về', () {
      expect(run, contains('APK_FILE="dist/apk/app-staging-release.apk"'));
      expect(run, contains("find dist/ipa -name '*.ipa'"));
      for (final (step, dir) in [
        ('Download APK', 'dist/apk'),
        ('Download IPA', 'dist/ipa'),
        ('Download release notes', 'dist/notes'),
      ]) {
        expect((stepNamed('distribute', step)['with'] as YamlMap)['path'], dir);
      }
    });
  });
}
