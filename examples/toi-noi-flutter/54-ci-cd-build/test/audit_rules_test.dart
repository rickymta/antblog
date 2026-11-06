import 'package:test/test.dart';
import 'package:toi_noi_54/release_audit.dart';

import 'support.dart';

/// Mỗi kiểm tra phải đỏ khi workflow mẫu sai. Ca sai là workflow mẫu bị sửa
/// đúng một chỗ; nếu mẫu đổi hình thù, [mutated] báo lỗi chứ không im lặng.
void main() {
  List<String> rules(List<Finding> findings) => [
    for (final f in findings) f.rule,
  ];

  // Dòng ghi keyAlias của bước khôi phục keystore: chỗ chèn các ca sai.
  const aliasLine =
      '          printf \'keyAlias=%s\\n\' "\$ANDROID_KEY_ALIAS" >> "\$PROPS"\n';
  const passwordLine =
      '          printf \'storePassword=%s\\n\' "\$ANDROID_KEYSTORE_PASSWORD" >> "\$PROPS"\n';
  const cleanupName = 'Clean up keychain and provisioning profile';

  group('trigger', () {
    test('thêm pull_request bị bắt: secret không nên mở cho PR', () {
      final wf = mutated(
        'on:\n  workflow_dispatch:\n',
        'on:\n  pull_request:\n  workflow_dispatch:\n',
      );
      expect(rules(wf.checkTriggers()), ['trigger']);
    });

    test('push theo nhánh bị bắt', () {
      final wf = mutated(
        "    tags: ['staging-v*']\n",
        "    tags: ['staging-v*']\n    branches: [main]\n",
      );
      expect(rules(wf.checkTriggers()), ['trigger']);
    });

    test('push không lọc tag bị bắt', () {
      final wf = mutated("    tags: ['staging-v*']\n", '    paths: [x]\n');
      expect(rules(wf.checkTriggers()), contains('trigger'));
    });
  });

  group('quyền', () {
    test('contents: write bị bắt', () {
      final wf = mutated('contents: read', 'contents: write');
      expect(wf.checkLeastPrivilege(), isNotEmpty);
    });

    test('thêm id-token: write bị bắt dù workflow không dùng OIDC', () {
      final wf = mutated(
        'permissions:\n  contents: read\n',
        'permissions:\n  contents: read\n  id-token: write\n',
      );
      expect(wf.checkLeastPrivilege(), isNotEmpty);
    });

    test('job tự khai permissions bị bắt', () {
      final wf = mutated(
        '    timeout-minutes: 30\n',
        '    timeout-minutes: 30\n    permissions:\n      contents: read\n',
      );
      expect(wf.checkLeastPrivilege().single.message, contains('android'));
    });
  });

  group('phạm vi secret', () {
    test(
      'secret ở env cấp job bị bắt: mọi step, kể cả action ngoài, thấy nó',
      () {
        final wf = mutated(
          '    timeout-minutes: 30\n    environment: staging\n    env:\n',
          '    timeout-minutes: 30\n    environment: staging\n    env:\n      LEAK: \${{ secrets.ANDROID_KEY_PASSWORD }}\n',
        );
        expect(rules(wf.checkSecretScope()), contains('secret-scope'));
      },
    );

    test('secret đưa vào with của action bị bắt', () {
      final wf = mutated(
        "          java-version: '17'\n",
        "          java-version: '17'\n          token: \${{ secrets.ANDROID_KEY_PASSWORD }}\n",
      );
      expect(rules(wf.checkSecretScope()), contains('secret-scope'));
    });

    test('secret ở env cấp workflow bị bắt', () {
      final wf = mutated(
        'concurrency:',
        'env:\n  LEAK: \${{ secrets.ANDROID_KEY_PASSWORD }}\nconcurrency:',
      );
      expect(rules(wf.checkSecretScope()), contains('secret-scope'));
    });

    test('job dùng secret mà bỏ environment bị bắt', () {
      final wf = mutated(
        '    timeout-minutes: 30\n    environment: staging\n',
        '    timeout-minutes: 30\n',
      );
      expect(rules(wf.checkSecretScope()), ['environment']);
    });
  });

  group('log không lộ secret', () {
    test('nội suy secret thẳng vào run bị bắt', () {
      final wf = mutated(
        'printf \'%s\' "\$ANDROID_KEYSTORE_BASE64"',
        'printf \'%s\' "\${{ secrets.ANDROID_KEYSTORE_BASE64 }}"',
      );
      expect(rules(wf.checkNoExpressionInRun()), ['expression-in-run']);
    });

    test('echo mật khẩu ra stdout bị bắt', () {
      final wf = mutated(
        passwordLine,
        '          echo "storePassword=\$ANDROID_KEYSTORE_PASSWORD"\n',
      );
      expect(rules(wf.checkSecretLeaks()), ['secret-echo']);
    });

    test('stderr (>&2) vẫn là log nên bị bắt', () {
      final wf = mutated(
        aliasLine,
        '          printf \'keyAlias=%s\\n\' "\$ANDROID_KEY_ALIAS" >&2\n',
      );
      expect(rules(wf.checkSecretLeaks()), ['secret-echo']);
    });

    test('echo secret vào GITHUB_ENV bị bắt: biến sống sang các step sau', () {
      final wf = mutated(
        aliasLine,
        '          echo "KEY=\$ANDROID_KEY_PASSWORD" >> "\$GITHUB_ENV"\n',
      );
      expect(rules(wf.checkSecretLeaks()), ['secret-to-github-file']);
    });

    test('echo nối nhiều dòng bằng dấu \\ vẫn bị bắt', () {
      final wf = mutated(
        aliasLine,
        '          echo "x" \\\n            "\$ANDROID_KEY_PASSWORD"\n',
      );
      expect(rules(wf.checkSecretLeaks()), ['secret-echo']);
    });

    test('bật set -x khi có secret bị bắt', () {
      final wf = mutated(
        '          set -euo pipefail\n          umask 077\n          : "\${ANDROID_KEYSTORE_BASE64',
        '          set -euxo pipefail\n          umask 077\n          : "\${ANDROID_KEYSTORE_BASE64',
      );
      expect(rules(wf.checkSecretLeaks()), ['secret-trace']);
    });

    test('in cả môi trường bị bắt', () {
      final wf = mutated(
        '          umask 077\n          : "\${IOS_CERTIFICATE_P12_BASE64',
        '          umask 077\n          env\n          : "\${IOS_CERTIFICATE_P12_BASE64',
      );
      expect(rules(wf.checkSecretLeaks()), ['env-dump']);
    });

    test('cat key.properties để "gỡ lỗi" bị bắt', () {
      final wf = mutated(aliasLine, '$aliasLine          cat "\$PROPS"\n');
      expect(rules(wf.checkSecretLeaks()), ['print-secret-file']);
    });

    test('pipe vào base64 (cách GitHub hướng dẫn) không bị coi là rò', () {
      expect(loadSample().checkSecretLeaks(), isEmpty);
    });
  });

  group('dọn keychain iOS và keystore Android', () {
    test('thiếu if: always() bị bắt: build đỏ thì keychain ở lại', () {
      final wf = mutated(
        '      - name: $cleanupName\n        if: always()\n',
        '      - name: $cleanupName\n',
      );
      expect(rules(wf.checkKeychainCleanup()), ['keychain-cleanup']);
    });

    test('if: success() bị bắt', () {
      final wf = mutated(
        '$cleanupName\n        if: always()',
        '$cleanupName\n        if: success()',
      );
      expect(wf.checkKeychainCleanup().single.message, contains('always()'));
    });

    test('viết dạng \${{ always() }} vẫn hợp lệ', () {
      final wf = mutated(
        '$cleanupName\n        if: always()',
        '$cleanupName\n        if: \${{ always() }}',
      );
      expect(wf.checkKeychainCleanup(), isEmpty);
    });

    test('xoá lệnh delete-keychain bị bắt', () {
      final wf = mutated(
        '          if [ -f "\$KEYCHAIN_PATH" ]; then security delete-keychain "\$KEYCHAIN_PATH"; fi\n',
        '',
      );
      expect(
        wf.checkKeychainCleanup().single.message,
        contains('delete-keychain'),
      );
    });

    test('dọn một keychain khác với keychain đã tạo bị bắt', () {
      final wf = mutated(
        '          KEYCHAIN_PATH="\$RUNNER_TEMP/toi-noi-signing.keychain-db"\n          if [ -f',
        '          KEYCHAIN_PATH="\$RUNNER_TEMP/other.keychain-db"\n          if [ -f',
      );
      expect(wf.checkKeychainCleanup().single.message, contains('khác'));
    });

    test('không xoá profile ở thư mục MobileDevice bị bắt', () {
      final wf = mutated(
        ' "\$HOME/Library/MobileDevice/Provisioning Profiles/toi-noi-staging.mobileprovision"',
        '',
      );
      expect(
        wf.checkKeychainCleanup().single.message,
        contains('MobileDevice'),
      );
    });

    test('không xoá profile ở thư mục UserData bị bắt', () {
      final wf = mutated(
        ' "\$HOME/Library/Developer/Xcode/UserData/Provisioning Profiles/toi-noi-staging.mobileprovision"',
        '',
      );
      expect(wf.checkKeychainCleanup().single.message, contains('UserData'));
    });

    test('thiếu kiểm rỗng cho một secret bị bắt', () {
      final wf = mutated(
        '          : "\${ANDROID_KEY_PASSWORD:?ANDROID_KEY_PASSWORD is empty}"\n',
        '',
      );
      expect(rules(wf.checkSecretGuards()), ['secret-guard']);
    });

    test('không xoá file p12 còn sót bị bắt', () {
      final wf = mutated('rm -f "\$RUNNER_TEMP/distribution.p12" ', 'rm -f ');
      expect(wf.checkKeychainCleanup().single.message, contains('p12'));
    });

    test('Android: xoá keystore không có always() bị bắt', () {
      final wf = mutated(
        '      - name: Remove keystore and key.properties\n        if: always()\n',
        '      - name: Remove keystore and key.properties\n',
      );
      expect(rules(wf.checkKeystoreCleanup()), ['keystore-cleanup']);
    });
  });

  group('build number', () {
    test('dùng run_id thay run_number bị bắt', () {
      final wf = mutated(r'${{ github.run_number }}', r'${{ github.run_id }}');
      expect(rules(wf.checkBuildNumber()), isNotEmpty);
    });

    test('dùng run_attempt bị bắt: chạy lại sẽ đổi số', () {
      final wf = mutated(
        r"          OFFSET: ${{ vars.BUILD_NUMBER_OFFSET || '0' }}",
        r'          OFFSET: ${{ github.run_attempt }}',
      );
      expect(rules(wf.checkBuildNumber()), contains('build-number'));
    });

    test('hard-code --build-number bị bắt', () {
      final wf = mutated(
        '--build-number="\$BUILD_NUMBER" \\\n            --obfuscate --split-debug-info=build/symbols/android',
        '--build-number=1 \\\n            --obfuscate --split-debug-info=build/symbols/android',
      );
      expect(wf.checkBuildNumber().single.message, contains('android'));
    });

    test('--build-name hard-code bị bắt', () {
      final wf = mutated(
        'flutter build ipa --release',
        'flutter build ipa --build-name=1.0.0 --release',
      );
      expect(wf.checkBuildNumber().single.message, contains('build-name'));
    });

    test('job không needs: prepare bị bắt', () {
      final wf = mutated(
        '    name: iOS IPA (macOS)\n    needs: prepare\n',
        '    name: iOS IPA (macOS)\n',
      );
      expect(wf.checkBuildNumber().single.message, contains('needs'));
    });

    test('bỏ trần versionCode 2100000000 bị bắt', () {
      final wf = mutated('2100000000', '9999999999', all: true);
      expect(rules(wf.checkBuildNumber()), ['build-number']);
    });
  });

  group('lệnh build, runner, artifact', () {
    test('nhãn -latest cho job iOS bị bắt', () {
      final wf = mutated('macos-26', 'macos-latest');
      expect(rules(wf.checkBuildCommands()), ['runner']);
    });

    test('iOS chạy trên ubuntu bị bắt', () {
      final wf = mutated('macos-26', 'ubuntu-24.04');
      expect(rules(wf.checkBuildCommands()), ['runner']);
    });

    test('JDK 21 thay 17 bị bắt', () {
      final wf = mutated("java-version: '17'", "java-version: '21'");
      expect(rules(wf.checkBuildCommands()), ['java']);
    });

    test('quên --flavor staging bị bắt', () {
      final wf = mutated(
        'flutter build appbundle --release --flavor staging',
        'flutter build appbundle --release',
      );
      expect(wf.checkBuildCommands().single.message, contains('--flavor'));
    });

    test('--obfuscate mà thiếu --split-debug-info bị bắt', () {
      final wf = mutated(
        '--obfuscate --split-debug-info=build/symbols/android',
        '--obfuscate',
      );
      expect(wf.checkBuildCommands().single.rule, 'build-command');
    });

    test('thiếu --export-options-plist cho ipa bị bắt', () {
      final wf = mutated(
        '            --export-options-plist=ios/ExportOptions-adhoc.plist \\\n',
        '',
      );
      expect(
        wf.checkBuildCommands().single.message,
        contains('export-options'),
      );
    });

    test('thư mục app sai chính tả bị bắt', () {
      final wf = mutated(
        'working-directory: apps/customer',
        'working-directory: apps/custumer',
        all: true,
      );
      expect(rules(wf.checkAppPaths()), ['app-path', 'app-path']);
    });

    test('if-no-files-found: warn cho artifact bị bắt', () {
      final wf = mutated('if-no-files-found: error', 'if-no-files-found: warn');
      expect(rules(wf.checkArtifacts()), ['artifact']);
    });

    test('retention-days 90 bị bắt', () {
      final wf = mutated('retention-days: 14', 'retention-days: 90');
      expect(rules(wf.checkArtifacts()), ['artifact']);
    });

    test('hai artifact trùng tên bị bắt', () {
      final wf = mutated(
        'toi-noi-customer-staging-android-symbols-',
        'toi-noi-customer-staging-aab-',
      );
      expect(rules(wf.checkArtifacts()), ['artifact']);
    });

    test('bỏ upload IPA bị bắt', () {
      final wf = mutated('/*.ipa', '/*.zip');
      expect(wf.checkArtifacts().single.message, contains('ipa'));
    });
  });

  group('vệ sinh và phần của lab 33', () {
    test('job không có timeout-minutes bị bắt', () {
      final wf = mutated('    timeout-minutes: 45\n', '');
      expect(rules(wf.checkHygiene()), ['timeout']);
    });

    test('checkout giữ token bị bắt', () {
      final wf = mutated(
        '        with:\n          persist-credentials: false\n      - name: Select Xcode',
        '      - name: Select Xcode',
      );
      expect(rules(wf.checkHygiene()), ['checkout']);
    });

    test('cancel-in-progress: true bị bắt', () {
      final wf = mutated(
        'cancel-in-progress: false',
        'cancel-in-progress: true',
      );
      expect(rules(wf.checkHygiene()), ['concurrency']);
    });

    test('action không ghim (checkout@main) bị bộ kiểm lab 33 bắt', () {
      final wf = mutated(
        'actions/checkout@v7.0.1',
        'actions/checkout@main',
        all: true,
      );
      expect(wf.base.checkActionRefs(), hasLength(2));
    });

    test('Flutter lệch phiên bản bị bộ kiểm lab 33 bắt ở cả hai job', () {
      final wf = mutated(
        "flutter-version: '3.47.4'",
        "flutter-version: '3.35.0'",
        all: true,
      );
      expect(wf.base.checkFlutterPin(), hasLength(2));
    });

    test('chuỗi giống bí mật dán vào file bị bộ quét lab 33 bắt', () {
      // Ghép từ mảnh để repo không chứa chuỗi giống bí mật.
      final fake = '-----BEGIN RSA PRIVATE ${'KEY'}-----';
      final wf = mutated('permissions:', '# $fake\npermissions:');
      expect(rules(wf.base.checkNoSecrets()), ['secret']);
    });
  });
}
