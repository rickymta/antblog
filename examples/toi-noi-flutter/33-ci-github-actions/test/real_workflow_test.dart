import 'dart:io';

import 'package:test/test.dart';
import 'package:toi_noi_33/workflow_audit.dart';
import 'package:yaml/yaml.dart';

import 'support.dart';

/// Các test này đọc workflow THẬT của repository. Chúng chứng minh workflow
/// có các tính chất bài 33 nói tới; chúng không chứng minh workflow chạy
/// được trên GitHub (xem README: nhánh chưa push).
void main() {
  final audit = loadRealWorkflow();

  group('đọc được file', () {
    test('tìm ra gốc repository và workflow nằm đúng chỗ', () {
      expect(
        File('${repoRoot.path}/$workflowRelativePath').existsSync(),
        isTrue,
      );
    });

    test('khoá on là chuỗi "on", không bị đọc thành true như YAML 1.1', () {
      expect(audit.document.containsKey('on'), isTrue);
      expect(audit.document.containsKey(true), isFalse);
      expect(audit.triggers.keys, containsAll(['push', 'pull_request']));
    });

    test('job verify là job cuối, golden-macos đứng trước nó', () {
      expect(audit.jobNames.last, 'verify');
      expect(
        audit.jobNames.indexOf('golden-macos'),
        lessThan(audit.jobNames.indexOf('verify')),
      );
    });
  });

  group('phiên bản và ref', () {
    test(
      'mọi flutter-action ghim đúng $expectedFlutterVersion, channel stable',
      () {
        expect(audit.checkFlutterPin(), isEmpty);
        // Cả hai job đều cài Flutter.
        expect(audit.usesOf('subosito/flutter-action'), hasLength(2));
      },
    );

    test('mọi action ghim tag đủ ba số, không có @main hay @master', () {
      expect(audit.checkActionRefs(), isEmpty);
      expect(audit.text, isNot(contains('@main')));
      expect(audit.text, isNot(contains('@master')));
    });

    test('tag v2 trôi của flutter-action (bản cũ của file) sẽ bị bắt', () {
      final old = mutated(
        'subosito/flutter-action@v2.23.0',
        'subosito/flutter-action@v2',
      );
      expect(old.checkActionRefs().map((f) => f.rule), contains('action-ref'));
    });
  });

  group('quyền và bí mật', () {
    test('có permissions ở gốc, chỉ contents: read', () {
      expect(audit.checkPermissions(), isEmpty);
      final permissions = audit.document['permissions'] as YamlMap;
      expect(permissions['contents'], 'read');
      expect(permissions.length, 1);
    });

    test('không có chuỗi giống bí mật', () {
      expect(audit.checkNoSecrets(), isEmpty);
    });

    test('workflow không dùng secrets nào (lab không cần tài khoản)', () {
      expect(audit.text, isNot(contains('secrets.')));
    });
  });

  group('trỏ tới file có thật', () {
    test('working-directory và script node đều tồn tại', () {
      expect(audit.checkReferencedFiles(), isEmpty);
    });

    test('mọi script verify-*.mjs trong lệnh node có mặt trên đĩa', () {
      final scripts = [
        for (final (_, step) in audit.steps)
          for (final m in RegExp(
            r'node\s+(\S+\.mjs)',
          ).allMatches('${step['run'] ?? ''}'))
            m.group(1)!,
      ];
      expect(scripts, isNotEmpty);
      for (final s in scripts) {
        expect(File('${repoRoot.path}/$s').existsSync(), isTrue, reason: s);
      }
    });
  });

  group('trigger theo path', () {
    test('push và pull_request bắt workflow và mọi project dưới examples', () {
      final dirs = projectDirs();
      expect(dirs, contains('examples/toi-noi-flutter/33-ci-github-actions'));
      expect(audit.checkPathsTrigger(dirs), isEmpty);
    });

    test('bài viết và README ngoài thư mục lab không kích hoạt workflow', () {
      for (final event in ['push', 'pull_request']) {
        final globs = [
          for (final p in audit.triggers[event]['paths'] as YamlList) '$p',
        ];
        expect(
          pathsFilterMatches(
            globs,
            'api/seed/content/flutter-ci-github-actions-analyze-test-golden-cache.md',
          ),
          isFalse,
        );
        expect(
          pathsFilterMatches(globs, 'docs/flutter-master-curriculum.md'),
          isFalse,
        );
      }
    });
  });

  group('job mà bài 33 mô tả', () {
    const lab29 = 'examples/toi-noi-flutter/29-golden-semantics';

    test('golden chạy trên runner macOS, đúng nơi ảnh chuẩn được sinh', () {
      final job = audit.jobs['golden-macos'] as YamlMap;
      expect('${job['runs-on']}', startsWith('macos-'));
      final harness = readRepoFile('$lab29/test/support/golden_harness.dart');
      expect(harness, contains("const goldenBaselineOs = 'macos';"));
    });

    test('lệnh golden chạy trong lab 29 với tag golden đã khai báo', () {
      final step = audit.steps
          .map((e) => e.$2)
          .singleWhere((s) => '${s['run']}'.contains('--tags golden'));
      expect(step['working-directory'], lab29);
      expect(readRepoFile('$lab29/dart_test.yaml'), contains('golden:'));
    });

    test(
      'khi golden đỏ, artifact là test/failures và lab 29 không commit nó',
      () {
        final upload = audit.steps
            .map((e) => e.$2)
            .singleWhere((s) => '${s['name']}' == 'Upload golden failures');
        expect(upload['if'], 'failure()');
        final path = (upload['with'] as YamlMap)['path'] as String;
        expect(path, '$lab29/test/failures/');
        expect(readRepoFile('$lab29/.gitignore'), contains('test/failures/'));
      },
    );

    test('cổng analyzer của lab 32 dùng dart analyze --fatal-infos', () {
      final step = audit.steps
          .map((e) => e.$2)
          .singleWhere((s) => '${s['run']}'.contains('dart analyze'));
      expect('${step['run']}', contains('dart analyze --fatal-infos'));
      expect(
        step['working-directory'],
        'examples/toi-noi-flutter/32-lint-analyzer',
      );
      // Lý do dùng dart analyze: lab 32 khai báo analyzer plugin.
      expect(
        readRepoFile(
          'examples/toi-noi-flutter/32-lint-analyzer/analysis_options.yaml',
        ),
        contains('plugins:'),
      );
    });

    test('coverage: sinh lcov.info, upload artifact, không có ngưỡng cứng', () {
      final steps = audit.steps.map((e) => e.$2).toList();
      final run = steps.singleWhere(
        (s) => '${s['run']}'.contains('--coverage'),
      );
      expect(
        run['working-directory'],
        'examples/toi-noi-flutter/28-widget-test',
      );
      final upload = steps.singleWhere(
        (s) => '${s['name']}'.startsWith('Upload coverage'),
      );
      expect(
        (upload['with'] as YamlMap)['path'],
        'examples/toi-noi-flutter/28-widget-test/coverage/lcov.info',
      );
      expect(
        audit.text,
        isNot(
          matches(
            RegExp(
              r'min-coverage|fail-under|fail_under|threshold',
              caseSensitive: false,
            ),
          ),
        ),
      );
    });
  });

  group('cache, timeout và concurrency', () {
    test('mọi job có timeout, workflow huỷ run cũ cùng nhánh', () {
      expect(audit.checkHygiene(), isEmpty);
      final concurrency = audit.document['concurrency'] as YamlMap;
      expect('${concurrency['group']}', contains('github.ref'));
    });

    test('push chỉ chạy cho main, nhánh PR chạy qua pull_request', () {
      final push = audit.triggers['push'] as YamlMap;
      expect(push['branches'], ['main']);
    });

    test('main: mỗi commit một group và không bị huỷ; nhánh khác thì huỷ', () {
      final concurrency = audit.document['concurrency'] as YamlMap;
      expect('${concurrency['group']}', contains('github.sha'));
      expect(
        '${concurrency['cancel-in-progress']}',
        contains("!= 'refs/heads/main'"),
      );
    });

    test('verify có timeout đủ rộng cho nhiều lab (>= 90 phút)', () {
      final verify = audit.jobs['verify'] as YamlMap;
      expect(verify['timeout-minutes'] as int, greaterThanOrEqualTo(90));
    });

    test(
      'khoá cache pub băm pubspec.yaml vì lab không commit pubspec.lock',
      () {
        expect(audit.checkPubCacheKey(), isEmpty);
        for (final lab in ['28-widget-test', '29-golden-semantics']) {
          expect(
            readRepoFile('examples/toi-noi-flutter/$lab/.gitignore'),
            contains('pubspec.lock'),
            reason: lab,
          );
        }
      },
    );
  });

  test('toàn bộ kiểm tra gộp không có phát hiện nào', () {
    expect(audit.all(labDirs: projectDirs()), isEmpty);
  });
}
