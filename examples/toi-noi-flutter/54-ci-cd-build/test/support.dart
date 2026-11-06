import 'dart:convert';
import 'dart:io';

import 'package:toi_noi_54/release_audit.dart';
import 'package:yaml/yaml.dart';

/// Thư mục lab (test chạy với thư mục hiện tại là lab).
final labDir = Directory.current.absolute;

/// Gốc repository, tìm bằng workflow thật của lab 33.
final repoRoot = findRepoRoot(labDir);

ReleaseWorkflow loadSample() =>
    ReleaseWorkflow.load(labDir, repoRoot: repoRoot);

/// Bản sao của workflow mẫu sau khi thay [from] bằng [to]. Ném lỗi nếu [from]
/// không có trong file, để ca sai không vô tình thành no-op.
ReleaseWorkflow mutated(String from, String to, {bool all = false}) {
  final original = loadSample().text;
  if (!original.contains(from)) {
    throw StateError('Workflow mẫu không chứa "$from"; sửa test.');
  }
  final text = all
      ? original.replaceAll(from, to)
      : original.replaceFirst(from, to);
  return ReleaseWorkflow(text, repoRoot: repoRoot);
}

/// Step của [job] có tên [name].
YamlMap stepNamed(ReleaseWorkflow wf, String job, String name) =>
    wf.stepsOf(job).singleWhere((s) => s['name'] == name);

/// `true` nếu máy có `bash` (các test chạy script thật cần nó).
final hasBash = () {
  try {
    return Process.runSync('bash', ['--version']).exitCode == 0;
  } on ProcessException {
    return false;
  }
}();

/// Kết quả chạy một script.
class RunResult {
  RunResult(this.exitCode, this.stdout, this.stderr);

  final int exitCode;
  final String stdout;
  final String stderr;

  /// Mọi thứ người đọc log của job sẽ thấy.
  String get log => '$stdout\n$stderr';
}

/// Chạy [script] bằng `bash --noprofile --norc -eo pipefail`. Mặc định của
/// GitHub cho `run` trên Linux và macOS là `bash -e {0}` (chỉ khi khai
/// `shell: bash` mới có `-eo pipefail`), nên đây chặt hơn thực tế: các script
/// của workflow tự đặt `set -euo pipefail` khi cần.
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

/// Giá trị giả cho mỗi biến mà step nhận từ secret. Ghép từ mảnh để không có
/// chuỗi nào trong repo trông như bí mật thật.
Map<String, String> fakeSecretsFor(YamlMap step) {
  final env = step['env'] as YamlMap;
  final out = <String, String>{};
  for (final key in env.keys.cast<String>()) {
    if (key.endsWith('_BASE64')) {
      final bytes = List<int>.generate(48, (i) => (i * 7 + key.length) % 256);
      out[key] = base64.encode(bytes);
    } else {
      out[key] = 'fake-${key.toLowerCase()}-${'x' * 6}';
    }
  }
  return out;
}

/// Môi trường tối thiểu để chạy script: PATH thật (cần `base64`, `openssl`),
/// thêm [extraPath] đứng trước, và thư mục tạm của "runner".
Map<String, String> baseEnv(Directory sandbox, {String? extraPath}) {
  final path = Platform.environment['PATH'] ?? '/usr/bin:/bin';
  return {
    'PATH': extraPath == null ? path : '$extraPath:$path',
    'HOME': '${sandbox.path}/home',
    'RUNNER_TEMP': '${sandbox.path}/runner_temp',
  };
}

/// Sandbox rỗng: `home/`, `runner_temp/`, `app/android/`, `bin/`.
Directory makeSandbox() {
  final dir = Directory.systemTemp.createTempSync('toi_noi_54_');
  for (final sub in ['home', 'runner_temp', 'app/android', 'bin']) {
    Directory('${dir.path}/$sub').createSync(recursive: true);
  }
  return dir;
}

/// Lệnh `security` giả: ghi mỗi lệnh vào `$STUB_LOG`, tạo file keychain khi
/// `create-keychain`, xoá khi `delete-keychain`, và thất bại ở lệnh con đặt
/// trong `$STUB_FAIL`.
void installSecurityStub(Directory sandbox) {
  final stub = File('${sandbox.path}/bin/security')
    ..writeAsStringSync(r'''#!/bin/bash
echo "$*" >> "$STUB_LOG"
sub="$1"
last="${@: -1}"
if [ "${STUB_FAIL:-}" = "$sub" ]; then exit 1; fi
case "$sub" in
  create-keychain) : > "$last" ;;
  delete-keychain) rm -f "$last" ;;
esac
''');
  Process.runSync('chmod', ['+x', stub.path]);
}
