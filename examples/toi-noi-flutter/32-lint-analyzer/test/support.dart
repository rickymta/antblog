import 'dart:io';

import 'package:path/path.dart' as p;

/// Một chẩn đoán trong đầu ra `dart analyze --format=machine`.
class Diag {
  const Diag(this.severity, this.code, this.file, this.line, this.message);

  /// Đọc một dòng `SEVERITY|TYPE|CODE|FILE|LINE|COL|LENGTH|MESSAGE`.
  factory Diag.parse(String line, String relativeTo) {
    final parts = line.split('|');
    return Diag(
      parts[0],
      parts[2].toLowerCase(),
      p.relative(parts[3], from: relativeTo),
      int.parse(parts[4]),
      parts.sublist(7).join('|'),
    );
  }

  final String severity;
  final String code;
  final String file;
  final int line;
  final String message;

  /// `code@file:line`, đủ để so sánh mà không dính câu chữ của thông báo.
  String get key => '$code@$file:$line';

  @override
  String toString() => '$severity $key';
}

class AnalyzeResult {
  const AnalyzeResult(this.exitCode, this.diagnostics, this.raw);

  final int exitCode;
  final List<Diag> diagnostics;
  final String raw;

  List<String> get keys => [for (final d in diagnostics) d.key];
}

/// Chạy `dart analyze` trên [directory] (tính từ thư mục lab).
Future<AnalyzeResult> analyze(
  String directory, {
  List<String> flags = const [],
}) async {
  final result = await Process.run('dart', [
    'analyze',
    '--format=machine',
    ...flags,
    directory,
  ]);
  final raw = '${result.stdout}${result.stderr}';
  // analyzer in đường dẫn thật (đã qua symlink, như /private/var trên macOS).
  final base = Directory(directory).resolveSymbolicLinksSync();
  final diagnostics = [
    for (final line in raw.split('\n'))
      if (RegExp(r'^(ERROR|WARNING|INFO)\|').hasMatch(line))
        Diag.parse(line, base),
  ];
  return AnalyzeResult(result.exitCode, diagnostics, raw);
}

/// Chạy một lệnh, trả mã thoát và đầu ra gộp.
Future<(int, String)> run(
  String executable,
  List<String> arguments, {
  String? workingDirectory,
  Map<String, String>? environment,
}) async {
  final result = await Process.run(
    executable,
    arguments,
    workingDirectory: workingDirectory,
    environment: environment,
  );
  return (result.exitCode, '${result.stdout}${result.stderr}');
}

/// Sao chép [source] sang [target], bỏ `.dart_tool` và `pubspec.lock`.
void copyDirectory(Directory source, Directory target) {
  target.createSync(recursive: true);
  for (final entity in source.listSync()) {
    final name = p.basename(entity.path);
    if (name == '.dart_tool' || name == 'pubspec.lock') {
      continue;
    }
    if (entity is Directory) {
      copyDirectory(entity, Directory(p.join(target.path, name)));
    } else if (entity is File) {
      entity.copySync(p.join(target.path, name));
    }
  }
}
