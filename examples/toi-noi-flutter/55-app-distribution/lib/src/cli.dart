import 'package:args/args.dart';
import 'package:toi_noi_53/release_plan.dart' show AppKind, Flavor;

import 'release_notes.dart';

/// Lỗi cách dùng; `bin/release_notes.dart` đổi thành mã thoát 64.
final class UsageError implements Exception {
  const UsageError(this.message);

  final String message;

  @override
  String toString() => message;
}

/// Kết quả của một lần chạy CLI.
final class CliResult {
  const CliResult(this.notes);

  final ReleaseNotes notes;

  /// Một dòng tóm tắt cho log của CI (không có nội dung commit).
  String get summary =>
      'Release notes: ${notes.included} dòng, ${notes.omitted} bị lược, '
      '${notes.skipped} commit nội bộ bị bỏ, ${notes.length} ký tự.';
}

ArgParser buildParser() => ArgParser()
  ..addOption(
    'app',
    allowed: [for (final a in AppKind.values) a.name],
    help: 'app: customer hoặc driver',
  )
  ..addOption(
    'flavor',
    defaultsTo: 'staging',
    help: 'chỉ staging được phân phối',
  )
  ..addOption('version', help: 'tên phiên bản x.y.z')
  ..addOption('build', help: 'build number (số nguyên dương)')
  ..addOption(
    'max',
    defaultsTo: '$defaultMaxNotesLength',
    help: 'số ký tự tối đa',
  );

/// Chạy CLI: đọc [lines] (mỗi dòng `sha<TAB>subject`) và trả release notes.
CliResult runCli(List<String> args, Iterable<String> lines) {
  final ArgResults options;
  try {
    options = buildParser().parse(args);
  } on FormatException catch (e) {
    throw UsageError(e.message);
  }

  final appName = options['app'] as String?;
  final version = options['version'] as String?;
  final buildText = options['build'] as String?;
  if (appName == null || version == null || buildText == null) {
    throw const UsageError('thiếu một trong --app, --version, --build');
  }
  final flavor = options['flavor'] as String;
  if (flavor != Flavor.staging.name) {
    throw UsageError(
      'flavor "$flavor" không được phân phối qua App Distribution; chỉ staging',
    );
  }
  final build = int.tryParse(buildText);
  final max = int.tryParse(options['max'] as String);
  if (build == null || max == null) {
    throw const UsageError('--build và --max phải là số nguyên');
  }

  final commits = [
    for (final line in lines)
      if (line.trim().isNotEmpty) Commit.parseLine(line),
  ];
  try {
    return CliResult(
      ReleaseNotes.fromCommits(
        commits,
        app: AppKind.values.byName(appName),
        version: version,
        build: build,
        maxLength: max,
      ),
    );
  } on ArgumentError catch (e) {
    throw UsageError('${e.name}: ${e.message}');
  }
}
