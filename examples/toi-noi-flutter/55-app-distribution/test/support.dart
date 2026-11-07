import 'dart:convert';
import 'dart:io';

import 'package:toi_noi_53/release_plan.dart' show AppKind;
import 'package:toi_noi_55/toi_noi_55.dart';
import 'package:toi_noi_core/toi_noi_core.dart' show calculateFare;
import 'package:yaml/yaml.dart';

/// Thư mục lab (test chạy với thư mục hiện tại là lab).
final Directory labDir = Directory.current.absolute;

File nativeConfigFile(String relativePath) =>
    File('${labDir.path}/native-config/$relativePath');

String readNativeConfig(String relativePath) =>
    nativeConfigFile(relativePath).readAsStringSync();

/// Cước của chuyến 2,001 km tính bằng `toi_noi_core`, không gõ tay.
final String fare2001 = calculateFare(2.001).toString();

/// Danh sách commit mẫu, mới nhất trước như `git log`. Có đủ các loại: tính
/// năng, lỗi, hiệu năng, commit nội bộ, commit của app tài xế, trùng lặp, thẻ
/// bỏ qua, merge và commit không theo Conventional Commits.
List<Commit> sampleCommits() => [
  const Commit(
    'feat(customer): thêm đếm ngược khi tìm tài xế (#212)',
    sha: 'a1b2c3d',
  ),
  Commit('fix(fare): cước 2,001 km hiển thị đúng $fare2001.', sha: 'b2c3d4e'),
  const Commit('chore: nâng flutter_lints', sha: 'c3d4e5f'),
  const Commit('feat(driver): nhận cuốc bằng một chạm', sha: 'd4e5f6a'),
  const Commit('perf(map): giảm số lần vẽ lại polyline', sha: 'e5f6a7b'),
  const Commit('ci: ghim action bằng SHA', sha: 'f6a7b8c'),
  const Commit('test(fare): thêm ca biên 100 km', sha: '0a1b2c3'),
  const Commit('Merge branch develop into feature/x', sha: '1b2c3d4'),
  const Commit(
    'fix(customer): sửa nút gọi lại bị ẩn sau bàn phím',
    sha: '2c3d4e5',
  ),
  const Commit(
    'Fix(Customer): Sửa nút gọi lại bị ẩn sau bàn phím',
    sha: '3d4e5f6',
  ),
  const Commit('feat: bật cờ thử nghiệm [skip-notes]', sha: '4e5f6a7'),
  const Commit('sửa linh tinh', sha: '5f6a7b8'),
  const Commit('refactor(core): tách Money', sha: '6a7b8c9'),
];

ReleaseNotes notesFor(
  Iterable<Commit> commits, {
  AppKind app = AppKind.customer,
  int maxLength = defaultMaxNotesLength,
}) => ReleaseNotes.fromCommits(
  commits,
  app: app,
  version: '1.4.0',
  build: 42,
  maxLength: maxLength,
);

/// Đọc một file của lab khác trong `examples/toi-noi-flutter` (đường dẫn tính
/// từ thư mục đó).
String siblingFile(String relativePath) =>
    File('${labDir.parent.path}/$relativePath').readAsStringSync();

const jobsSnippetPath =
    '.github/workflows/release-staging.distribute-jobs.example.yml';
const apkStepsPath = '.github/workflows/build-apk-steps.example.yml';

String get jobsSnippetText => readNativeConfig(jobsSnippetPath);

/// Workflow dựng bản staging của bài 54 (nơi đoạn dán được đặt vào).
String get buildWorkflowText => siblingFile(
  '54-ci-cd-build/native-config/.github/workflows/release-staging.yml',
);

YamlMap get buildWorkflow => loadYaml(buildWorkflowText) as YamlMap;

/// Hai job của đoạn dán, đọc như một mục `jobs:` riêng.
YamlMap get snippetJobs => loadYaml(jobsSnippetText) as YamlMap;

/// Workflow của bài 54 sau khi dán đoạn mẫu vào cuối: đoạn mẫu thụt hai dấu
/// cách và `jobs:` là khoá cuối của file, nên nối chuỗi là đủ.
String get mergedWorkflowText => '$buildWorkflowText\n$jobsSnippetText';

YamlMap get mergedWorkflow => loadYaml(mergedWorkflowText) as YamlMap;

YamlMap jobOf(String name) => snippetJobs[name] as YamlMap;

List<YamlMap> stepsOf(String job) => [
  for (final s in jobOf(job)['steps'] as YamlList) s as YamlMap,
];

YamlMap stepNamed(String job, String name) =>
    stepsOf(job).singleWhere((s) => s['name'] == name);

/// Ghép các dòng nối bằng `\` thành dòng lô-gic, bỏ dòng chú thích và dòng trống.
List<String> logicalLines(String script) {
  final out = <String>[];
  final buffer = StringBuffer();
  for (final raw in script.split('\n')) {
    final line = raw.trimRight();
    if (line.trimLeft().startsWith('#')) continue;
    if (line.endsWith(r'\')) {
      buffer.write('${line.substring(0, line.length - 1)} ');
      continue;
    }
    buffer.write(line);
    final joined = buffer.toString().trim();
    buffer.clear();
    if (joined.isNotEmpty) out.add(joined);
  }
  return out;
}

/// Kết quả chạy một script.
class RunResult {
  RunResult(this.exitCode, this.stdout, this.stderr);

  final int exitCode;
  final String stdout;
  final String stderr;

  /// Mọi thứ người đọc log của job sẽ thấy.
  String get log => '$stdout\n$stderr';
}

/// Chạy [script] bằng bash với cùng cờ GitHub dùng cho `run` mặc định:
/// `bash --noprofile --norc -eo pipefail`.
RunResult runScript(
  String script, {
  required Map<String, String> env,
  required Directory workingDirectory,
}) {
  final file = File('${workingDirectory.path}/.step.sh')
    ..writeAsStringSync(script);
  final result = Process.runSync(
    'bash',
    ['--noprofile', '--norc', '-eo', 'pipefail', file.path],
    workingDirectory: workingDirectory.path,
    environment: env,
    includeParentEnvironment: false,
    stdoutEncoding: utf8,
    stderrEncoding: utf8,
  );
  file.deleteSync();
  return RunResult(
    result.exitCode,
    result.stdout as String,
    result.stderr as String,
  );
}

/// `true` nếu máy có đủ `bash`, `git` và `jq` (các test chạy script cần chúng).
final bool canRunScripts = ['bash', 'git', 'jq'].every((tool) {
  try {
    return Process.runSync(tool, ['--version']).exitCode == 0;
  } on ProcessException {
    return false;
  }
});
