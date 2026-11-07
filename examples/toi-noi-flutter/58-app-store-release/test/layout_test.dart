import 'dart:io';

import 'package:test/test.dart';
import 'package:toi_noi_58/toi_noi_58.dart';

/// Tên file `fastlane deliver` đọc trong thư mục ngôn ngữ và thư mục gốc của
/// metadata (docs.fastlane.tools/actions/deliver, kiểm tại 11/2026).
const _localizedFiles = {
  'name.txt',
  'subtitle.txt',
  'privacy_url.txt',
  'description.txt',
  'keywords.txt',
  'release_notes.txt',
  'support_url.txt',
  'marketing_url.txt',
  'promotional_text.txt',
};
const _reviewFiles = {
  'first_name.txt',
  'last_name.txt',
  'phone_number.txt',
  'email_address.txt',
  'demo_user.txt',
  'demo_password.txt',
  'notes.txt',
};
const _rootFiles = {
  'copyright.txt',
  'primary_category.txt',
  'secondary_category.txt',
};

Set<String> names(Directory d) => {
  for (final f in d.listSync().whereType<File>()) f.uri.pathSegments.last,
};

void main() {
  for (final app in StoreApp.values) {
    group('bố cục metadata của ${app.name}', () {
      final root = Directory('store/${app.name}/metadata');

      test('mọi file mang tên mà fastlane deliver biết', () {
        expect(
          _rootFiles.containsAll(names(root)),
          isTrue,
          reason: '${names(root)}',
        );
        expect(
          _localizedFiles.containsAll(names(Directory('${root.path}/vi'))),
          isTrue,
        );
        expect(
          _reviewFiles.containsAll(
            names(Directory('${root.path}/review_information')),
          ),
          isTrue,
        );
      });

      test('có ngôn ngữ vi và đủ file bắt buộc cho phiên bản đầu', () {
        final vi = names(Directory('${root.path}/vi'));
        expect(
          vi,
          containsAll([
            'name.txt',
            'keywords.txt',
            'description.txt',
            'support_url.txt',
            'privacy_url.txt',
          ]),
        );
        expect(
          vi,
          isNot(contains('release_notes.txt')),
          reason: 'phiên bản đầu không có What\'s New',
        );
      });

      test('mọi file kết thúc bằng xuống dòng và dùng UTF-8 hợp lệ', () {
        for (final f in root.listSync(recursive: true).whereType<File>()) {
          final text = f
              .readAsStringSync(); // ném FormatException nếu không phải UTF-8
          expect(text.endsWith('\n'), isTrue, reason: f.path);
        }
      });
    });
  }

  group('dữ liệu hư cấu, không lộ bí mật', () {
    test('mọi URL trong metadata trỏ tới toinoi.example', () {
      final url = RegExp(r'https?://[^\s)]+');
      for (final f in Directory(
        'store',
      ).listSync(recursive: true).whereType<File>()) {
        for (final m in url.allMatches(f.readAsStringSync())) {
          expect(Uri.parse(m.group(0)!).host, 'toinoi.example', reason: f.path);
        }
      }
    });

    test('mọi email kết thúc bằng .example', () {
      final email = RegExp(r'[\w.+-]+@[\w.-]+');
      for (final f in Directory(
        'store',
      ).listSync(recursive: true).whereType<File>()) {
        for (final m in email.allMatches(f.readAsStringSync())) {
          expect(m.group(0)!.endsWith('.example'), isTrue, reason: f.path);
        }
      }
    });

    test('mật khẩu demo là mã OTP hư cấu 6 chữ số, không nằm trong notes, khác 123456 của fake API', () {
      for (final app in StoreApp.values) {
        final review = loadReleaseInput(Directory.current, app).metadata.review;
        expect(review.demoPassword, matches(RegExp(r'^\d{6}$')));
        expect(review.demoPassword, isNot('123456'));
        expect(review.demoPassword, isNot('000000'));
        expect(review.notes, isNot(contains(review.demoPassword!)));
      }
    });

    test(
      'notes nói tài khoản demo nằm trên production và tự tạo lại sau khi xoá',
      () {
        for (final app in StoreApp.values) {
          final notes = loadReleaseInput(
            Directory.current,
            app,
          ).metadata.review.notes!;
          expect(notes, contains('production'));
          expect(notes, contains('tự tạo lại'));
          expect(notes, isNot(contains('Máy chủ duyệt')));
          expect(notes, isNot(contains('Môi trường duyệt')));
        }
      },
    );

    test('.gitignore neo /android/ và /ios/ để không nuốt native-config/', () {
      final lines = File('.gitignore').readAsLinesSync();
      expect(lines, containsAll(['/android/', '/ios/']));
      expect(lines, isNot(contains('ios/')));
    });
  });

  group('bin/check.dart', () {
    test('gói mẫu chỉ thiếu ảnh chụp (repo không chứa ảnh), mã thoát 1', () {
      final result = Process.runSync(Platform.resolvedExecutable, [
        'bin/check.dart',
        'customer',
      ]);
      expect(result.exitCode, 1);
      final lines = (result.stdout as String).trim().split('\n');
      expect(lines.first, 'vn.toinoi.customer: 1 phát hiện');
      expect(lines.last, contains('screenshot-iphone-required-size'));
    });

    test('tên app lạ trả mã 64', () {
      final result = Process.runSync(Platform.resolvedExecutable, [
        'bin/check.dart',
        'rider',
      ]);
      expect(result.exitCode, 64);
    });
  });
}
