import 'dart:convert';
import 'dart:io';

import 'package:toi_noi_55/toi_noi_55.dart';

/// Đọc `git log --pretty=format:'%h%x09%s'` từ stdin, in release notes ra stdout.
///
///     git log --pretty=format:'%h%x09%s' v1..HEAD | \
///       dart run bin/release_notes.dart --app customer --version 1.4.0 --build 42
Future<void> main(List<String> args) async {
  final lines = await stdin
      .transform(utf8.decoder)
      .transform(const LineSplitter())
      .toList();
  try {
    final result = runCli(args, lines);
    stdout.writeln(result.notes.text);
    stderr.writeln(result.summary);
  } on UsageError catch (e) {
    stderr.writeln('Lỗi: ${e.message}');
    stderr.writeln(buildParser().usage);
    exitCode = 64;
  } on FormatException catch (e) {
    stderr.writeln('Lỗi: ${e.message}');
    exitCode = 65;
  }
}
