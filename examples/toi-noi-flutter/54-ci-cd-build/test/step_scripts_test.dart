import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';
import 'package:toi_noi_54/build_number.dart';
import 'package:toi_noi_54/release_audit.dart';
import 'package:yaml/yaml.dart';

import 'support.dart';

/// Chạy chính các script `run` của workflow mẫu bằng bash, trong thư mục tạm,
/// với giá trị secret GIẢ và lệnh `security` GIẢ. Không đụng keychain thật,
/// không mạng. Chứng minh hành vi của script, không chứng minh GitHub chạy nó.
void main() {
  final wf = loadSample();
  final skip = hasBash ? null : 'máy không có bash';

  late Directory sandbox;
  setUp(() => sandbox = makeSandbox());
  tearDown(() => sandbox.deleteSync(recursive: true));

  group('Android: khôi phục keystore', () {
    late YamlMap restore;
    late Map<String, String> secrets;
    late Directory app;

    setUp(() {
      restore = stepNamed(wf, 'android', 'Restore upload keystore');
      secrets = fakeSecretsFor(restore);
      app = Directory('${sandbox.path}/app');
    });

    Map<String, String> env() => {...baseEnv(sandbox), ...secrets};

    test(
      'giải mã đúng keystore, ghi key.properties, không in secret ra log',
      () {
        final result = runScript(
          ReleaseWorkflow.runOf(restore),
          env: env(),
          workingDirectory: app,
        );
        expect(result.exitCode, 0, reason: result.log);

        for (final value in secrets.values) {
          expect(result.log, isNot(contains(value)), reason: 'lộ $value');
        }

        final keystore = File(
          '${sandbox.path}/runner_temp/upload-keystore.jks',
        );
        expect(
          keystore.readAsBytesSync(),
          base64.decode(secrets['ANDROID_KEYSTORE_BASE64']!),
        );

        final props = File('${app.path}/android/key.properties');
        final lines = props.readAsLinesSync();
        expect(lines, [
          'storeFile=${keystore.path}',
          'storePassword=${secrets['ANDROID_KEYSTORE_PASSWORD']}',
          'keyAlias=${secrets['ANDROID_KEY_ALIAS']}',
          'keyPassword=${secrets['ANDROID_KEY_PASSWORD']}',
        ]);
      },
      skip: skip,
    );

    test('file chỉ chủ sở hữu đọc được (umask 077)', () {
      final result = runScript(
        ReleaseWorkflow.runOf(restore),
        env: env(),
        workingDirectory: app,
      );
      expect(result.exitCode, 0, reason: result.log);
      for (final path in [
        '${sandbox.path}/runner_temp/upload-keystore.jks',
        '${app.path}/android/key.properties',
      ]) {
        final mode = File(path).statSync().mode & 0x1FF;
        expect(mode.toRadixString(8), '600', reason: path);
      }
    }, skip: skip);

    test('mật khẩu có ký tự đặc biệt không bị shell diễn giải', () {
      secrets['ANDROID_KEYSTORE_PASSWORD'] =
          r'p$a"s s`x`&;|*'
          "'";
      final result = runScript(
        ReleaseWorkflow.runOf(restore),
        env: env(),
        workingDirectory: app,
      );
      expect(result.exitCode, 0, reason: result.log);
      final props = File('${app.path}/android/key.properties')
          .readAsStringSync();
      expect(
        props,
        contains('storePassword=${secrets['ANDROID_KEYSTORE_PASSWORD']}\n'),
      );
      expect(
        result.log,
        isNot(contains(secrets['ANDROID_KEYSTORE_PASSWORD']!)),
      );
    }, skip: skip);

    test('thiếu một secret thì dừng chứ không ghi file thiếu', () {
      final environment = env()..remove('ANDROID_KEY_PASSWORD');
      final result = runScript(
        ReleaseWorkflow.runOf(restore),
        env: environment,
        workingDirectory: app,
      );
      expect(result.exitCode, isNot(0));
      expect(result.stderr, contains('ANDROID_KEY_PASSWORD'));
    }, skip: skip);

    test('secret rỗng hay chưa đặt thì dừng, không sinh keystore 0 byte', () {
      for (final name in secrets.keys) {
        for (final empty in [true, false]) {
          final environment = env();
          if (empty) {
            environment[name] = '';
          } else {
            environment.remove(name);
          }
          final result = runScript(
            ReleaseWorkflow.runOf(restore),
            env: environment,
            workingDirectory: app,
          );
          expect(result.exitCode, isNot(0), reason: '$name empty=$empty');
          expect(result.stderr, contains(name));
          expect(
            File('${sandbox.path}/runner_temp/upload-keystore.jks')
                .existsSync(),
            isFalse,
            reason: '$name empty=$empty',
          );
        }
      }
    }, skip: skip);

    test('bước xoá dọn sạch cả hai file, kể cả khi chưa có file nào', () {
      final cleanup = stepNamed(
        wf,
        'android',
        'Remove keystore and key.properties',
      );
      final restoreResult = runScript(
        ReleaseWorkflow.runOf(restore),
        env: env(),
        workingDirectory: app,
      );
      expect(restoreResult.exitCode, 0, reason: restoreResult.log);
      for (var i = 0; i < 2; i++) {
        final result = runScript(
          ReleaseWorkflow.runOf(cleanup),
          env: baseEnv(sandbox),
          workingDirectory: app,
        );
        expect(result.exitCode, 0, reason: 'lần $i: ${result.log}');
      }
      expect(
        File('${sandbox.path}/runner_temp/upload-keystore.jks').existsSync(),
        isFalse,
      );
      expect(File('${app.path}/android/key.properties').existsSync(), isFalse);
    }, skip: skip);

    test(
      'ca sai: bản có echo mật khẩu THẬT SỰ in ra log (test phát hiện được)',
      () {
        final script = ReleaseWorkflow.runOf(restore).replaceFirst(
          'printf \'storePassword=%s\\n\' "\$ANDROID_KEYSTORE_PASSWORD" >> "\$PROPS"',
          'echo "storePassword=\$ANDROID_KEYSTORE_PASSWORD"',
        );
        expect(script, isNot(ReleaseWorkflow.runOf(restore)));
        final result = runScript(script, env: env(), workingDirectory: app);
        expect(result.stdout, contains(secrets['ANDROID_KEYSTORE_PASSWORD']));
      },
      skip: skip,
    );
  });

  group('iOS: keychain tạm (lệnh security giả)', () {
    late YamlMap install;
    late YamlMap cleanup;
    late Map<String, String> secrets;
    late Directory app;
    late File log;

    setUp(() {
      install = stepNamed(
        wf,
        'ios',
        'Install certificate and provisioning profile',
      );
      cleanup = stepNamed(
        wf,
        'ios',
        'Clean up keychain and provisioning profile',
      );
      secrets = fakeSecretsFor(install);
      app = Directory('${sandbox.path}/app');
      installSecurityStub(sandbox);
      log = File('${sandbox.path}/security.log');
    });

    Map<String, String> env({String? fail}) => {
      ...baseEnv(sandbox, extraPath: '${sandbox.path}/bin'),
      'STUB_LOG': log.path,
      'STUB_FAIL': ?fail,
    };

    RunResult runInstall({String? fail}) => runScript(
      ReleaseWorkflow.runOf(install),
      env: {
        ...env(fail: fail),
        ...secrets,
      },
      workingDirectory: app,
    );

    RunResult runCleanup() => runScript(
      ReleaseWorkflow.runOf(cleanup),
      env: env(),
      workingDirectory: app,
    );

    final keychain = 'runner_temp/toi-noi-signing.keychain-db';
    // Xcode cũ đọc MobileDevice, Xcode mới (và Flutter 3.47.4) đọc UserData.
    final profiles = [
      'home/Library/MobileDevice/Provisioning Profiles/toi-noi-staging.mobileprovision',
      'home/Library/Developer/Xcode/UserData/Provisioning Profiles/toi-noi-staging.mobileprovision',
    ];

    test('cài đủ sáu lệnh security theo thứ tự, không in secret ra log', () {
      final result = runInstall();
      expect(result.exitCode, 0, reason: result.log);
      for (final value in secrets.values) {
        expect(result.log, isNot(contains(value)), reason: 'lộ $value');
      }
      final subs = [for (final l in log.readAsLinesSync()) l.split(' ').first];
      expect(subs, [
        'create-keychain',
        'set-keychain-settings',
        'unlock-keychain',
        'import',
        'set-key-partition-list',
        'list-keychain',
      ]);
    }, skip: skip);

    test('profile được giải mã đúng chỗ (đường dẫn có dấu cách), p12 bị xoá sau import', () {
      expect(runInstall().exitCode, 0);
      for (final profile in profiles) {
        expect(
          File('${sandbox.path}/$profile').readAsBytesSync(),
          base64.decode(secrets['IOS_PROVISIONING_PROFILE_BASE64']!),
          reason: profile,
        );
      }
      expect(
        File('${sandbox.path}/runner_temp/distribution.p12').existsSync(),
        isFalse,
      );
      final importLine = log.readAsLinesSync().singleWhere(
        (l) => l.startsWith('import '),
      );
      expect(importLine, contains('distribution.p12'));
      expect(importLine, contains('-k ${sandbox.path}/$keychain'));
    }, skip: skip);

    test('mật khẩu keychain sinh ngẫu nhiên mỗi lần, không lấy từ secret', () {
      expect(runInstall().exitCode, 0);
      String passwordOf(File f) =>
          RegExp(r'create-keychain -p (\S+) ')
              .firstMatch(f.readAsStringSync())!
              .group(1)!;
      final first = passwordOf(log);
      log.deleteSync();
      expect(runInstall().exitCode, 0);
      final second = passwordOf(log);
      expect(first, hasLength(32));
      expect(second, isNot(first));
      expect(secrets.values, isNot(contains(first)));
    }, skip: skip);

    test('secret rỗng thì dừng trước khi tạo keychain hay chép profile', () {
      for (final name in secrets.keys) {
        secrets[name] = '';
        final result = runInstall();
        expect(result.exitCode, isNot(0), reason: name);
        expect(result.stderr, contains(name));
        expect(log.existsSync(), isFalse, reason: name);
        for (final profile in profiles) {
          expect(File('${sandbox.path}/$profile').existsSync(), isFalse);
        }
        secrets.addAll(fakeSecretsFor(install));
      }
    }, skip: skip);

    test('dọn: xoá đúng keychain đã tạo, profile và p12', () {
      expect(runInstall().exitCode, 0);
      expect(File('${sandbox.path}/$keychain').existsSync(), isTrue);
      final result = runCleanup();
      expect(result.exitCode, 0, reason: result.log);
      expect(
        log.readAsLinesSync().last,
        'delete-keychain ${sandbox.path}/$keychain',
      );
      expect(File('${sandbox.path}/$keychain').existsSync(), isFalse);
      for (final profile in profiles) {
        expect(File('${sandbox.path}/$profile').existsSync(), isFalse);
      }
    }, skip: skip);

    test('import lỗi giữa chừng: p12 và keychain ở lại, bước dọn gỡ sạch', () {
      final failed = runInstall(fail: 'import');
      expect(failed.exitCode, isNot(0));
      // Đây là lý do bước dọn cần if: always(): script cài đặt đã dừng.
      expect(
        File('${sandbox.path}/runner_temp/distribution.p12').existsSync(),
        isTrue,
      );
      expect(File('${sandbox.path}/$keychain').existsSync(), isTrue);
      for (final profile in profiles) {
        expect(File('${sandbox.path}/$profile').existsSync(), isTrue);
      }

      expect(runCleanup().exitCode, 0);
      expect(
        File('${sandbox.path}/runner_temp/distribution.p12').existsSync(),
        isFalse,
      );
      expect(File('${sandbox.path}/$keychain').existsSync(), isFalse);
      for (final profile in profiles) {
        expect(File('${sandbox.path}/$profile').existsSync(), isFalse);
      }
    }, skip: skip);

    test(
      'dọn khi cài đặt chưa chạy (job hỏng sớm): thoát 0 và không gọi security',
      () {
        final result = runCleanup();
        expect(result.exitCode, 0, reason: result.log);
        expect(log.existsSync(), isFalse);
      },
      skip: skip,
    );

    test('dọn chạy hai lần vẫn thoát 0 (idempotent)', () {
      expect(runInstall().exitCode, 0);
      expect(runCleanup().exitCode, 0);
      expect(runCleanup().exitCode, 0);
    }, skip: skip);
  });

  group('Build number: chạy script của bước Compute build number', () {
    final step = stepNamed(wf, 'prepare', 'Compute build number');
    final script = ReleaseWorkflow.runOf(step);

    /// Chạy script, trả về (exit, build_number hoặc null).
    (int, String?, RunResult) compute(String run, String offset) {
      final output = File('${sandbox.path}/output');
      final summary = File('${sandbox.path}/summary');
      // Mỗi step của GitHub có file output riêng, trống lúc bắt đầu.
      for (final f in [output, summary]) {
        if (f.existsSync()) f.deleteSync();
      }
      final result = runScript(
        script,
        env: {
          ...baseEnv(sandbox),
          'RUN_NUMBER': run,
          'OFFSET': offset,
          'GITHUB_OUTPUT': output.path,
          'GITHUB_STEP_SUMMARY': summary.path,
        },
        workingDirectory: Directory('${sandbox.path}/app'),
      );
      String? number;
      if (output.existsSync()) {
        final m = RegExp(
          r'^build_number=(\d+)$',
          multiLine: true,
        ).firstMatch(output.readAsStringSync());
        number = m?.group(1);
      }
      return (result.exitCode, number, result);
    }

    test('run_number tăng ngặt thì build number tăng ngặt, khớp hàm Dart', () {
      final numbers = <int>[];
      for (final run in [1, 2, 3, 9, 10, 11, 99, 100, 1000, 54321]) {
        final (code, number, result) = compute('$run', '0');
        expect(code, 0, reason: result.log);
        expect(int.parse(number!), computeBuildNumber(runNumber: run));
        numbers.add(int.parse(number));
      }
      expect(isStrictlyIncreasing(numbers), isTrue);
    }, skip: skip);

    test('chạy lại cùng run_number cho đúng cùng số (re-run không đổi số)', () {
      final a = compute('42', '0').$2;
      final b = compute('42', '0').$2;
      expect(a, '42');
      expect(b, a);
    }, skip: skip);

    test('offset cộng thêm: store đã nhận build 500 thì run 1 ra 501', () {
      expect(compute('1', '500').$2, '501');
      expect(compute('2', '500').$2, '502');
    }, skip: skip);

    test('số có chữ số 0 đứng đầu là thập phân, không phải bát phân', () {
      expect(compute('008', '009').$2, '17');
    }, skip: skip);

    test('ghi tóm tắt vào GITHUB_STEP_SUMMARY', () {
      compute('7', '0');
      expect(
        File('${sandbox.path}/summary').readAsStringSync(),
        contains('Build number: 7'),
      );
    }, skip: skip);

    test('đầu vào hỏng làm bước đỏ và không xuất build number', () {
      for (final (run, offset) in [
        ('', '0'),
        ('1', ''),
        ('abc', '0'),
        ('-1', '0'),
        ('1.5', '0'),
        ('1 2', '0'),
        ('1', 'x'),
        ('99999999999', '0'),
        ('0', '0'),
      ]) {
        final (code, number, result) = compute(run, offset);
        expect(
          code,
          isNot(0),
          reason: 'run="$run" offset="$offset": ${result.log}',
        );
        expect(number, isNull, reason: 'run="$run" offset="$offset"');
      }
    }, skip: skip);

    test(
      'trần versionCode của Google Play: 2100000000 qua, 2100000001 không',
      () {
        expect(compute('2100000000', '0').$2, '2100000000');
        expect(compute('2100000001', '0').$1, isNot(0));
        expect(compute('2000000000', '100000000').$2, '2100000000');
        expect(compute('2000000000', '100000001').$1, isNot(0));
      },
      skip: skip,
    );
  });
}
