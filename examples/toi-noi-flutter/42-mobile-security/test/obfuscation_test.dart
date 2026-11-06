@Timeout(Duration(minutes: 3))
library;

import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:test/test.dart';
import 'package:toi_noi_42/toi_noi_42.dart';

/// Biên dịch `tool/obfuscation_target.dart` thành AOT snapshot bằng
/// `dart compile aot-snapshot` và chạy bằng `dartaotruntime`. Đây là bộ biên
/// dịch AOT của Dart, cùng `gen_snapshot` mà `flutter build --obfuscate` dùng,
/// nhưng KHÔNG phải một lần `flutter build`: không có Android/iOS ở đây, và
/// không thử `--split-debug-info` (xem README).
void main() {
  late Directory out;
  late _Run plain;
  late _Run stripOnly;
  late _Run obfuscated;
  late ObfuscationMap map;

  setUpAll(() async {
    out = await Directory.systemTemp.createTemp('toi_noi_42_obf_');
    final mapFile = '${out.path}/map.json';
    // flutter_tools mặc định strip snapshot (flutter_tools/lib/src/base/build.dart:
    // iOS strip sau khi xuất dSYM, Android để AGP lo, nền tảng khác thêm
    // `--strip`). Bản "obf" dùng `--strip` để gần với bản phát hành.
    final results = await Future.wait([
      _build(out, 'plain', const []),
      _build(out, 'strip', const ['--strip']),
      _build(out, 'obf', [
        '--obfuscate',
        '--strip',
        '--save-obfuscation-map=$mapFile',
      ]),
    ]);
    plain = results[0];
    stripOnly = results[1];
    obfuscated = results[2];
    map = ObfuscationMap.parse(await File(mapFile).readAsString());
  });

  tearDownAll(() => out.delete(recursive: true));

  test('bản thường: stack trace, runtimeType và file đều có tên thật', () {
    expect(plain.stdout, contains('type=QuoteCalculator'));
    expect(plain.stdout, contains('QuoteCalculator.fareFor'));
    expect(plain.snapshotContains('QuoteCalculator'), isTrue);
  });

  test('--strip một mình không giấu tên: chỉ --obfuscate mới đổi tên', () {
    expect(stripOnly.snapshotContains('QuoteCalculator'), isTrue);
    expect(stripOnly.snapshotContains('fareFor'), isTrue);
  });

  test(
    '--obfuscate: tên lớp và hàm đổi trong stack trace, runtimeType và file',
    () {
      expect(obfuscated.stdout, isNot(contains('QuoteCalculator')));
      expect(obfuscated.stdout, isNot(contains('fareFor')));
      expect(obfuscated.snapshotContains('QuoteCalculator'), isFalse);
      expect(obfuscated.snapshotContains('fareFor'), isFalse);
    },
  );

  test(
    '--obfuscate không đụng tới chuỗi hằng: "bí mật" vẫn nằm nguyên trong file',
    () {
      for (final run in [plain, stripOnly, obfuscated]) {
        expect(run.stdout, contains('secret=demo-khoa-api-gia-lap-0001'));
        expect(run.snapshotContains('demo-khoa-api-gia-lap-0001'), isTrue);
      }
    },
  );

  test('tên enum không bị làm rối (đúng như tài liệu Flutter ghi)', () {
    expect(obfuscated.stdout, contains('enum=TripPhase.searching'));
  });

  test('có bản đồ thì đọc lại được stack trace về tên gốc', () {
    final trace = obfuscated.stdout
        .split('\n')
        .where((line) => line.startsWith('#'))
        .join('\n');
    expect(trace, isNot(contains('fareFor')));

    final restored = map.deobfuscateStackTrace(trace);

    expect(restored, contains('QuoteCalculator.fareFor'));
    expect(restored, contains('tool/obfuscation_target.dart:'));
  });

  group('ObfuscationMap', () {
    test('mảng lẻ hoặc không phải chuỗi bị từ chối', () {
      expect(() => ObfuscationMap.parse('["a"]'), throwsFormatException);
      expect(() => ObfuscationMap.parse('{"a":"b"}'), throwsFormatException);
      expect(() => ObfuscationMap.parse('["a", 1]'), throwsFormatException);
    });

    test('chỉ dịch tên hàm và tên file của frame, không động tới thông báo lỗi', () {
      final tiny = ObfuscationMap.parse(
        '["MaterialApp","ex","Scaffold","state","a.dart","xN"]',
      );
      const trace =
          'Bad state: Scaffold state\n#0      ex.build (xN:12)\n#1      state.<anonymous closure> (xN:7:3)';

      expect(
        tiny.deobfuscateStackTrace(trace),
        'Bad state: Scaffold state\n'
        '#0      MaterialApp.build (a.dart:12)\n'
        '#1      Scaffold.<anonymous closure> (a.dart:7:3)',
      );
    });
  });
}

final class _Run {
  _Run(this.stdout, this.snapshotBytes);

  final String stdout;
  final Uint8List snapshotBytes;

  bool snapshotContains(String text) {
    final needle = utf8.encode(text);
    outer:
    for (var i = 0; i <= snapshotBytes.length - needle.length; i++) {
      for (var j = 0; j < needle.length; j++) {
        if (snapshotBytes[i + j] != needle[j]) continue outer;
      }
      return true;
    }
    return false;
  }
}

Future<_Run> _build(
  Directory out,
  String name,
  List<String> snapshotOptions,
) async {
  final snapshot = '${out.path}/$name.aot';
  final compile = await Process.run(Platform.resolvedExecutable, [
    'compile',
    'aot-snapshot',
    if (snapshotOptions.isNotEmpty)
      '--extra-gen-snapshot-options=${snapshotOptions.join(',')}',
    'tool/obfuscation_target.dart',
    '-o',
    snapshot,
  ]);
  if (compile.exitCode != 0) {
    throw StateError('dart compile lỗi: ${compile.stdout}${compile.stderr}');
  }
  final runtime =
      '${File(Platform.resolvedExecutable).parent.path}/dartaotruntime';
  final run = await Process.run(runtime, [snapshot]);
  return _Run('${run.stdout}${run.stderr}', await File(snapshot).readAsBytes());
}
