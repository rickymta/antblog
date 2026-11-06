import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'support.dart';

const _longRun = Timeout(Duration(minutes: 5));

void main() {
  group('ba cờ strict của analyzer', () {
    test('mã "trước" bị bắt: 2 lỗi, 3 cảnh báo', () async {
      final result = await analyze('fixtures/strict_before');

      expect(
        result.keys,
        unorderedEquals([
          'invalid_assignment@lib/before.dart:6', // strict-casts
          'invalid_assignment@lib/before.dart:7', // strict-casts
          'strict_raw_type@lib/before.dart:12', // strict-raw-types
          'inference_failure_on_collection_literal@lib/before.dart:15',
          'inference_failure_on_instance_creation@lib/before.dart:16',
        ]),
      );
      expect(result.exitCode, 3, reason: 'có error thì dart analyze thoát 3');
    }, timeout: _longRun);

    test('mã "sau" sạch với cùng ba cờ', () async {
      final result = await analyze('fixtures/strict_after');

      expect(result.diagnostics, isEmpty);
      expect(result.exitCode, 0);
    }, timeout: _longRun);

    test('mã "trước" không có cờ nào thì analyzer im lặng', () async {
      final dir = Directory.systemTemp.createTempSync('toi_noi_32_default_');
      addTearDown(() => dir.deleteSync(recursive: true));
      Directory(p.join(dir.path, 'lib')).createSync();
      File('fixtures/strict_before/lib/before.dart')
          .copySync(p.join(dir.path, 'lib', 'before.dart'));
      File(p.join(dir.path, 'analysis_options.yaml')).writeAsStringSync('');

      final result = await analyze(dir.path);

      expect(result.diagnostics, isEmpty);
      expect(result.exitCode, 0);
    }, timeout: _longRun);

    // Mỗi cờ chặn đúng một loại lỗ hổng, không lấn sang loại khác.
    const perFlag = {
      'strict-casts': [
        'invalid_assignment@lib/before.dart:6',
        'invalid_assignment@lib/before.dart:7',
      ],
      'strict-raw-types': ['strict_raw_type@lib/before.dart:12'],
      'strict-inference': [
        'inference_failure_on_collection_literal@lib/before.dart:15',
        'inference_failure_on_instance_creation@lib/before.dart:16',
      ],
    };
    for (final MapEntry(key: flag, value: expected) in perFlag.entries) {
      test('chỉ bật $flag', () async {
        final dir = Directory.systemTemp.createTempSync('toi_noi_32_flag_');
        addTearDown(() => dir.deleteSync(recursive: true));
        Directory(p.join(dir.path, 'lib')).createSync();
        File('fixtures/strict_before/lib/before.dart')
            .copySync(p.join(dir.path, 'lib', 'before.dart'));
        File(p.join(dir.path, 'analysis_options.yaml'))
            .writeAsStringSync('analyzer:\n  language:\n    $flag: true\n');

        final result = await analyze(dir.path);

        expect(result.keys, unorderedEquals(expected));
      }, timeout: _longRun);
    }
  });
}
