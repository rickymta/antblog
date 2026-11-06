import 'dart:io';

import 'package:yaml/yaml.dart';

/// Phiên bản Flutter mà mọi lab của loạt bài đã kiểm.
const expectedFlutterVersion = '3.47.4';

/// Đường dẫn workflow thật, tính từ gốc repository.
const workflowRelativePath = '.github/workflows/toi-noi-flutter.yml';

/// Một phát hiện của bộ kiểm tra: [rule] là tên ổn định để test so sánh.
class Finding {
  const Finding(this.rule, this.message);

  final String rule;
  final String message;

  @override
  String toString() => '$rule: $message';
}

/// Đi ngược từ [start] tới thư mục chứa workflow thật; ném [StateError]
/// nếu không thấy (ví dụ lab bị chép ra ngoài repository).
Directory findRepoRoot(Directory start) {
  var dir = start.absolute;
  while (true) {
    if (File('${dir.path}/$workflowRelativePath').existsSync()) return dir;
    final parent = dir.parent;
    if (parent.path == dir.path) {
      throw StateError('Không thấy $workflowRelativePath từ ${start.path}');
    }
    dir = parent;
  }
}

/// Chuyển pattern của bộ lọc `paths` thành regex theo bảng "filter pattern
/// cheat sheet" của GitHub: `*` không khớp `/`, `**` khớp mọi ký tự.
RegExp globToRegExp(String glob) {
  final out = StringBuffer('^');
  for (var i = 0; i < glob.length; i++) {
    final c = glob[i];
    if (c == '*') {
      if (i + 1 < glob.length && glob[i + 1] == '*') {
        out.write('.*');
        i++;
      } else {
        out.write('[^/]*');
      }
    } else {
      out.write(RegExp.escape(c));
    }
  }
  out.write(r'$');
  return RegExp(out.toString());
}

/// `true` nếu [path] khớp ít nhất một pattern trong [globs].
bool pathsFilterMatches(Iterable<String> globs, String path) =>
    globs.any((g) => globToRegExp(g).hasMatch(path));

/// `uses:` có ref cố định: tag đủ ba số (`v7.0.1`) hoặc SHA 40 ký tự hex.
/// Action cục bộ (`./...`) không có ref nên không xét.
final _exactRef = RegExp(r'^(v\d+\.\d+\.\d+|[0-9a-f]{40})$');

/// Các mẫu của bí mật dễ nhận ra. Chuỗi mẫu được ghép từ mảnh để chính file
/// này không bị công cụ quét bí mật coi là bí mật.
final secretPatterns = <String, RegExp>{
  'aws-access-key': RegExp('AK${'IA'}[0-9A-Z]{16}'),
  'github-token': RegExp('gh[pousr]_[A-Za-z0-9]{36,}'),
  'private-key': RegExp('-----BEGIN [A-Z ]*PRIVATE ${'KEY'}-----'),
  'google-api-key': RegExp('AI${'za'}[0-9A-Za-z_-]{35}'),
  'slack-token': RegExp('xox[abprs]-[0-9A-Za-z-]{10,}'),
  // Gán thẳng một giá trị dài cho tên nghe như bí mật, trừ `${{ ... }}`.
  'inline-credential': RegExp(
    r'''(password|secret|token|api[_-]?key)\s*[:=]\s*['"]?(?!\$\{\{)[A-Za-z0-9+/_\-]{16,}''',
    caseSensitive: false,
  ),
};

/// Bộ kiểm tra một workflow đã đọc vào bộ nhớ.
///
/// [text] là nội dung gốc (để quét chuỗi), [repoRoot] là nơi tìm các file mà
/// workflow nhắc tới.
class WorkflowAudit {
  WorkflowAudit(this.text, {required this.repoRoot})
    : _doc = loadYaml(text) as YamlMap;

  factory WorkflowAudit.load(Directory repoRoot) => WorkflowAudit(
    File('${repoRoot.path}/$workflowRelativePath').readAsStringSync(),
    repoRoot: repoRoot,
  );

  final String text;
  final Directory repoRoot;
  final YamlMap _doc;

  YamlMap get document => _doc;

  /// Khối `on:`. Với YAML 1.2 (package `yaml`) khoá `on` là chuỗi, không
  /// bị đọc thành `true` như YAML 1.1.
  YamlMap get triggers => _doc['on'] as YamlMap;

  YamlMap get jobs => _doc['jobs'] as YamlMap;

  /// Tên job theo thứ tự khai báo trong file.
  List<String> get jobNames => [for (final k in jobs.keys) k as String];

  /// Mọi step của mọi job, kèm tên job.
  Iterable<(String, YamlMap)> get steps sync* {
    for (final entry in jobs.entries) {
      final job = entry.value as YamlMap;
      for (final step in (job['steps'] as YamlList? ?? YamlList())) {
        yield (entry.key as String, step as YamlMap);
      }
    }
  }

  /// Step nào của [job] có `uses` bắt đầu bằng [action].
  Iterable<YamlMap> usesOf(String action, {String? job}) => [
    for (final (j, s) in steps)
      if ((job == null || j == job) &&
          (s['uses'] as String? ?? '').startsWith('$action@'))
        s,
  ];

  /// Flutter phải được ghim đúng [expectedFlutterVersion] và channel stable.
  List<Finding> checkFlutterPin() {
    final found = <Finding>[];
    final uses = usesOf('subosito/flutter-action').toList();
    if (uses.isEmpty) {
      found.add(
        const Finding('flutter-pin', 'không có subosito/flutter-action'),
      );
    }
    for (final step in uses) {
      final with_ = step['with'] as YamlMap? ?? YamlMap();
      final version = with_['flutter-version'];
      if (version != expectedFlutterVersion) {
        found.add(
          Finding(
            'flutter-pin',
            'flutter-version là "$version", cần "$expectedFlutterVersion"',
          ),
        );
      }
      if (with_['channel'] != 'stable') {
        found.add(
          Finding(
            'flutter-pin',
            'channel là "${with_['channel']}", cần stable',
          ),
        );
      }
    }
    return found;
  }

  /// Mọi `uses:` từ bên ngoài phải có ref cố định, không nhánh trôi.
  List<Finding> checkActionRefs() {
    final found = <Finding>[];
    for (final (job, step) in steps) {
      final uses = step['uses'] as String?;
      if (uses == null || uses.startsWith('./')) continue;
      final at = uses.lastIndexOf('@');
      final ref = at < 0 ? '' : uses.substring(at + 1);
      if (!_exactRef.hasMatch(ref)) {
        found.add(
          Finding(
            'action-ref',
            'job $job: "$uses" không ghim tag đủ ba số hay SHA 40 ký tự',
          ),
        );
      }
    }
    return found;
  }

  /// `permissions` ở gốc phải có và chỉ cho đọc; job không được nâng quyền.
  List<Finding> checkPermissions() {
    final found = <Finding>[];
    final top = _doc['permissions'];
    if (top == null) {
      found.add(const Finding('permissions', 'thiếu khối permissions ở gốc'));
    }
    final blocks = <String, Object?>{'gốc': top};
    for (final entry in jobs.entries) {
      blocks['job ${entry.key}'] = (entry.value as YamlMap)['permissions'];
    }
    for (final entry in blocks.entries) {
      final value = entry.value;
      final granted = switch (value) {
        null => <String>[],
        final String s => [s],
        final YamlMap m => [for (final v in m.values) '$v'],
        _ => ['?'],
      };
      if (granted.any((v) => v != 'read' && v != 'none')) {
        found.add(
          Finding(
            'permissions',
            '${entry.key} cấp quyền ngoài read/none: $granted',
          ),
        );
      }
    }
    final contents = top is YamlMap ? top['contents'] : null;
    if (top != null && contents != 'read') {
      found.add(
        Finding('permissions', 'contents phải là read, đang là $contents'),
      );
    }
    return found;
  }

  /// Chuỗi trông như bí mật trong cả file (kể cả comment).
  List<Finding> checkNoSecrets() => [
    for (final entry in secretPatterns.entries)
      if (entry.value.hasMatch(text))
        Finding('secret', 'khớp mẫu ${entry.key}'),
  ];

  /// File và thư mục mà workflow nhắc tới phải có thật: `working-directory`,
  /// file `.mjs` trong lệnh `node`, và phần cố định của các `paths`.
  List<Finding> checkReferencedFiles() {
    final found = <Finding>[];
    bool exists(String rel) =>
        FileSystemEntity.typeSync('${repoRoot.path}/$rel') !=
        FileSystemEntityType.notFound;

    for (final (job, step) in steps) {
      final wd = step['working-directory'] as String?;
      if (wd != null && !exists(wd)) {
        found.add(
          Finding('missing-file', 'job $job: working-directory "$wd" không có'),
        );
      }
      final run = step['run'] as String?;
      if (run == null) continue;
      for (final m in RegExp(r'\bnode\s+(\S+\.mjs)\b').allMatches(run)) {
        final script = m.group(1)!;
        if (!exists(script)) {
          found.add(
            Finding('missing-file', 'job $job: script "$script" không có'),
          );
        }
      }
    }
    for (final event in ['push', 'pull_request']) {
      for (final glob in _pathsOf(event)) {
        // Phần đứng trước ký tự đại diện đầu tiên phải tồn tại.
        final fixed = glob.split(RegExp(r'[*?\[]')).first;
        final dir = fixed.endsWith('/')
            ? fixed.substring(0, fixed.length - 1)
            : fixed;
        if (!exists(dir)) {
          found.add(Finding('missing-file', 'paths "$glob": "$dir" không có'));
        }
      }
    }
    return found;
  }

  List<String> _pathsOf(String event) {
    final block = triggers[event];
    if (block is! YamlMap) return const [];
    final paths = block['paths'];
    return paths is YamlList ? [for (final p in paths) '$p'] : const [];
  }

  /// Bộ lọc `paths` của push và pull_request phải bắt được chính workflow và
  /// mọi file trong [labDirs] (thư mục lab, tính từ gốc repository).
  List<Finding> checkPathsTrigger(List<String> labDirs) {
    final found = <Finding>[];
    for (final event in ['push', 'pull_request']) {
      final globs = _pathsOf(event);
      if (globs.isEmpty) {
        found.add(Finding('paths', '$event không có bộ lọc paths'));
        continue;
      }
      final probes = [
        workflowRelativePath,
        for (final d in labDirs) '$d/pubspec.yaml',
      ];
      for (final probe in probes) {
        if (!pathsFilterMatches(globs, probe)) {
          found.add(Finding('paths', '$event không bắt "$probe"'));
        }
      }
    }
    return found;
  }

  /// Vệ sinh vận hành: mọi job có `timeout-minutes`, workflow có `concurrency`
  /// với `cancel-in-progress`, và `push` chỉ chạy cho nhánh liệt kê (nếu
  /// không, mỗi push vào nhánh có PR sinh hai run: push và pull_request).
  List<Finding> checkHygiene() {
    final found = <Finding>[];
    for (final entry in jobs.entries) {
      if ((entry.value as YamlMap)['timeout-minutes'] == null) {
        found.add(Finding('timeout', 'job ${entry.key} thiếu timeout-minutes'));
      }
    }
    final push = triggers['push'];
    if (push is! YamlMap || push['branches'] == null) {
      found.add(const Finding('push-branches', 'push không giới hạn branches'));
    }
    final concurrency = _doc['concurrency'];
    if (concurrency is! YamlMap ||
        concurrency['group'] == null ||
        concurrency['cancel-in-progress'] == null) {
      found.add(
        const Finding(
          'concurrency',
          'thiếu concurrency.group hoặc cancel-in-progress',
        ),
      );
    }
    return found;
  }

  /// Cache pub: lab không commit `pubspec.lock`, nên khoá chỉ băm lockfile sẽ
  /// không đổi khi dependency đổi. Khoá riêng phải băm `pubspec.yaml`.
  List<Finding> checkPubCacheKey() {
    final found = <Finding>[];
    for (final step in usesOf('subosito/flutter-action')) {
      final with_ = step['with'] as YamlMap? ?? YamlMap();
      if ('${with_['cache']}' != 'true') continue;
      final key = '${with_['pub-cache-key'] ?? ''}';
      if (!key.contains('hashFiles(') || !key.contains('pubspec.yaml')) {
        found.add(
          const Finding(
            'pub-cache-key',
            'pub-cache-key không băm pubspec.yaml',
          ),
        );
      }
    }
    return found;
  }

  /// Gộp mọi kiểm tra.
  List<Finding> all({required List<String> labDirs}) => [
    ...checkFlutterPin(),
    ...checkActionRefs(),
    ...checkPermissions(),
    ...checkNoSecrets(),
    ...checkReferencedFiles(),
    ...checkPathsTrigger(labDirs),
    ...checkHygiene(),
    ...checkPubCacheKey(),
  ];
}
