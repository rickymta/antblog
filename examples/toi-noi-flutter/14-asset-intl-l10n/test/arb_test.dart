import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

Map<String, dynamic> readArb(String name) {
  final text = File('lib/l10n/$name').readAsStringSync();
  return jsonDecode(text) as Map<String, dynamic>;
}

/// Các khoá thông điệp: bỏ `@@locale` và các khoá metadata bắt đầu bằng `@`.
Set<String> messageKeys(Map<String, dynamic> arb) =>
    arb.keys.where((k) => !k.startsWith('@')).toSet();

void main() {
  final vi = readArb('app_vi.arb');
  final en = readArb('app_en.arb');

  group('cấu hình gen-l10n', () {
    final config = File('l10n.yaml').readAsStringSync();

    test('ARB mẫu là tiếng Việt, locale ưu tiên là vi', () {
      expect(config, contains('template-arb-file: app_vi.arb'));
      expect(config, contains('preferred-supported-locales: [vi]'));
    });

    test('không dùng synthetic-package (đã bị gỡ khỏi Flutter)', () {
      expect(config, isNot(contains('synthetic-package')));
    });

    test('code sinh không được commit', () {
      final ignore = File('.gitignore').readAsStringSync();
      expect(ignore, contains('lib/l10n/app_localizations*.dart'));
    });
  });

  group('hai file ARB khớp nhau', () {
    test('@@locale trùng hậu tố tên file', () {
      expect(vi['@@locale'], 'vi');
      expect(en['@@locale'], 'en');
    });

    test('cùng một bộ khoá, không thiếu, không thừa', () {
      expect(messageKeys(en), messageKeys(vi));
    });

    test('mỗi placeholder khai báo ở bản mẫu đều có mặt trong bản en', () {
      for (final key in messageKeys(vi)) {
        final meta = vi['@$key'] as Map<String, dynamic>?;
        final placeholders = (meta?['placeholders'] as Map?)?.keys ?? const [];
        for (final name in placeholders) {
          expect(vi[key], contains('{$name'), reason: 'vi: $key thiếu {$name}');
          expect(en[key], contains('{$name'), reason: 'en: $key thiếu {$name}');
        }
      }
    });

    test('mọi câu plural đều có nhánh other', () {
      for (final arb in [vi, en]) {
        for (final key in messageKeys(arb)) {
          final text = arb[key] as String;
          if (text.contains(', plural,')) {
            expect(text, contains('other{'), reason: '$key thiếu other');
          }
        }
      }
    });

    test(
      'bản mẫu mô tả những chuỗi có tham số để người dịch hiểu ngữ cảnh',
      () {
        for (final key in [
          'quoteDistance',
          'quoteFare',
          'quoteRequestedAt',
          'driversNearby',
        ]) {
          final meta = vi['@$key'] as Map<String, dynamic>;
          expect(meta['description'], isNotEmpty, reason: key);
        }
      },
    );
  });
}
