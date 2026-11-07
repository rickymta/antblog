import 'dart:io';

import 'package:toi_noi_33/workflow_audit.dart';
import 'package:yaml/yaml.dart';

export 'package:toi_noi_33/workflow_audit.dart'
    show Finding, findRepoRoot, workflowRelativePath;

/// Đường dẫn workflow mẫu, tính từ thư mục lab.
const sampleWorkflowPath =
    'native-config/.github/workflows/release-staging.yml';

/// Thư mục app khách, tính từ gốc repository (nơi các file cấu hình của
/// workflow trỏ tới: `config/staging.json`).
const customerAppPath = 'examples/toi-noi-flutter/toi-noi-app/apps/customer';

/// Đường dẫn nhắc tới các secret: chỉ cho phép trong `env` của bước `run`.
final _githubFiles = RegExp(
  r'^"?\$\{?(GITHUB_ENV|GITHUB_OUTPUT|GITHUB_PATH|GITHUB_STATE|GITHUB_STEP_SUMMARY)\}?"?$',
);

/// Ghép các dòng nối bằng `\` thành dòng lô-gic, bỏ dòng chú thích.
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
  if (buffer.isNotEmpty) out.add(buffer.toString().trim());
  return out;
}

/// Bộ kiểm tra workflow phát hành mẫu. Phần chung (ghim Flutter, ref action,
/// permissions, quét bí mật) dùng lại [WorkflowAudit] của lab 33.
class ReleaseWorkflow {
  ReleaseWorkflow(this.text, {required this.repoRoot})
    : base = WorkflowAudit(text, repoRoot: repoRoot);

  factory ReleaseWorkflow.load(
    Directory labDir, {
    required Directory repoRoot,
  }) => ReleaseWorkflow(
    File('${labDir.path}/$sampleWorkflowPath').readAsStringSync(),
    repoRoot: repoRoot,
  );

  final String text;
  final Directory repoRoot;
  final WorkflowAudit base;

  YamlMap get document => base.document;
  YamlMap get triggers => base.triggers;
  YamlMap get jobs => base.jobs;

  YamlMap job(String name) => jobs[name] as YamlMap;

  List<YamlMap> stepsOf(String jobName) => [
    for (final s in job(jobName)['steps'] as YamlList) s as YamlMap,
  ];

  /// Vị trí step của [jobName] thoả [test], hoặc -1.
  int indexOfStep(String jobName, bool Function(YamlMap step) test) {
    final steps = stepsOf(jobName);
    for (var i = 0; i < steps.length; i++) {
      if (test(steps[i])) return i;
    }
    return -1;
  }

  static String runOf(YamlMap step) => '${step['run'] ?? ''}';

  /// Tên biến môi trường nhận giá trị từ `secrets.*` và hiệu lực với [step]
  /// (env của workflow, của job và của chính step).
  Set<String> secretVarsOf(String jobName, YamlMap step) {
    final names = <String>{};
    void collect(Object? env) {
      if (env is! YamlMap) return;
      for (final e in env.entries) {
        if ('${e.value}'.contains('secrets.')) names.add('${e.key}');
      }
    }

    collect(document['env']);
    collect(job(jobName)['env']);
    collect(step['env']);
    return names;
  }

  // ---------------------------------------------------------------- trigger

  /// Chỉ chạy tay và theo tag: không `pull_request`, không push nhánh, không
  /// `workflow_run`/`schedule`. Môi trường có secret không nên mở cho mọi push.
  List<Finding> checkTriggers() {
    final found = <Finding>[];
    final extra = triggers.keys.toSet().difference({
      'workflow_dispatch',
      'push',
    });
    if (extra.isNotEmpty) {
      found.add(
        Finding('trigger', 'trigger ngoài workflow_dispatch/push: $extra'),
      );
    }
    if (!triggers.containsKey('workflow_dispatch')) {
      found.add(const Finding('trigger', 'thiếu workflow_dispatch'));
    }
    final push = triggers['push'];
    if (push is! YamlMap || push['tags'] == null) {
      found.add(const Finding('trigger', 'push phải lọc theo tags'));
    } else if (push.containsKey('branches') || push.containsKey('paths')) {
      found.add(const Finding('trigger', 'push không được có branches/paths'));
    }
    return found;
  }

  // ----------------------------------------------------------------- secret

  /// Secret chỉ nằm trong `env` của bước `run` (không ở env workflow/job, không
  /// trong `with` của action bên thứ ba), và job dùng secret phải có
  /// `environment` để reviewer duyệt.
  List<Finding> checkSecretScope() {
    final found = <Finding>[];
    bool hasSecret(Object? v) => '$v'.contains('secrets.');

    if (hasSecret(document['env'])) {
      found.add(const Finding('secret-scope', 'secret ở env cấp workflow'));
    }
    for (final entry in jobs.entries) {
      final name = entry.key as String;
      final job = entry.value as YamlMap;
      var usesSecret = false;
      for (final key in job.keys) {
        if (key != 'steps' && hasSecret(job[key])) {
          found.add(
            Finding('secret-scope', 'job $name: secret ở khoá "$key" cấp job'),
          );
        }
      }
      for (final step in job['steps'] as YamlList) {
        step as YamlMap;
        for (final key in step.keys) {
          if (!hasSecret(step[key])) continue;
          usesSecret = true;
          final isRunEnv = key == 'env' && step['run'] != null;
          if (!isRunEnv) {
            found.add(
              Finding(
                'secret-scope',
                'job $name: secret ở khoá "$key" của step "${step['name'] ?? step['uses']}"',
              ),
            );
          }
        }
      }
      if (usesSecret && job['environment'] == null) {
        found.add(
          Finding(
            'environment',
            'job $name dùng secret nhưng không có environment',
          ),
        );
      }
    }
    return found;
  }

  /// Không có `${{ ... }}` trong script `run`: giá trị đi qua `env`, nên
  /// không bị nội suy vào mã shell (chèn lệnh) và không lộ trong log.
  List<Finding> checkNoExpressionInRun() => [
    for (final jobName in jobs.keys.cast<String>())
      for (final step in stepsOf(jobName))
        if (runOf(step).contains(r'${{'))
          Finding(
            'expression-in-run',
            'job $jobName: step "${step['name']}" nội suy \${{ }} trong run',
          ),
  ];

  /// Biến chứa secret không được in ra log: không `echo`/`printf`/`cat` ra
  /// stdout, không ghi vào file GitHub (`GITHUB_ENV`...), không bật `set -x`,
  /// không in cả môi trường, không `cat` file đã ghi secret.
  List<Finding> checkSecretLeaks() {
    final found = <Finding>[];
    final printers = RegExp(r'\b(echo|printf|cat|print)\b');
    final fileReaders = RegExp(r'\b(cat|head|tail|less|more|xxd|od|strings)\b');
    final sensitiveFiles = RegExp(
      r'key\.properties|\.p12|\.jks|\.mobileprovision|\$\{?(PROPS|KEYSTORE_PATH|CERT_PATH|PROFILE_PATH)\b',
    );
    final xtrace = RegExp(
      r'\bset\s+-[a-zA-Z]*x|\bset\s+-o\s+xtrace|\b(ba)?sh\s+-[a-zA-Z]*x',
    );
    final envDump = RegExp(
      r'^(env|printenv|export\s+-p|declare\s+-p|set)(\s*[|>].*)?$',
    );
    final redirect = RegExp(r'(\d?)>>?\s*(&?\S+)');

    for (final jobName in jobs.keys.cast<String>()) {
      for (final step in stepsOf(jobName)) {
        final run = runOf(step);
        if (run.isEmpty) continue;
        final label = 'job $jobName, step "${step['name']}"';
        final secrets = secretVarsOf(jobName, step);
        final shell = '${step['shell'] ?? ''}';
        if (secrets.isNotEmpty && xtrace.hasMatch('$run\n$shell')) {
          found.add(
            Finding('secret-trace', '$label: bật xtrace khi có secret'),
          );
        }
        for (final line in logicalLines(run)) {
          if (envDump.hasMatch(line)) {
            found.add(Finding('env-dump', '$label: "$line" in cả môi trường'));
          }
          if (fileReaders.hasMatch(line) && sensitiveFiles.hasMatch(line)) {
            found.add(
              Finding(
                'print-secret-file',
                '$label: "$line" in file chứa bí mật',
              ),
            );
          }
          if (!printers.hasMatch(line)) continue;
          for (final name in secrets) {
            final ref = RegExp('\\\$\\{?$name\\b');
            final at = ref.firstMatch(line);
            if (at == null) continue;
            final tail = line.substring(at.end);
            var toFile = false;
            for (final m in redirect.allMatches(tail)) {
              final fd = m.group(1)!;
              final target = m.group(2)!;
              if (_githubFiles.hasMatch(target)) {
                found.add(
                  Finding(
                    'secret-to-github-file',
                    '$label: ghi \$$name vào $target',
                  ),
                );
                toFile = true;
              } else if ((fd.isEmpty || fd == '1') &&
                  !target.startsWith('&') &&
                  !target.startsWith('/dev/std') &&
                  target != '/dev/tty') {
                toFile = true;
              }
            }
            final piped = tail.contains('|');
            if (!toFile && !piped) {
              found.add(
                Finding('secret-echo', '$label: in \$$name ra log: "$line"'),
              );
            }
          }
        }
      }
    }
    return found;
  }

  /// Mỗi biến lấy từ secret phải được chặn rỗng (`${VAR:?...}`) trước khi dùng:
  /// secret chưa đặt hay rỗng sẽ làm bước đỏ thay vì sinh file 0 byte.
  List<Finding> checkSecretGuards() {
    final found = <Finding>[];
    for (final jobName in jobs.keys.cast<String>()) {
      for (final step in stepsOf(jobName)) {
        final run = runOf(step);
        for (final name in secretVarsOf(jobName, step)) {
          if (!run.contains('\${$name:?')) {
            found.add(
              Finding(
                'secret-guard',
                'job $jobName, step "${step['name']}": \$$name không có kiểm rỗng',
              ),
            );
          }
        }
      }
    }
    return found;
  }

  // ---------------------------------------------------------------- cleanup

  static String? _firstMatch(String text, RegExp re) =>
      re.firstMatch(text)?.group(0);

  static bool _isAlways(Object? condition) {
    final raw = '${condition ?? ''}'
        .replaceAll(RegExp(r'^\s*\$\{\{\s*'), '')
        .replaceAll(RegExp(r'\s*\}\}\s*$'), '')
        .trim();
    return raw == 'always()';
  }

  /// Job iOS: keychain tạm và provisioning profile phải được dọn ở một bước
  /// `if: always()` đứng SAU bước build, nhắm đúng file đã tạo.
  ///
  /// Trên runner GitHub-hosted máy ảo bị huỷ sau job nên bước này là phòng
  /// thủ; trên self-hosted runner nó là bắt buộc.
  List<Finding> checkKeychainCleanup({String jobName = 'ios'}) {
    final found = <Finding>[];
    final steps = stepsOf(jobName);
    final create = indexOfStep(
      jobName,
      (s) => runOf(s).contains('security create-keychain'),
    );
    final build = indexOfStep(
      jobName,
      (s) => runOf(s).contains('flutter build ipa'),
    );
    final cleanup = steps.lastIndexWhere(
      (s) => runOf(s).contains('security delete-keychain'),
    );
    if (create < 0) {
      return [
        Finding('keychain-cleanup', 'job $jobName không tạo keychain tạm'),
      ];
    }
    if (build < 0) {
      found.add(Finding('keychain-cleanup', 'job $jobName không build ipa'));
    }
    if (cleanup < 0) {
      return [
        ...found,
        Finding(
          'keychain-cleanup',
          'job $jobName thiếu bước security delete-keychain',
        ),
      ];
    }
    final cleanupStep = steps[cleanup];
    if (!_isAlways(cleanupStep['if'])) {
      found.add(
        Finding(
          'keychain-cleanup',
          'bước dọn keychain không có if: always() (đang: ${cleanupStep['if']})',
        ),
      );
    }
    if (build >= 0 && cleanup < build) {
      found.add(
        const Finding('keychain-cleanup', 'bước dọn đứng trước bước build'),
      );
    }
    final keychain = RegExp(r'[\w.-]+\.keychain-db');
    final created = _firstMatch(runOf(steps[create]), keychain);
    final deleted = _firstMatch(runOf(cleanupStep), keychain);
    if (created == null || created != deleted) {
      found.add(
        Finding(
          'keychain-cleanup',
          'keychain tạo ($created) khác keychain xoá ($deleted)',
        ),
      );
    }
    final profile = _firstMatch(
      runOf(steps[create]),
      RegExp(r'[\w.-]+\.mobileprovision'),
    );
    final cleanupRun = runOf(cleanupStep);
    if (profile == null ||
        !cleanupRun.contains(profile) ||
        !cleanupRun.contains('rm ')) {
      found.add(
        Finding('keychain-cleanup', 'bước dọn không xoá profile ($profile)'),
      );
    }
    final profileDirs = RegExp(r'Library/[\w/]*Provisioning Profiles')
        .allMatches(runOf(steps[create]))
        .map((m) => m.group(0)!)
        .toSet();
    if (profileDirs.isEmpty) {
      found.add(
        const Finding(
          'keychain-cleanup',
          'bước cài không chép profile vào Library',
        ),
      );
    }
    for (final dir in profileDirs) {
      if (!cleanupRun.contains(dir)) {
        found.add(
          Finding('keychain-cleanup', 'bước dọn không xoá profile ở $dir'),
        );
      }
    }
    final p12 = _firstMatch(runOf(steps[create]), RegExp(r'[\w.-]+\.p12'));
    if (p12 == null || !cleanupRun.contains(p12)) {
      found.add(Finding('keychain-cleanup', 'bước dọn không xoá file $p12'));
    }
    return found;
  }

  /// Job Android: keystore và `key.properties` được xoá ở bước `if: always()`
  /// sau bước build.
  List<Finding> checkKeystoreCleanup({String jobName = 'android'}) {
    final found = <Finding>[];
    final steps = stepsOf(jobName);
    final build = indexOfStep(
      jobName,
      (s) => runOf(s).contains('flutter build appbundle'),
    );
    final cleanup = steps.lastIndexWhere(
      (s) => runOf(s).contains('rm ') && runOf(s).contains('key.properties'),
    );
    if (cleanup < 0) {
      return [
        Finding('keystore-cleanup', 'job $jobName không xoá key.properties'),
      ];
    }
    final run = runOf(steps[cleanup]);
    if (!_isAlways(steps[cleanup]['if'])) {
      found.add(
        const Finding(
          'keystore-cleanup',
          'bước xoá keystore không có if: always()',
        ),
      );
    }
    if (!run.contains('.jks')) {
      found.add(
        const Finding('keystore-cleanup', 'bước xoá không nhắc tới file .jks'),
      );
    }
    if (build >= 0 && cleanup < build) {
      found.add(
        const Finding('keystore-cleanup', 'bước xoá đứng trước bước build'),
      );
    }
    return found;
  }

  // ----------------------------------------------------------- build number

  /// Build number: một nguồn duy nhất (job `prepare`) từ `github.run_number`
  /// (+ offset từ biến repository), dùng chung cho cả hai nền tảng, truyền qua
  /// `--build-number`. Không dùng `run_attempt`, `run_id`, SHA hay giờ hệ thống.
  List<Finding> checkBuildNumber() {
    final found = <Finding>[];
    if (!jobs.containsKey('prepare')) {
      return [const Finding('build-number', 'thiếu job prepare')];
    }
    final prepare = job('prepare');
    final outputs = prepare['outputs'];
    if (outputs is! YamlMap ||
        !'${outputs['build_number']}'.contains(
          'steps.number.outputs.build_number',
        )) {
      found.add(
        const Finding(
          'build-number',
          'prepare không xuất output build_number từ step number',
        ),
      );
    }
    final numberStep = stepsOf('prepare')
        .where((s) => s['id'] == 'number')
        .toList();
    if (numberStep.length != 1) {
      found.add(
        const Finding('build-number', 'prepare cần đúng một step id: number'),
      );
    } else {
      final step = numberStep.single;
      final env = step['env'] as YamlMap? ?? YamlMap();
      if ('${env['RUN_NUMBER']}'.replaceAll(' ', '') !=
          r'${{github.run_number}}') {
        found.add(
          Finding(
            'build-number',
            'RUN_NUMBER không lấy từ github.run_number: ${env['RUN_NUMBER']}',
          ),
        );
      }
      final run = runOf(step);
      if (!run.contains('GITHUB_OUTPUT')) {
        found.add(
          const Finding('build-number', 'step number không ghi GITHUB_OUTPUT'),
        );
      }
      if (!run.contains('2100000000')) {
        found.add(
          const Finding(
            'build-number',
            'step number không chặn trần versionCode 2100000000',
          ),
        );
      }
    }
    final prepareText = '${prepare.toString()} ${numberStep.map(runOf).join()}';
    for (final banned in [
      'run_attempt',
      'run_id',
      'github.sha',
      r'date +',
      'RANDOM',
    ]) {
      if (prepareText.contains(banned) || text.contains(banned)) {
        found.add(
          Finding(
            'build-number',
            'nguồn build number không được chứa "$banned"',
          ),
        );
        break;
      }
    }
    for (final name in ['android', 'ios']) {
      final j = job(name);
      final needs = j['needs'];
      final needsList = needs is YamlList
          ? needs.map((e) => '$e').toList()
          : ['$needs'];
      if (!needsList.contains('prepare')) {
        found.add(Finding('build-number', 'job $name không needs: prepare'));
      }
      final env = j['env'] as YamlMap? ?? YamlMap();
      if ('${env['BUILD_NUMBER']}'.replaceAll(' ', '') !=
          r'${{needs.prepare.outputs.build_number}}') {
        found.add(
          Finding(
            'build-number',
            'job $name: BUILD_NUMBER không lấy từ prepare',
          ),
        );
      }
      var builds = 0;
      for (final step in stepsOf(name)) {
        for (final line in logicalLines(runOf(step))) {
          if (!line.contains('flutter build')) continue;
          builds++;
          if (!line.contains(r'--build-number="$BUILD_NUMBER"')) {
            found.add(
              Finding(
                'build-number',
                'job $name: flutter build thiếu --build-number="\$BUILD_NUMBER"',
              ),
            );
          }
          if (line.contains('--build-name')) {
            found.add(
              Finding(
                'build-number',
                'job $name: --build-name phải lấy từ pubspec, không hard-code',
              ),
            );
          }
        }
      }
      if (builds == 0) {
        found.add(
          Finding('build-number', 'job $name không có lệnh flutter build'),
        );
      }
    }
    return found;
  }

  // ----------------------------------------------------------- build / paths

  /// Lệnh build của từng nền tảng, runner và JDK.
  List<Finding> checkBuildCommands() {
    final found = <Finding>[];
    String runnerOf(String n) => '${job(n)['runs-on']}';
    if (!runnerOf('android').startsWith('ubuntu-')) {
      found.add(
        Finding(
          'runner',
          'android phải chạy trên ubuntu, đang ${runnerOf('android')}',
        ),
      );
    }
    if (!runnerOf('ios').startsWith('macos-')) {
      found.add(
        Finding('runner', 'ios phải chạy trên macOS, đang ${runnerOf('ios')}'),
      );
    }
    for (final n in ['prepare', 'android', 'ios']) {
      if (runnerOf(n).endsWith('-latest')) {
        found.add(
          Finding(
            'runner',
            'job $n dùng nhãn -latest (đổi image không báo trước)',
          ),
        );
      }
    }
    final javaIndex = indexOfStep(
      'android',
      (s) => '${s['uses']}'.startsWith('actions/setup-java@'),
    );
    final androidBuild = indexOfStep(
      'android',
      (s) => runOf(s).contains('flutter build appbundle'),
    );
    if (javaIndex < 0) {
      found.add(
        const Finding('java', 'job android không cài JDK bằng setup-java'),
      );
    } else {
      final w = stepsOf('android')[javaIndex]['with'] as YamlMap? ?? YamlMap();
      if ('${w['java-version']}' != '17' ||
          '${w['distribution']}' != 'temurin') {
        found.add(
          Finding(
            'java',
            'cần temurin 17, đang ${w['distribution']} ${w['java-version']}',
          ),
        );
      }
      if (androidBuild >= 0 && javaIndex > androidBuild) {
        found.add(const Finding('java', 'setup-java đứng sau bước build'));
      }
    }
    for (final (jobName, command) in [
      ('android', 'flutter build appbundle'),
      ('ios', 'flutter build ipa'),
    ]) {
      final lines = [
        for (final s in stepsOf(jobName))
          for (final l in logicalLines(runOf(s)))
            if (l.contains(command)) l,
      ];
      if (lines.length != 1) {
        found.add(
          Finding('build-command', 'job $jobName cần đúng một "$command"'),
        );
        continue;
      }
      final line = lines.single;
      if (!line.contains('--release')) {
        found.add(Finding('build-command', 'job $jobName: thiếu --release'));
      }
      if (!line.contains('--flavor staging')) {
        found.add(
          Finding('build-command', 'job $jobName: thiếu --flavor staging'),
        );
      }
      if (!line.contains('--dart-define-from-file=config/staging.json')) {
        found.add(
          Finding(
            'build-command',
            'job $jobName: thiếu --dart-define-from-file=config/staging.json',
          ),
        );
      }
      if (line.contains('--obfuscate') !=
          line.contains('--split-debug-info=')) {
        found.add(
          Finding(
            'build-command',
            'job $jobName: --obfuscate phải đi cùng --split-debug-info',
          ),
        );
      }
    }
    final ipaLine = [
      for (final s in stepsOf('ios'))
        for (final l in logicalLines(runOf(s)))
          if (l.contains('flutter build ipa')) l,
    ].firstOrNull;
    if (ipaLine != null &&
        !RegExp(r'--export-options-plist=\S+\.plist').hasMatch(ipaLine)) {
      found.add(
        const Finding('build-command', 'ipa thiếu --export-options-plist'),
      );
    }
    return found;
  }

  /// Đường dẫn trong workflow phải trỏ tới thứ có thật trong app: thư mục làm
  /// việc mặc định của mỗi job và file cấu hình flavor.
  List<Finding> checkAppPaths() {
    final found = <Finding>[];
    final appDir = Directory(
      '${repoRoot.path}/examples/toi-noi-flutter/toi-noi-app',
    );
    for (final n in ['android', 'ios']) {
      final defaults = job(n)['defaults'];
      final wd = defaults is YamlMap && defaults['run'] is YamlMap
          ? (defaults['run'] as YamlMap)['working-directory']
          : null;
      if (wd == null || !Directory('${appDir.path}/$wd').existsSync()) {
        found.add(
          Finding(
            'app-path',
            'job $n: working-directory "$wd" không có trong toi-noi-app',
          ),
        );
        continue;
      }
      final config = File('${appDir.path}/$wd/config/staging.json');
      if (!config.existsSync()) {
        found.add(Finding('app-path', 'thiếu $wd/config/staging.json'));
      } else if (!config.readAsStringSync().contains('"FLAVOR": "staging"')) {
        found.add(
          Finding(
            'app-path',
            '$wd/config/staging.json không khai FLAVOR staging',
          ),
        );
      }
    }
    return found;
  }

  // -------------------------------------------------------------- artifacts

  /// Artifact: AAB và IPA phải có, báo lỗi khi không có file, thời hạn giữ
  /// hữu hạn, và tên không trùng (upload-artifact không cho hai artifact cùng tên).
  List<Finding> checkArtifacts() {
    final found = <Finding>[];
    final names = <String>[];
    final paths = <String>[];
    for (final jobName in ['android', 'ios']) {
      for (final step in stepsOf(jobName)) {
        if (!'${step['uses']}'.startsWith('actions/upload-artifact@')) continue;
        final w = step['with'] as YamlMap;
        names.add('${w['name']}');
        paths.add('${w['path']}');
        if ('${w['if-no-files-found']}' != 'error') {
          found.add(
            Finding(
              'artifact',
              '${w['name']}: if-no-files-found phải là error',
            ),
          );
        }
        final days = int.tryParse('${w['retention-days']}');
        if (days == null || days < 1 || days > 30) {
          found.add(
            Finding('artifact', '${w['name']}: retention-days ngoài 1..30'),
          );
        }
      }
    }
    if (!paths.any((p) => p.endsWith('/*.aab'))) {
      found.add(const Finding('artifact', 'không upload *.aab'));
    }
    if (!paths.any((p) => p.endsWith('/*.ipa'))) {
      found.add(const Finding('artifact', 'không upload *.ipa'));
    }
    if (names.toSet().length != names.length) {
      found.add(const Finding('artifact', 'tên artifact trùng nhau'));
    }
    return found;
  }

  // ---------------------------------------------------------------- hygiene

  /// Timeout cho mọi job, `persist-credentials: false` cho checkout,
  /// `concurrency` không huỷ run đang build.
  List<Finding> checkHygiene() {
    final found = <Finding>[];
    for (final entry in jobs.entries) {
      if ((entry.value as YamlMap)['timeout-minutes'] == null) {
        found.add(Finding('timeout', 'job ${entry.key} thiếu timeout-minutes'));
      }
    }
    for (final jobName in jobs.keys.cast<String>()) {
      for (final step in stepsOf(jobName)) {
        if (!'${step['uses']}'.startsWith('actions/checkout@')) continue;
        final w = step['with'] as YamlMap? ?? YamlMap();
        if ('${w['persist-credentials']}' != 'false') {
          found.add(
            Finding(
              'checkout',
              'job $jobName: checkout thiếu persist-credentials: false',
            ),
          );
        }
      }
    }
    final concurrency = document['concurrency'];
    if (concurrency is! YamlMap || concurrency['group'] == null) {
      found.add(const Finding('concurrency', 'thiếu concurrency.group'));
    } else if ('${concurrency['cancel-in-progress']}' != 'false') {
      found.add(
        const Finding('concurrency', 'cancel-in-progress phải là false'),
      );
    }
    return found;
  }

  /// Quyền: chỉ `contents: read` ở gốc, không job nào nâng quyền, không có
  /// `id-token` (workflow này không dùng OIDC).
  List<Finding> checkLeastPrivilege() {
    final found = [...base.checkPermissions()];
    final top = document['permissions'];
    if (top is YamlMap && (top.length != 1 || top['contents'] != 'read')) {
      found.add(const Finding('permissions', 'gốc chỉ được có contents: read'));
    }
    for (final entry in jobs.entries) {
      if ((entry.value as YamlMap).containsKey('permissions')) {
        found.add(
          Finding('permissions', 'job ${entry.key} khai permissions riêng'),
        );
      }
    }
    if (text.contains('id-token')) {
      found.add(
        const Finding(
          'permissions',
          'có id-token nhưng workflow không dùng OIDC',
        ),
      );
    }
    return found;
  }

  /// Gộp mọi kiểm tra, cả của lab 33.
  List<Finding> all() => [
    ...base.checkFlutterPin(),
    ...base.checkActionRefs(),
    ...base.checkNoSecrets(),
    ...checkLeastPrivilege(),
    ...checkTriggers(),
    ...checkSecretScope(),
    ...checkNoExpressionInRun(),
    ...checkSecretLeaks(),
    ...checkSecretGuards(),
    ...checkKeychainCleanup(),
    ...checkKeystoreCleanup(),
    ...checkBuildNumber(),
    ...checkBuildCommands(),
    ...checkAppPaths(),
    ...checkArtifacts(),
    ...checkHygiene(),
  ];
}
