import 'dart:io';

import 'package:test/test.dart';
import 'package:toi_noi_33/workflow_audit.dart' show pathsFilterMatches;
import 'package:toi_noi_54/release_audit.dart';
import 'package:yaml/yaml.dart';

import 'support.dart';

/// Các test này đọc workflow MẪU. Chúng chứng minh file có các tính chất bài 54
/// nói tới; chúng không chứng minh workflow chạy được trên GitHub (chưa chạy).
void main() {
  final wf = loadSample();

  group('đọc được file', () {
    test('mẫu nằm trong native-config, không nằm ở .github/ thật', () {
      expect(File('${labDir.path}/$sampleWorkflowPath').existsSync(), isTrue);
      expect(Directory('${labDir.path}/.github').existsSync(), isFalse);
      final real = Directory('${repoRoot.path}/.github/workflows')
          .listSync()
          .map((e) => e.path.split('/').last);
      expect(real, isNot(contains('release-staging.yml')));
    });

    test('ba job theo thứ tự prepare, android, ios', () {
      expect(wf.base.jobNames, ['prepare', 'android', 'ios']);
    });

    test('toàn bộ kiểm tra gộp không có phát hiện nào', () {
      expect(wf.all(), isEmpty);
    });
  });

  group('phần dùng lại từ lab 33', () {
    test('Flutter ghim 3.47.4 ở cả hai job build, channel stable', () {
      expect(wf.base.checkFlutterPin(), isEmpty);
      expect(wf.base.usesOf('subosito/flutter-action'), hasLength(2));
    });

    test('mọi action ghim tag đủ ba số (kể cả setup-java v6.2.0)', () {
      expect(wf.base.checkActionRefs(), isEmpty);
      final uses = [
        for (final (_, s) in wf.base.steps)
          if (s['uses'] != null) '${s['uses']}',
      ];
      expect(
        uses.toSet(),
        containsAll([
          'actions/checkout@v7.0.1',
          'actions/setup-java@v6.2.0',
          'subosito/flutter-action@v2.23.0',
          'actions/upload-artifact@v7.0.2',
        ]),
      );
    });

    test('đúng các phiên bản mà workflow thật của lab 33 đã ghim', () {
      final real = File('${repoRoot.path}/$workflowRelativePath')
          .readAsStringSync();
      for (final ref in [
        'actions/checkout@v7.0.1',
        'subosito/flutter-action@v2.23.0',
        'actions/upload-artifact@v7.0.2',
      ]) {
        expect(real, contains(ref));
        expect(wf.text, contains(ref));
      }
    });

    test('không có chuỗi giống bí mật trong file', () {
      expect(wf.base.checkNoSecrets(), isEmpty);
    });
  });

  group('trigger và quyền', () {
    test('chỉ workflow_dispatch và push theo tag', () {
      expect(wf.checkTriggers(), isEmpty);
      expect(wf.triggers.keys, unorderedEquals(['workflow_dispatch', 'push']));
      expect((wf.triggers['push'] as YamlMap)['tags'], ['staging-v*']);
    });

    test('mẫu tag staging-v* bắt tag phát hành, không bắt tag khác', () {
      final globs = [
        for (final t in (wf.triggers['push'] as YamlMap)['tags'] as YamlList)
          '$t',
      ];
      expect(pathsFilterMatches(globs, 'staging-v0.1.0'), isTrue);
      expect(pathsFilterMatches(globs, 'staging-v0.1.0-rc.1'), isTrue);
      expect(pathsFilterMatches(globs, 'v0.1.0'), isFalse);
      expect(pathsFilterMatches(globs, 'prod-v0.1.0'), isFalse);
      // * không vượt qua dấu /, nên tag kiểu thư mục không khớp.
      expect(pathsFilterMatches(globs, 'staging-v1/x'), isFalse);
    });

    test('quyền tối thiểu: chỉ contents: read, không OIDC', () {
      expect(wf.checkLeastPrivilege(), isEmpty);
      final permissions = wf.document['permissions'] as YamlMap;
      expect(permissions.length, 1);
      expect(permissions['contents'], 'read');
      expect(wf.text, isNot(contains('id-token')));
    });

    test('hai job build gắn environment staging; prepare không có secret', () {
      expect(wf.job('android')['environment'], 'staging');
      expect(wf.job('ios')['environment'], 'staging');
      expect(wf.job('prepare')['environment'], isNull);
      expect(wf.checkSecretScope(), isEmpty);
    });

    test('mọi secret chỉ nằm trong env của bước run, tên đúng như README', () {
      final names = RegExp(r'secrets\.([A-Z0-9_]+)')
          .allMatches(wf.text)
          .map((m) => m.group(1))
          .toSet();
      expect(names, {
        'ANDROID_KEYSTORE_BASE64',
        'ANDROID_KEYSTORE_PASSWORD',
        'ANDROID_KEY_ALIAS',
        'ANDROID_KEY_PASSWORD',
        'IOS_CERTIFICATE_P12_BASE64',
        'IOS_CERTIFICATE_PASSWORD',
        'IOS_PROVISIONING_PROFILE_BASE64',
      });
      final readme = File('${labDir.path}/README.md').readAsStringSync();
      for (final n in names) {
        expect(readme, contains(n!), reason: 'README phải liệt kê $n');
      }
    });
  });

  group('secret không lộ ra log (kiểm tĩnh)', () {
    test('không nội suy \${{ }} trong run', () {
      expect(wf.checkNoExpressionInRun(), isEmpty);
    });

    test('không echo, không xtrace, không in môi trường', () {
      expect(wf.checkSecretLeaks(), isEmpty);
    });

    test(
      'mỗi biến secret chỉ được dùng qua pipe hoặc chuyển hướng ra file',
      () {
        // Mọi dòng printf/echo nhắc biến secret đều có | hoặc >.
        for (final job in ['android', 'ios']) {
          for (final step in wf.stepsOf(job)) {
            final secrets = wf.secretVarsOf(job, step);
            for (final line in logicalLines(ReleaseWorkflow.runOf(step))) {
              final isPrinter = RegExp(r'\b(echo|printf)\b').hasMatch(line);
              for (final s in secrets) {
                if (isPrinter && line.contains('\$$s')) {
                  expect(
                    line,
                    anyOf(contains('|'), contains('>')),
                    reason: line,
                  );
                }
              }
            }
          }
        }
      },
    );
  });

  group('dọn keychain và keystore', () {
    test(
      'job iOS có bước dọn keychain: if always(), ngay sau build, đúng file',
      () {
        expect(wf.checkKeychainCleanup(), isEmpty);
        final steps = wf.stepsOf('ios');
        final build = steps.indexWhere(
          (s) => ReleaseWorkflow.runOf(s).contains('flutter build ipa'),
        );
        final cleanup = steps[build + 1];
        expect(cleanup['name'], 'Clean up keychain and provisioning profile');
        expect(cleanup['if'], 'always()');
        expect(
          ReleaseWorkflow.runOf(cleanup),
          contains('security delete-keychain'),
        );
      },
    );

    test('bước dọn đứng trước mọi bước upload: file ký không còn trên đĩa', () {
      for (final (job, build) in [
        ('ios', 'flutter build ipa'),
        ('android', 'flutter build appbundle'),
      ]) {
        final steps = wf.stepsOf(job);
        final buildAt = steps.indexWhere(
          (s) => ReleaseWorkflow.runOf(s).contains(build),
        );
        final cleanupAt = steps.indexWhere(
          (s) =>
              s['if'] == 'always()' && ReleaseWorkflow.runOf(s).contains('rm '),
        );
        final firstUpload = steps.indexWhere(
          (s) => '${s['uses']}'.startsWith('actions/upload-artifact@'),
        );
        expect(cleanupAt, buildAt + 1, reason: job);
        expect(cleanupAt, lessThan(firstUpload), reason: job);
      }
    });

    test('mỗi secret được chặn rỗng trước khi dùng', () {
      expect(wf.checkSecretGuards(), isEmpty);
    });

    test('profile được chép vào cả hai thư mục Xcode có thể đọc', () {
      final install = ReleaseWorkflow.runOf(
        stepNamed(wf, 'ios', 'Install certificate and provisioning profile'),
      );
      expect(install, contains('Library/MobileDevice/Provisioning Profiles'));
      expect(
        install,
        contains('Library/Developer/Xcode/UserData/Provisioning Profiles'),
      );
    });

    test('job Android xoá keystore và key.properties ở bước always()', () {
      expect(wf.checkKeystoreCleanup(), isEmpty);
    });
  });

  group('build number', () {
    test('một nguồn từ run_number, cả hai nền tảng dùng chung', () {
      expect(wf.checkBuildNumber(), isEmpty);
      final env = wf.job('android')['env'] as YamlMap;
      expect(env['BUILD_NUMBER'], r'${{ needs.prepare.outputs.build_number }}');
      final iosEnv = wf.job('ios')['env'] as YamlMap;
      expect(iosEnv['BUILD_NUMBER'], env['BUILD_NUMBER']);
    });

    test('offset lấy từ biến repository, mặc định 0', () {
      final step = stepNamed(wf, 'prepare', 'Compute build number');
      final env = step['env'] as YamlMap;
      expect(env['OFFSET'], r"${{ vars.BUILD_NUMBER_OFFSET || '0' }}");
    });

    test('không dùng run_attempt, run_id, SHA hay giờ hệ thống', () {
      for (final banned in ['run_attempt', 'run_id', 'github.sha', 'date +']) {
        expect(wf.text, isNot(contains(banned)));
      }
    });
  });

  group('lệnh build và artifact', () {
    test(
      'Android: ubuntu, temurin 17 trước build, appbundle flavor staging',
      () {
        expect(wf.checkBuildCommands(), isEmpty);
        expect('${wf.job('android')['runs-on']}', 'ubuntu-24.04');
        expect('${wf.job('ios')['runs-on']}', 'macos-26');
      },
    );

    test('cấu hình flavor và thư mục app có thật trong toi-noi-app', () {
      expect(wf.checkAppPaths(), isEmpty);
    });

    test('artifact AAB, IPA và symbols, tên không trùng, giữ 14 ngày', () {
      expect(wf.checkArtifacts(), isEmpty);
      final names = [
        for (final job in ['android', 'ios'])
          for (final s in wf.stepsOf(job))
            if ('${s['uses']}'.startsWith('actions/upload-artifact@'))
              '${(s['with'] as YamlMap)['name']}',
      ];
      expect(names, hasLength(4));
    });

    test(
      'mọi job có timeout, checkout không giữ token, không huỷ run đang build',
      () {
        expect(wf.checkHygiene(), isEmpty);
      },
    );

    test(
      'version trong pubspec của app là x.y.z+build; workflow không hard-code',
      () {
        final pubspec = File('${repoRoot.path}/$customerAppPath/pubspec.yaml')
            .readAsStringSync();
        expect(
          RegExp(
            r'^version: \d+\.\d+\.\d+\+\d+$',
            multiLine: true,
          ).hasMatch(pubspec),
          isTrue,
        );
        expect(wf.text, isNot(contains('--build-name')));
      },
    );
  });
}
