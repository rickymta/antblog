import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

/// Lab commit file sinh (`*.freezed.dart`, `*.g.dart`) để `flutter test` xanh
/// từ trạng thái sạch mà không cần `build_runner`. Test này giữ lời hứa đó:
/// mỗi `part` trỏ tới file sinh thì file ấy phải có mặt và chưa bị ignore.
void main() {
  final lib = Directory('lib');
  final parts = RegExp(
    r"^part '([^']+\.(?:freezed|g)\.dart)';",
    multiLine: true,
  );

  // Danh sách (file nguồn, file sinh) rút từ chính các dòng `part`.
  final expected = <(File, File)>[
    for (final source in lib.listSync(recursive: true).whereType<File>())
      if (source.path.endsWith('.dart') &&
          !source.path.endsWith('.freezed.dart') &&
          !source.path.endsWith('.g.dart'))
        for (final match in parts.allMatches(source.readAsStringSync()))
          (source, File('${source.parent.path}/${match[1]}')),
  ];

  test('có file sinh để kiểm (không phải danh sách rỗng)', () {
    expect(expected.length, greaterThanOrEqualTo(8));
  });

  for (final (source, generated) in expected) {
    test(
      '${generated.path} có mặt, có dấu hiệu máy sinh và part of đúng nguồn',
      () {
        expect(
          generated.existsSync(),
          isTrue,
          reason: 'thiếu: chạy build_runner',
        );
        final text = generated.readAsStringSync();
        expect(text, startsWith('// GENERATED CODE - DO NOT MODIFY BY HAND'));
        final name = source.uri.pathSegments.last;
        expect(text, contains("part of '$name';"));
      },
    );
  }

  test('.gitignore của lab không chặn file sinh', () {
    final lines = File('.gitignore')
        .readAsLinesSync()
        .map((l) => l.trim())
        .where((l) => l.isNotEmpty && !l.startsWith('#'));
    expect(lines.where((l) => l.contains('.g.dart')), isEmpty);
    expect(lines.where((l) => l.contains('.freezed.dart')), isEmpty);
  });

  test(
    'có đủ ba nhóm file sinh: freezed, json_serializable, riverpod_generator',
    () {
      final names = expected.map((e) => e.$2.path).toList();
      expect(names.where((n) => n.endsWith('.freezed.dart')), isNotEmpty);
      expect(names.where((n) => n.endsWith('trip_dto.g.dart')), isNotEmpty);
      expect(names.where((n) => n.endsWith('providers.g.dart')), isNotEmpty);
    },
  );
}
