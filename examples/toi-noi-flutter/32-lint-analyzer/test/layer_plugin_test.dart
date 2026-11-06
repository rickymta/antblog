import 'package:flutter_test/flutter_test.dart';

import 'support.dart';

/// Plugin được nạp bởi chính analysis server, nên khởi động có thể chậm
/// ở lần đầu trên máy tải nặng.
const _longRun = Timeout(Duration(minutes: 5));

void main() {
  group('toi_noi_lints qua dart analyze thật', () {
    setUpAll(() async {
      final (code, output) = await run('dart', [
        'pub',
        'get',
      ], workingDirectory: 'fixtures/layers');
      expect(code, 0, reason: output);
    });

    test('bắt file presentation import data, ở mức warning', () async {
      final result = await analyze('fixtures/layers');

      expect(
        result.keys,
        contains(
          'no_data_import_in_presentation@lib/presentation/bad_screen.dart:1',
        ),
      );
      final bad = result.diagnostics.firstWhere(
        (d) => d.file.endsWith('bad_screen.dart'),
      );
      expect(bad.severity, 'WARNING');
      expect(bad.message, contains("'../data/fare_table_repository.dart'"));
      expect(
        result.exitCode,
        2,
        reason: 'warning làm dart analyze thoát 2 dù không có --fatal-infos',
      );
    }, timeout: _longRun);

    test('ignore phải có tiền tố tên plugin mới chặn được', () async {
      final result = await analyze('fixtures/layers');

      // ignored_screen.dart dùng `toi_noi_lints/...`: im lặng.
      // ignored_plain_screen.dart dùng tên trần: vẫn bị báo ở dòng import (dòng 2).
      expect(
        result.keys,
        unorderedEquals([
          'no_data_import_in_presentation@lib/presentation/bad_screen.dart:1',
          'no_data_import_in_presentation@lib/presentation/ignored_plain_screen.dart:2',
        ]),
      );
    }, timeout: _longRun);

    test(
      'bẫy: chỉ định thư mục lib thì chẩn đoán của plugin biến mất',
      () async {
        final wholePackage = await analyze('fixtures/layers');
        final libOnly = await analyze('fixtures/layers/lib');
        final oneFile = await analyze(
          'fixtures/layers/lib/presentation/bad_screen.dart',
        );

        expect(wholePackage.diagnostics, isNotEmpty);
        expect(libOnly.diagnostics, isEmpty);
        expect(
          libOnly.exitCode,
          0,
          reason: 'thoát 0 dù có vi phạm: cổng thủng',
        );
        expect(oneFile.diagnostics.map((d) => d.code), [
          'no_data_import_in_presentation',
        ]);
      },
      timeout: _longRun,
    );

    test('pub.dev không với tới được thì dart analyze thoát 4', () async {
      final (code, output) = await run(
        'dart',
        ['analyze', 'fixtures/layers'],
        environment: {'PUB_HOSTED_URL': 'http://127.0.0.1:9'},
      );

      expect(code, 4);
      expect(output, contains('analyzer plugin'));
    }, timeout: _longRun);

    test('composition root import data và không bị báo', () async {
      final result = await analyze('fixtures/layers');

      expect(
        result.diagnostics.where((d) => d.file.endsWith('composition.dart')),
        isEmpty,
      );
    }, timeout: _longRun);

    test(
      'rule tắt mặc định: không có khối diagnostics thì không báo gì',
      () async {
        final (code, output) = await run('dart', [
          'pub',
          'get',
        ], workingDirectory: 'fixtures/layers_off');
        expect(code, 0, reason: output);

        final result = await analyze('fixtures/layers_off');

        expect(result.diagnostics, isEmpty);
        expect(result.exitCode, 0);
      },
      timeout: _longRun,
    );
  });
}
