/// Kiểm tra tĩnh các file fastlane trong `native-config/`.
///
/// Không chạy Ruby và không gọi fastlane: chỉ đọc văn bản, bỏ dòng chú thích,
/// rồi đối chiếu với [release_plan.dart]. Hàm trả danh sách vấn đề; rỗng là đạt.
library;

import 'release_plan.dart';

const iosFastfile = 'ios/fastlane/Fastfile';
const iosAppfile = 'ios/fastlane/Appfile';
const iosMatchfile = 'ios/fastlane/Matchfile';
const androidFastfile = 'android/fastlane/Fastfile';
const iosGemfile = 'ios/Gemfile';
const androidGemfile = 'android/Gemfile';
const envExample = '.env.example';
const iosGitignore = 'ios/gitignore.fragment.example';
const androidGitignore = 'android/gitignore.fragment.example';

/// Một chỗ sai: [code] ngắn gọn để test so khớp, [file] là đường dẫn trong
/// `native-config/`.
final class Problem {
  const Problem(this.code, this.file, this.message);

  final String code;
  final String file;
  final String message;

  @override
  String toString() => '[$code] $file: $message';
}

/// Văn bản bỏ các dòng chú thích `#` (cả Ruby lẫn `.env`).
String codeOf(String text) => text
    .split('\n')
    .where((line) => !line.trimLeft().startsWith('#'))
    .join('\n');

/// Thân lane [name]: từ dòng `lane :name do` tới dòng `end` cùng độ thụt.
/// `null` nếu không có lane đó.
String? laneBody(String text, String name) {
  final lines = text.split('\n');
  final start = RegExp('^(\\s*)lane :$name do\\b');
  for (var i = 0; i < lines.length; i++) {
    final match = start.firstMatch(lines[i]);
    if (match == null) continue;
    final indent = match.group(1)!;
    for (var j = i + 1; j < lines.length; j++) {
      if (lines[j] == '${indent}end') {
        return codeOf(lines.sublist(i + 1, j).join('\n'));
      }
    }
  }
  return null;
}

/// Thân lời gọi `<action>(` tới dấu `)` đóng tương ứng, trong [text].
String? callBody(String text, String action) {
  final at = RegExp('\\b$action\\(').firstMatch(text);
  if (at == null) return null;
  var depth = 1;
  for (var i = at.end; i < text.length; i++) {
    final c = text[i];
    if (c == '(') depth++;
    if (c == ')' && --depth == 0) return text.substring(at.end, i);
  }
  return null;
}

/// Các lane khai báo trong một Fastfile.
Set<String> lanesIn(String fastfile) => {
  for (final m in RegExp(
    r'^\s*lane :(\w+) do\b',
    multiLine: true,
  ).allMatches(codeOf(fastfile)))
    m.group(1)!,
};

/// Bảng `"app/flavor" => "id"` của một Fastfile.
Map<String, String> appIdsIn(String fastfile) => {
  for (final m in RegExp(
    r'"(\w+/\w+)"\s*=>\s*"([^"]+)"',
  ).allMatches(codeOf(fastfile)))
    m.group(1)!: m.group(2)!,
};

/// Tên trong `STORE_FLAVORS = %w[...]`.
Set<String> storeFlavorsIn(String fastfile) {
  final m = RegExp(r'STORE_FLAVORS = %w\[([^\]]*)\]')
      .firstMatch(codeOf(fastfile));
  return m == null ? {} : m.group(1)!.trim().split(RegExp(r'\s+')).toSet();
}

/// Tên biến môi trường mà mã đọc: `env!("X")` và `ENV["X"]`.
Set<String> envReadIn(String text, {bool onlyBang = false}) => {
  for (final m in RegExp(
    onlyBang ? r'env!\("([A-Z0-9_]+)"\)' : r'(?:env!\(|ENV\[)"([A-Z0-9_]+)"',
  ).allMatches(codeOf(text)))
    m.group(1)!,
};

/// Biến khai báo trong `.env.example` với giá trị (có thể rỗng).
Map<String, String> envDeclared(String text) => {
  for (final line in codeOf(text).split('\n'))
    if (RegExp(r'^[A-Z0-9_]+=').hasMatch(line))
      line.substring(0, line.indexOf('=')): line.substring(
        line.indexOf('=') + 1,
      ),
};

final _secretPatterns = <String, RegExp>{
  'khoá riêng PEM': RegExp(r'-----BEGIN [A-Z ]*PRIVATE KEY-----'),
  'khoá Google API': RegExp(r'AIza[0-9A-Za-z_-]{20,}'),
  'token GitHub': RegExp(r'gh[pousr]_[A-Za-z0-9]{20,}'),
  'JSON service account': RegExp(r'"(?:private_key|client_email)"\s*:'),
  'chuỗi dài giống token': RegExp(r'''["'][A-Za-z0-9+/=_-]{40,}["']'''),
};

/// Tham số mà giá trị bắt buộc đến từ ENV.
const _secretParams = [
  'key_id',
  'issuer_id',
  'key_content',
  'json_key_data',
  'json_key_file',
  'json_key',
  'password',
  'git_basic_authorization',
];

/// Kiểm toàn bộ [files] (khoá là đường dẫn trong `native-config/`).
List<Problem> auditNativeConfig(Map<String, String> files) {
  final problems = <Problem>[];
  void add(String code, String file, String message) =>
      problems.add(Problem(code, file, message));

  String? get(String path) {
    final text = files[path];
    if (text == null) add('thiếu-file', path, 'không có file này');
    return text;
  }

  for (final path in [
    iosFastfile,
    iosAppfile,
    iosMatchfile,
    androidFastfile,
    iosGemfile,
    androidGemfile,
    iosGitignore,
    androidGitignore,
  ]) {
    get(path);
  }

  // 1. Không có chuỗi giống bí mật; .env.example chỉ có giá trị rỗng.
  for (final entry in files.entries) {
    final code = codeOf(entry.value);
    for (final p in _secretPatterns.entries) {
      if (p.value.hasMatch(code)) {
        add('bí-mật', entry.key, 'có ${p.key}');
      }
    }
  }
  final env = get(envExample);
  if (env != null) {
    for (final e in envDeclared(env).entries) {
      if (e.value.trim().isNotEmpty) {
        add('bí-mật', envExample, '${e.key} phải để trống');
      }
    }
  }

  // 2. Tham số bí mật chỉ nhận giá trị từ ENV, không bao giờ literal.
  const rubyFiles = [iosFastfile, iosAppfile, iosMatchfile, androidFastfile];
  final paramPattern = RegExp(
    '\\b(${_secretParams.join('|')})\\s*:\\s*([^\\s,)]+)',
  );
  for (final path in rubyFiles) {
    final text = files[path];
    if (text == null) continue;
    for (final m in paramPattern.allMatches(codeOf(text))) {
      final value = m.group(2)!;
      if (!value.startsWith('env!(') && !value.startsWith('ENV[')) {
        add(
          'bí-mật-literal',
          path,
          '${m.group(1)} nhận "$value", không phải ENV',
        );
      }
    }
  }

  final matchfileText = files[iosMatchfile];
  if (matchfileText != null) {
    for (final m in RegExp(
      r'(git_basic_authorization|git_private_key)\(\s*([^)\s]*)',
    ).allMatches(codeOf(matchfileText))) {
      if (!m.group(2)!.startsWith('ENV[')) {
        add(
          'bí-mật-literal',
          iosMatchfile,
          '${m.group(1)} nhận "${m.group(2)}", không phải ENV',
        );
      }
    }
  }

  // 3. Biến ENV mã đọc khớp đúng .env.example, và khớp biến lane cần.
  final read = <String>{};
  for (final path in rubyFiles) {
    final text = files[path];
    if (text != null) read.addAll(envReadIn(text));
  }
  if (env != null) {
    final declared = envDeclared(env).keys.toSet();
    for (final name in read.difference(declared)) {
      add('env-chưa-khai', envExample, '$name được đọc nhưng chưa khai báo');
    }
    for (final name in declared.difference(read)) {
      add('env-thừa', envExample, '$name được khai báo nhưng không ai đọc');
    }
  }
  for (final (platform, path) in [
    (StorePlatform.ios, iosFastfile),
    (StorePlatform.android, androidFastfile),
  ]) {
    final text = files[path];
    if (text == null) continue;
    final bang = envReadIn(text, onlyBang: true);
    if (bang.length != requiredEnv(platform).length ||
        !bang.containsAll(requiredEnv(platform))) {
      add(
        'env-lane',
        path,
        'env! đọc $bang, release_plan yêu cầu ${requiredEnv(platform)}',
      );
    }
  }

  // 4. Bundle ID và application ID khớp curriculum §3.2.
  final expectedIds = {
    for (final app in AppKind.values)
      for (final flavor in Flavor.values)
        '${app.name}/${flavor.name}': applicationId(app, flavor),
  };
  for (final path in [iosFastfile, androidFastfile]) {
    final text = files[path];
    if (text == null) continue;
    final actual = appIdsIn(text);
    for (final e in expectedIds.entries) {
      if (actual[e.key] != e.value) {
        add('bundle-id', path, '${e.key}: có ${actual[e.key]}, cần ${e.value}');
      }
    }
    if (actual.length != expectedIds.length) {
      add(
        'bundle-id',
        path,
        'có ${actual.length} dòng, cần ${expectedIds.length}',
      );
    }
  }

  // 5. Chỉ flavor có lane trong release_plan mới được phân phối; dev thì không.
  final storeFlavors = {
    for (final f in Flavor.values)
      if (lanesFor(f).isNotEmpty) f.name,
  };
  for (final path in [iosFastfile, androidFastfile]) {
    final text = files[path];
    if (text == null) continue;
    if (!storeFlavors.containsAll(storeFlavorsIn(text)) ||
        storeFlavorsIn(text).length != storeFlavors.length) {
      add(
        'flavor',
        path,
        'STORE_FLAVORS ${storeFlavorsIn(text)} khác $storeFlavors',
      );
    }
  }

  // 6. Mỗi lane của release_plan có mặt đúng Fastfile.
  final ios = files[iosFastfile];
  final android = files[androidFastfile];
  if (ios != null) {
    final lanes = lanesIn(ios);
    for (final spec in [iosBeta, iosSyncCerts]) {
      if (!lanes.contains(spec.lane)) {
        add('lane', iosFastfile, 'thiếu lane ${spec.lane}');
      }
    }
  }
  if (android != null && !lanesIn(android).contains(androidInternal.lane)) {
    add('lane', androidFastfile, 'thiếu lane ${androidInternal.lane}');
  }

  // 7. Android: track internal, AAB, không có đường vòng lên track khác.
  if (android != null) {
    final upload = callBody(codeOf(android), 'upload_to_play_store');
    if (upload == null) {
      add('android-track', androidFastfile, 'không có upload_to_play_store');
    } else {
      if (!upload.contains('track: "${androidInternal.track}"')) {
        add(
          'android-track',
          androidFastfile,
          'track phải là "${androidInternal.track}"',
        );
      }
      if (!upload.contains('aab:') || upload.contains('apk:')) {
        add('android-aab', androidFastfile, 'phải đẩy aab:, không đẩy apk:');
      }
    }
    final code = codeOf(android);
    for (final banned in [
      'track_promote_to',
      '"production"',
      '"beta"',
      '"alpha"',
    ]) {
      if (code.contains(banned)) {
        add('android-track', androidFastfile, 'không được có $banned');
      }
    }
    if (!code.contains('flutter build appbundle')) {
      add(
        'android-aab',
        androidFastfile,
        'phải build bằng flutter build appbundle',
      );
    }
  }

  // 8. iOS: match chỉ đọc, API key thay mật khẩu, lane quản trị không chạy trên CI.
  if (ios != null) {
    final beta = laneBody(ios, 'beta');
    if (beta == null) {
      add('lane', iosFastfile, 'thiếu lane beta');
    } else {
      final matchCall = callBody(beta, 'match');
      if (matchCall == null ||
          !matchCall.contains('readonly: true') ||
          !matchCall.contains('type: "appstore"') ||
          !matchCall.contains('api_key:')) {
        add(
          'ios-match',
          iosFastfile,
          'lane beta phải gọi match(type: "appstore", readonly: true, api_key: ...)',
        );
      }
      final gymCall = callBody(beta, 'gym');
      if (gymCall == null ||
          !gymCall.contains('skip_build_archive: true') ||
          !gymCall.contains('MATCH_PROVISIONING_PROFILE_MAPPING')) {
        add(
          'ios-gym',
          iosFastfile,
          'gym phải xuất từ archive với profile của match',
        );
      }
      if (gymCall == null || !gymCall.contains('scheme: flavor')) {
        add(
          'ios-scheme',
          iosFastfile,
          'gym phải truyền scheme: flavor, vì project có nhiều scheme (bài 50)',
        );
      }
      final signing = callBody(beta, 'update_code_signing_settings');
      final matchAt = beta.indexOf('match(');
      final signingAt = beta.indexOf('update_code_signing_settings(');
      final buildAt = beta.indexOf('flutter build ipa');
      if (signing == null ||
          !signing.contains('use_automatic_signing: false') ||
          !signing.contains('build_configurations: ["Release-#{flavor}"]') ||
          !signing.contains(
            'profile_name: lane_context[SharedValues::MATCH_PROVISIONING_PROFILE_MAPPING][bundle_id]',
          ) ||
          !(matchAt >= 0 && matchAt < signingAt && signingAt < buildAt)) {
        add(
          'ios-signing',
          iosFastfile,
          'phải ép Release-<flavor> ký bằng profile của match, sau match và trước flutter build ipa',
        );
      }
      final branch = beta.indexOf('if ipa.empty?');
      for (final inBranch in [
        'build_number!(options)',
        'env!("MATCH_PASSWORD")',
      ]) {
        final at = beta.indexOf(inBranch);
        if (branch < 0 || at < branch) {
          add(
            'ios-ipa-branch',
            iosFastfile,
            '$inBranch phải nằm trong nhánh build, vì ipa: có sẵn thì không cần',
          );
        }
      }
      final pilotCall = callBody(beta, 'upload_to_testflight');
      if (pilotCall == null || !pilotCall.contains('api_key:')) {
        add(
          'ios-pilot',
          iosFastfile,
          'upload_to_testflight phải truyền api_key',
        );
      }
      if (pilotCall != null &&
          pilotCall.contains('distribute_external: true')) {
        add(
          'ios-pilot',
          iosFastfile,
          'lane beta không tự phát cho tester ngoài',
        );
      }
      if (!beta.contains('--build-number') || !beta.contains('--flavor')) {
        add(
          'ios-build',
          iosFastfile,
          'flutter build ipa thiếu --flavor hoặc --build-number',
        );
      }
    }
    final sync = laneBody(ios, 'sync_certs');
    if (sync != null) {
      if (!RegExp(r'\bis_ci\b').hasMatch(sync)) {
        add(
          'ios-sync',
          iosFastfile,
          'sync_certs phải từ chối khi chạy trên CI',
        );
      }
      if (!sync.contains('readonly: false')) {
        add(
          'ios-sync',
          iosFastfile,
          'sync_certs là lane duy nhất được ghi vào kho match',
        );
      }
    }
    final writers = RegExp(r'readonly: false').allMatches(codeOf(ios)).length;
    if (writers != 1) {
      add(
        'ios-match',
        iosFastfile,
        'readonly: false xuất hiện $writers lần, cần đúng 1',
      );
    }
  }
  for (final path in [iosFastfile, iosAppfile, iosMatchfile]) {
    final text = files[path];
    if (text == null) continue;
    if (RegExp(r'apple_id|FASTLANE_PASSWORD|FASTLANE_USER|FASTLANE_SESSION')
        .hasMatch(codeOf(text))) {
      add('ios-mật-khẩu', path, 'đăng nhập bằng API key, không bằng Apple ID');
    }
  }
  final matchfile = files[iosMatchfile];
  if (matchfile != null) {
    final code = codeOf(matchfile);
    for (final need in [
      'readonly(true)',
      'type("appstore")',
      'storage_mode("git")',
      'git_url(ENV["MATCH_GIT_URL"])',
      'git_basic_authorization(ENV["MATCH_GIT_BASIC_AUTHORIZATION"])',
      'git_private_key(ENV["MATCH_GIT_PRIVATE_KEY"])',
    ]) {
      if (!code.contains(need)) add('matchfile', iosMatchfile, 'thiếu $need');
    }
  }

  // 9. Gemfile ghim fastlane theo minor.
  for (final path in [iosGemfile, androidGemfile]) {
    final text = files[path];
    if (text == null) continue;
    if (!RegExp(r'gem "fastlane", "~> \d+\.\d+\.\d+"').hasMatch(codeOf(text))) {
      add(
        'gemfile',
        path,
        'phải ghim gem "fastlane", "~> x.y.z" (chỉ nhận bản vá)',
      );
    }
  }

  // 10. File .env không vào repo: mẫu gitignore của hai thư mục có dòng `.env`.
  for (final path in [iosGitignore, androidGitignore]) {
    final text = files[path];
    if (text == null) continue;
    final lines = codeOf(text).split('\n').map((l) => l.trim()).toSet();
    if (!lines.contains('.env') || !lines.contains('.env.*')) {
      add('gitignore', path, 'phải có .env và .env.*');
    }
  }

  return problems;
}
