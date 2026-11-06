import 'dart:io';

import 'package:test/test.dart';

/// `git check-ignore` trả 0 nếu đường dẫn bị bỏ qua, 1 nếu không, 128 nếu lỗi.
/// `--no-index` để kết quả không phụ thuộc file đã được theo dõi hay chưa.
int checkIgnore(String path) =>
    Process.runSync('git', ['check-ignore', '--no-index', '-q', path]).exitCode;

/// `git check-ignore` chỉ có nghĩa khi đứng trong một repository git.
final String? _noGitReason = () {
  try {
    final r = Process.runSync('git', ['rev-parse', '--is-inside-work-tree']);
    return r.exitCode == 0 ? null : 'không nằm trong git repository';
  } on ProcessException {
    return 'không có lệnh git';
  }
}();

void main() {
  final lines = File('.gitignore')
      .readAsLinesSync()
      .map((l) => l.trim())
      .toList();

  group('.gitignore của lab liệt kê ba mẫu chặn bí mật', () {
    for (final pattern in ['key.properties', '*.jks', '*.keystore']) {
      test(pattern, () => expect(lines, contains(pattern)));
    }

    test('runner anh em neo gốc để không nuốt native-config/', () {
      expect(lines, containsAll(['/android/', '/ios/']));
    });
  });

  group('git check-ignore (đứng trong thư mục lab)', () {
    // Đường dẫn chưa tồn tại cũng kiểm được: git chỉ so mẫu.
    final ignored = [
      // Cố ý nằm ngoài android/ (đã bị /android/ chặn) để thử riêng ba mẫu.
      'app/key.properties',
      'secrets/upload-keystore.jks',
      'keys/release.keystore',
      'key.properties',
    ];
    for (final path in ignored) {
      test(
        'chặn $path',
        () => expect(checkIgnore(path), 0),
        skip: _noGitReason,
      );
    }

    final tracked = [
      'native-config/key.properties.example',
      'native-config/build.gradle.kts',
      'lib/key_properties.dart',
    ];
    for (final path in tracked) {
      test(
        'không chặn $path',
        () => expect(checkIgnore(path), 1),
        skip: _noGitReason,
      );
    }
  });
}
