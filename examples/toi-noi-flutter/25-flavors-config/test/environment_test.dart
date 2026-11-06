// Hai nhóm test dưới đây loại trừ nhau theo cách chạy:
//
//   flutter test                                              -> chỉ nhóm "không define"
//   flutter test --dart-define-from-file=config/staging.json  -> chỉ nhóm "có define"
//
// `tool/verify-flavors.mjs` chạy bộ test theo nhiều cách (mặc định, ba flavor, chỉ file, hai ca cố ý thất bại).
import 'package:flutter/services.dart' show appFlavor;
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_25/config/app_config.dart';
import 'package:toi_noi_25/config/flavor.dart';

import 'support/config_files.dart';

// Phải là const: đây chính là điều bài viết dạy.
const _flavorDefine = String.fromEnvironment('FLAVOR');
const _hasDefines = _flavorDefine != '';

void main() {
  group(
    'chạy không có define',
    skip: _hasDefines ? 'đang chạy với --dart-define-from-file' : false,
    () {
      test(
        'fromEnvironment báo thiếu cả năm khoá, không chạy với giá trị đoán',
        () {
          final error = _thrown(AppConfig.fromEnvironment);
          expect(error.problems, hasLength(AppConfig.keys.length));
          expect(error.problems, everyElement(contains('thiếu khoá')));
        },
      );

      test('khoá không có define: hasEnvironment sai, String.fromEnvironment trả rỗng', () {
        expect(const bool.hasEnvironment('FLAVOR'), isFalse);
        expect(const String.fromEnvironment('FLAVOR'), isEmpty);
      });

      test('không có --flavor thì appFlavor là null', () {
        expect(appFlavor, isNull);
      });
    },
  );

  group('chạy với --dart-define-from-file', skip: _hasDefines ? false : 'không có define; xem tool/verify-flavors.mjs', () {
    final flavor = Flavor.tryParse(_flavorDefine);

    test('FLAVOR là một trong ba flavor', () {
      expect(flavor, isNotNull);
    });

    test('fromEnvironment cho đúng cấu hình của file cùng tên flavor', () {
      final fromBuild = AppConfig.fromEnvironment();
      final fromFile = AppConfig.fromMap(readFlavorDefines(flavor!.name));
      expect(fromBuild, fromFile);
    });

    test(
      'lời gọi không const: trong flutter test cho chuỗi rỗng dù define có mặt',
      () {
        // Cùng tên khoá, cùng lần chạy, chỉ khác có `const` hay không. Tài liệu
        // chỉ đảm bảo bản `const`; nền tảng khác có thể cho kết quả khác.
        final nonConst = String.fromEnvironment('FLAVOR');
        expect(nonConst, isEmpty);
        expect(const String.fromEnvironment('FLAVOR'), _flavorDefine);
      },
    );

    test('khoá khai báo thì hasEnvironment đúng', () {
      expect(const bool.hasEnvironment('FLAVOR'), isTrue);
      expect(const bool.hasEnvironment('KHONG_CO'), isFalse);
    });

    test('--flavor, nếu có, đã được so khớp với FLAVOR', () {
      expect(appFlavor == null || appFlavor == flavor!.name, isTrue);
    });
  });
}

ConfigException _thrown(AppConfig Function() body) {
  try {
    body();
  } on ConfigException catch (error) {
    return error;
  }
  fail('đáng lẽ ném ConfigException');
}
