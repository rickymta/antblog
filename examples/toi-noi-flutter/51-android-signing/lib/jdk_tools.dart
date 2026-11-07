import 'dart:io';
import 'dart:math';

/// Tên biến môi trường chứa mật khẩu khi gọi `keytool` và `jarsigner`, để mật
/// khẩu không nằm trong danh sách tham số của tiến trình.
const String _passwordEnv = 'TOI_NOI_KEYSTORE_PASSWORD';

/// Kết quả một lần chạy công cụ JDK.
class ToolResult {
  const ToolResult(this.exitCode, this.stdout, this.stderr);

  final int exitCode;
  final String stdout;
  final String stderr;

  String get output => '$stdout\n$stderr';
}

Future<ToolResult> _run(
  String tool,
  List<String> args, {
  String? password,
}) async {
  // -J-Duser.language=en: chữ cái cố định ("jar verified.") dù máy dùng
  // ngôn ngữ nào.
  final result = await Process.run(
    tool,
    ['-J-Duser.language=en', ...args],
    environment: {_passwordEnv: ?password},
  );
  return ToolResult(
    result.exitCode,
    result.stdout as String,
    result.stderr as String,
  );
}

/// Có đủ `keytool`, `jarsigner`, `jar` chạy được hay không (cần JDK).
///
/// Trên macOS, `/usr/bin/keytool` có thể chỉ là bản giữ chỗ khi chưa cài JDK
/// và thoát với mã khác 0, nên kiểm bằng cách chạy thật `-help`.
bool get jdkToolsAvailable {
  for (final (tool, flag) in const [
    ('keytool', '-help'),
    ('jarsigner', '-help'),
    ('jar', '--help'),
  ]) {
    try {
      if (Process.runSync(tool, [flag]).exitCode != 0) return false;
    } on ProcessException {
      return false;
    }
  }
  return true;
}

/// Mật khẩu ngẫu nhiên cho keystore TEST; không bao giờ là hằng trong repo.
String randomTestPassword([int length = 24]) {
  const alphabet =
      'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
  final random = Random.secure();
  return List.generate(
    length,
    (_) => alphabet[random.nextInt(alphabet.length)],
  ).join();
}

/// Thông tin chứng chỉ đọc từ `keytool -list -v`.
class CertInfo {
  const CertInfo({
    required this.owner,
    required this.sha256,
    required this.keySizeBits,
    required this.validUntilYear,
  });

  factory CertInfo.parse(String keytoolOutput) {
    String? field(RegExp pattern) =>
        pattern.firstMatch(keytoolOutput)?.group(1);
    final owner = field(RegExp(r'^Owner: (.+)$', multiLine: true));
    final sha256 = field(RegExp(r'SHA256: ((?:[0-9A-F]{2}:){31}[0-9A-F]{2})'));
    final bits = field(RegExp(r'Subject Public Key Algorithm: (\d+)-bit RSA'));
    final year = field(RegExp(r'until: .*?(\d{4})\s*$', multiLine: true));
    if (owner == null || sha256 == null || bits == null || year == null) {
      throw const FormatException('Không đọc được chứng chỉ từ đầu ra keytool');
    }
    return CertInfo(
      owner: owner,
      sha256: sha256,
      keySizeBits: int.parse(bits),
      validUntilYear: int.parse(year),
    );
  }

  final String owner;

  /// Vân tay SHA-256, dạng `AA:BB:...` (32 byte).
  final String sha256;

  final int keySizeBits;

  /// Năm hết hạn của chứng chỉ.
  final int validUntilYear;
}

/// Upload keystore TEST do `keytool` sinh ra trong một thư mục tạm.
class TestKeystore {
  TestKeystore._(this.file, this.alias, this.password);

  /// Sinh keystore theo đúng lệnh trong tài liệu Flutter: RSA 2048, hiệu lực
  /// 10000 ngày, `-storetype JKS`; mật khẩu truyền qua biến môi trường.
  static Future<TestKeystore> generate({
    required Directory directory,
    required String password,
    String fileName = 'upload-keystore.jks',
    String alias = 'upload',
    String distinguishedName = 'CN=Toi Noi Test Upload, O=Toi Noi Example',
    int validityDays = 10000,
  }) async {
    final file = File('${directory.path}/$fileName');
    final result = await _run('keytool', [
      '-genkeypair',
      '-keystore',
      file.path,
      '-storetype',
      'JKS',
      '-keyalg',
      'RSA',
      '-keysize',
      '2048',
      '-validity',
      '$validityDays',
      '-alias',
      alias,
      '-dname',
      distinguishedName,
      '-storepass:env',
      _passwordEnv,
      '-keypass:env',
      _passwordEnv,
    ], password: password);
    if (result.exitCode != 0) {
      throw ProcessException('keytool', ['-genkeypair'], result.output);
    }
    return TestKeystore._(file, alias, password);
  }

  final File file;
  final String alias;
  final String password;

  /// Đọc chứng chỉ (vân tay SHA-256, cỡ khoá, năm hết hạn).
  Future<CertInfo> inspect() async {
    final result = await _run('keytool', [
      '-list',
      '-v',
      '-keystore',
      file.path,
      '-alias',
      alias,
      '-storepass:env',
      _passwordEnv,
    ], password: password);
    if (result.exitCode != 0) {
      throw ProcessException('keytool', ['-list'], result.output);
    }
    return CertInfo.parse(result.stdout);
  }

  /// Xuất chứng chỉ công khai dạng PEM: thứ ta đưa cho Play Console khi xin
  /// reset upload key. Không chứa khoá riêng.
  Future<File> exportCertificatePem(File out) async {
    final result = await _run('keytool', [
      '-exportcert',
      '-rfc',
      '-keystore',
      file.path,
      '-alias',
      alias,
      '-file',
      out.path,
      '-storepass:env',
      _passwordEnv,
    ], password: password);
    if (result.exitCode != 0) {
      throw ProcessException('keytool', ['-exportcert'], result.output);
    }
    return out;
  }

  /// Ký [bundle] bằng `jarsigner`, cách Google Play nhận AAB.
  Future<void> signBundle(File bundle) async {
    final result = await _run('jarsigner', [
      '-keystore',
      file.path,
      '-storepass:env',
      _passwordEnv,
      '-digestalg',
      'SHA-256',
      '-sigalg',
      'SHA256withRSA',
      bundle.path,
      alias,
    ], password: password);
    if (result.exitCode != 0) {
      throw ProcessException('jarsigner', ['-sign'], result.output);
    }
  }
}

/// Dựng một "AAB giả": file zip có cấu trúc tối giản của app bundle, chỉ để
/// thử chữ ký. Không phải AAB thật và `bundletool` sẽ không nhận.
Future<File> createStandInBundle(Directory directory) async {
  final src = Directory('${directory.path}/bundle-src/base/manifest')
    ..createSync(recursive: true);
  File('${directory.path}/bundle-src/BundleConfig.pb').writeAsStringSync('cfg');
  File('${src.path}/AndroidManifest.xml').writeAsStringSync('<manifest/>');
  final out = File('${directory.path}/app-release.aab');
  final result = await _run('jar', [
    '--create',
    '--file',
    out.path,
    '--no-manifest',
    '-C',
    '${directory.path}/bundle-src',
    '.',
  ]);
  if (result.exitCode != 0) {
    throw ProcessException('jar', ['--create'], result.output);
  }
  return out;
}

/// Thay nội dung một file trong bundle (để thử phát hiện sửa sau khi ký).
Future<void> replaceEntry(File bundle, String entry, String content) async {
  final dir = Directory.systemTemp.createTempSync('tn51-replace');
  try {
    final target = File('${dir.path}/$entry')..createSync(recursive: true);
    target.writeAsStringSync(content);
    final result = await _run('jar', [
      '--update',
      '--file',
      bundle.path,
      '-C',
      dir.path,
      entry,
    ]);
    if (result.exitCode != 0) {
      throw ProcessException('jar', ['--update'], result.output);
    }
  } finally {
    dir.deleteSync(recursive: true);
  }
}

/// Trạng thái chữ ký của một bundle theo `jarsigner -verify`.
enum SignatureStatus { verified, unsigned, invalid }

/// Kết quả [verifyBundle].
class BundleVerification {
  const BundleVerification(this.status, this.output);

  final SignatureStatus status;
  final String output;
}

/// Chạy `jarsigner -verify` và đọc kết quả từ đầu ra, **không** từ mã thoát:
/// bundle chưa ký vẫn cho mã thoát 0 ("jar is unsigned.").
Future<BundleVerification> verifyBundle(File bundle) async {
  final result = await _run('jarsigner', ['-verify', bundle.path]);
  final out = result.output;
  if (result.exitCode != 0 || out.contains('SecurityException')) {
    return BundleVerification(SignatureStatus.invalid, out);
  }
  if (out.contains('jar is unsigned')) {
    return BundleVerification(SignatureStatus.unsigned, out);
  }
  if (out.contains('jar verified.')) {
    return BundleVerification(SignatureStatus.verified, out);
  }
  return BundleVerification(SignatureStatus.invalid, out);
}

/// Vân tay SHA-256 của chứng chỉ đã ký bundle: tương đương
/// `apksigner verify --print-certs` nhưng cho AAB.
Future<String> bundleSignerSha256(File bundle) async {
  final result = await _run('keytool', ['-printcert', '-jarfile', bundle.path]);
  if (result.exitCode != 0) {
    throw ProcessException('keytool', ['-printcert'], result.output);
  }
  return CertInfo.parse(result.stdout.replaceFirst('Signer #1:', '')).sha256;
}
