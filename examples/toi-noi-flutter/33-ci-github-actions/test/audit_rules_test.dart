import 'package:test/test.dart';
import 'package:toi_noi_33/workflow_audit.dart';

import 'support.dart';

/// Mỗi kiểm tra phải đỏ khi workflow sai. Các ca sai là workflow thật bị sửa
/// đúng một chỗ, nên nếu workflow đổi hình thù thì [mutated] báo lỗi chứ
/// không im lặng thành ca đúng.
void main() {
  List<String> rules(List<Finding> findings) => [
    for (final f in findings) f.rule,
  ];

  group('ghim Flutter', () {
    test('bản khác 3.47.4 bị bắt ở cả hai job', () {
      final audit = mutated(
        "flutter-version: '3.47.4'",
        "flutter-version: '3.35.0'",
        all: true,
      );
      expect(rules(audit.checkFlutterPin()), ['flutter-pin', 'flutter-pin']);
    });

    test('"stable" viết thành "any" hay bỏ ghim đều bị bắt', () {
      final audit = mutated('channel: stable', 'channel: master');
      expect(audit.checkFlutterPin().single.message, contains('master'));
    });
  });

  group('ref của action', () {
    for (final ref in ['main', 'master', 'v7', 'latest', 'HEAD']) {
      test('@$ref bị bắt', () {
        final audit = mutated(
          'actions/checkout@v7.0.1',
          'actions/checkout@$ref',
        );
        expect(rules(audit.checkActionRefs()), ['action-ref']);
      });
    }

    test('thiếu hẳn @ref bị bắt', () {
      final audit = mutated('actions/setup-node@v7.1.0', 'actions/setup-node');
      expect(rules(audit.checkActionRefs()), ['action-ref']);
    });

    test('SHA 40 ký tự hợp lệ, SHA ngắn thì không', () {
      final sha = 'a' * 40;
      expect(
        mutated(
          'actions/checkout@v7.0.1',
          'actions/checkout@$sha',
        ).checkActionRefs(),
        isEmpty,
      );
      final short = mutated(
        'actions/checkout@v7.0.1',
        'actions/checkout@${'a' * 7}',
      );
      expect(rules(short.checkActionRefs()), ['action-ref']);
    });
  });

  group('permissions', () {
    test('thiếu khối permissions bị bắt', () {
      final audit = mutated('permissions:\n  contents: read\n', '');
      expect(rules(audit.checkPermissions()), ['permissions']);
    });

    test('contents: write bị bắt hai lần: quyền ghi và sai giá trị', () {
      final audit = mutated('contents: read', 'contents: write');
      expect(rules(audit.checkPermissions()), ['permissions', 'permissions']);
    });

    test('job tự nâng quyền bị bắt', () {
      final audit = mutated(
        '    timeout-minutes: 120\n',
        '    timeout-minutes: 120\n    permissions:\n      contents: write\n',
      );
      expect(audit.checkPermissions().single.message, contains('job verify'));
    });
  });

  group('bí mật', () {
    // Giá trị giả được ghép từ mảnh để repo không chứa chuỗi giống bí mật.
    final fakes = <String, String>{
      'aws-access-key': 'AK${'IA'}IOSFODNN7EXAMPLE',
      'github-token': 'gh${'p'}_${'a' * 36}',
      'private-key': '-----BEGIN RSA PRIVATE ${'KEY'}-----',
      'google-api-key': 'AI${'za'}${'b' * 35}',
      'slack-token': 'xox${'b'}-1234567890-abcdefghij',
      'inline-credential': 'password: ${'x' * 20}',
    };
    fakes.forEach((name, value) {
      test('mẫu $name bị bắt khi dán vào workflow', () {
        final audit = mutated('permissions:', '# $value\npermissions:');
        expect(
          audit.checkNoSecrets().map((f) => f.message),
          contains(contains(name)),
        );
      });
    });

    test('tham chiếu secrets.NAME hợp lệ không bị coi là bí mật', () {
      final audit = mutated(
        'permissions:',
        '# token: \${{ secrets.DEMO_TOKEN }}\npermissions:',
      );
      expect(audit.checkNoSecrets(), isEmpty);
    });
  });

  group('file được nhắc tới', () {
    test('script verify không có trên đĩa bị bắt', () {
      final audit = mutated('verify-01-06.mjs', 'verify-99-99.mjs');
      expect(
        audit.checkReferencedFiles().single.message,
        contains('verify-99-99.mjs'),
      );
    });

    test('working-directory sai chính tả bị bắt', () {
      final audit = mutated(
        '29-golden-semantics\n        run',
        '29-golden-semantic\n        run',
      );
      expect(rules(audit.checkReferencedFiles()), ['missing-file']);
    });

    test('paths trỏ tới thư mục không có bị bắt', () {
      final audit = mutated(
        "'examples/toi-noi-flutter/**'",
        "'example/toi-noi-flutter/**'",
      );
      expect(
        audit.checkReferencedFiles().map((f) => f.rule),
        everyElement('missing-file'),
      );
    });
  });

  group('trigger theo path', () {
    test('quên workflow trong paths: sửa workflow không chạy CI', () {
      final audit = mutated(
        "      - '.github/workflows/toi-noi-flutter.yml'\n",
        '',
        all: true,
      );
      expect(audit.checkPathsTrigger(projectDirs()), hasLength(2));
    });

    test('đổi ** thành * làm mất lab nằm sâu một cấp', () {
      final audit = mutated(
        "'examples/toi-noi-flutter/**'",
        "'examples/toi-noi-flutter/*'",
        all: true,
      );
      expect(audit.checkPathsTrigger(projectDirs()), isNotEmpty);
    });

    test('bỏ hẳn paths của pull_request bị bắt', () {
      final audit = mutated(
        '  pull_request:\n    paths:\n      - \'examples/toi-noi-flutter/**\'\n      - \'.github/workflows/toi-noi-flutter.yml\'\n',
        '  pull_request:\n',
      );
      expect(
        audit.checkPathsTrigger(projectDirs()).single.message,
        contains('pull_request'),
      );
    });
  });

  group('glob của bộ lọc paths', () {
    test('* không vượt qua dấu /, ** thì có', () {
      expect(pathsFilterMatches(['docs/*'], 'docs/a.md'), isTrue);
      expect(pathsFilterMatches(['docs/*'], 'docs/x/a.md'), isFalse);
      expect(pathsFilterMatches(['docs/**'], 'docs/x/a.md'), isTrue);
    });

    test('dấu chấm là chữ, không phải ký tự đại diện', () {
      expect(pathsFilterMatches(['a.yml'], 'aXyml'), isFalse);
      expect(pathsFilterMatches(['a.yml'], 'a.yml'), isTrue);
    });
  });

  group('vệ sinh vận hành', () {
    test('job không có timeout-minutes bị bắt', () {
      final audit = mutated('    timeout-minutes: 20\n', '');
      expect(rules(audit.checkHygiene()), ['timeout']);
    });

    test('push không giới hạn nhánh bị bắt', () {
      final audit = mutated('    branches: [main]\n', '');
      expect(rules(audit.checkHygiene()), ['push-branches']);
    });

    test('thiếu concurrency bị bắt', () {
      final audit = mutated(
        "concurrency:\n  group: \${{ github.workflow }}-\${{ github.ref == 'refs/heads/main' && github.sha || github.ref }}\n  cancel-in-progress: \${{ github.ref != 'refs/heads/main' }}\n",
        '',
      );
      expect(rules(audit.checkHygiene()), ['concurrency']);
    });

    test('khoá cache chỉ băm pubspec.lock bị bắt', () {
      final audit = mutated(
        "hashFiles('examples/toi-noi-flutter/**/pubspec.yaml')",
        "hashFiles('**/pubspec.lock')",
        all: true,
      );
      expect(rules(audit.checkPubCacheKey()), [
        'pub-cache-key',
        'pub-cache-key',
      ]);
    });
  });
}
