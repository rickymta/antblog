import 'dart:io';

import 'package:toi_noi_33/workflow_audit.dart';

/// Gốc repository, tìm từ thư mục chạy test.
final repoRoot = findRepoRoot(Directory.current);

/// Workflow thật của repository.
WorkflowAudit loadRealWorkflow() => WorkflowAudit.load(repoRoot);

/// Bản sao của workflow thật sau khi thay [from] bằng [to]. Ném lỗi nếu
/// [from] không có trong file, để test "ca sai" không vô tình thành no-op.
WorkflowAudit mutated(String from, String to, {bool all = false}) {
  final original = loadRealWorkflow().text;
  if (!original.contains(from)) {
    throw StateError('Workflow thật không chứa "$from"; sửa test.');
  }
  final text = all
      ? original.replaceAll(from, to)
      : original.replaceFirst(from, to);
  return WorkflowAudit(text, repoRoot: repoRoot);
}

/// Thư mục project (có pubspec.yaml) dưới `examples/toi-noi-flutter`, tính từ
/// gốc repository: lab, package dùng chung và tools.
List<String> projectDirs() {
  const base = 'examples/toi-noi-flutter';
  final found = <String>[];
  void scan(Directory dir, String rel, int depth) {
    if (File('${dir.path}/pubspec.yaml').existsSync()) found.add(rel);
    if (depth == 0) return;
    for (final e in dir.listSync().whereType<Directory>()) {
      final name = e.uri.pathSegments.where((s) => s.isNotEmpty).last;
      if (name.startsWith('.') || name == 'build') continue;
      scan(e, '$rel/$name', depth - 1);
    }
  }

  scan(Directory('${repoRoot.path}/$base'), base, 2);
  found.sort();
  return found;
}

/// Đọc một file text của repository.
String readRepoFile(String rel) =>
    File('${repoRoot.path}/$rel').readAsStringSync();
