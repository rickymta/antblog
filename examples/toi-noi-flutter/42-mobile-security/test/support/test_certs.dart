import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

/// Chứng chỉ chỉ dùng cho test, sinh lúc chạy bằng `openssl` vào thư mục tạm
/// và xoá sau khi xong. Khoá riêng của CA và của các chứng chỉ lá không phải
/// bí mật: chúng chỉ tồn tại trong một lần chạy test và chỉ dùng cho kết nối
/// tới localhost.
final class TestCert {
  TestCert._(
    this.certPem,
    this.keyPem,
    this.keyFile,
    this._rootPem,
    this._intermediatePem,
  );

  final String certPem;
  final String keyPem;
  final String keyFile;
  final String _rootPem;
  final String? _intermediatePem;

  /// Bytes DER của chứng chỉ lá (giải mã từ PEM).
  Uint8List get der => _pemToDer(certPem);

  /// Bytes DER của chứng chỉ trung gian, nếu lá được cấp qua trung gian.
  Uint8List? get intermediateDer {
    final pem = _intermediatePem;
    return pem == null ? null : _pemToDer(pem);
  }

  /// Server gửi lá, kèm chứng chỉ trung gian nếu có.
  SecurityContext serverContext() {
    final intermediate = _intermediatePem;
    return SecurityContext()
      ..useCertificateChainBytes(
        utf8.encode(intermediate == null ? certPem : certPem + intermediate),
      )
      ..usePrivateKeyBytes(utf8.encode(keyPem));
  }

  /// Context của client tin CA gốc test đã cấp chứng chỉ này (giả lập "một CA
  /// hợp lệ đã cấp"), không tin CA nào khác.
  SecurityContext trustingClientContext() =>
      SecurityContext(withTrustedRoots: false)
        ..setTrustedCertificatesBytes(utf8.encode(_rootPem));
}

/// Một CA gốc test (và một CA trung gian tạo khi cần) trong thư mục tạm.
final class TestPki {
  TestPki._(this._dir, this._rootPem);

  final Directory _dir;
  final String _rootPem;
  var _serial = 1000;
  String? _intermediatePem;

  static Future<TestPki> create() async {
    final dir = await Directory.systemTemp.createTemp('toi_noi_42_pki_');
    await _genKey('${dir.path}/root.key.pem');
    await _openssl([
      'req',
      '-x509',
      '-sha256',
      '-new',
      '-key',
      '${dir.path}/root.key.pem',
      '-out',
      '${dir.path}/root.cert.pem',
      '-days',
      '2',
      '-subj',
      '/CN=Toi Noi Test Root CA',
      '-addext',
      'basicConstraints=critical,CA:TRUE',
      '-addext',
      'keyUsage=critical,keyCertSign,cRLSign',
      '-addext',
      'subjectKeyIdentifier=hash',
    ]);
    return TestPki._(
      dir,
      await File('${dir.path}/root.cert.pem').readAsString(),
    );
  }

  /// Sinh khoá EC P-256 mới và chứng chỉ lá do CA test ký. [san] là danh sách
  /// SubjectAltName; [viaIntermediate] cấp qua một CA trung gian.
  Future<TestCert> issue(
    String name, {
    String san = 'DNS:localhost,IP:127.0.0.1',
    bool viaIntermediate = false,
  }) async {
    final keyFile = '${_dir.path}/$name.key.pem';
    await _genKey(keyFile);
    return reissue(name, keyFile, san: san, viaIntermediate: viaIntermediate);
  }

  /// Cấp lại chứng chỉ mới trên khoá cũ (giống gia hạn chứng chỉ).
  Future<TestCert> reissue(
    String name,
    String keyFile, {
    String san = 'DNS:localhost,IP:127.0.0.1',
    bool viaIntermediate = false,
  }) async {
    final signer = viaIntermediate ? await _ensureIntermediate() : 'root';
    final id = _serial++;
    final csr = '${_dir.path}/$name.$id.csr.pem';
    final ext = '${_dir.path}/$name.$id.ext';
    final certFile = '${_dir.path}/$name.$id.cert.pem';
    await File(ext).writeAsString(
      'subjectAltName=$san\n'
      'basicConstraints=CA:FALSE\n'
      'keyUsage=critical,digitalSignature\n'
      'extendedKeyUsage=serverAuth\n'
      'subjectKeyIdentifier=hash\n'
      'authorityKeyIdentifier=keyid\n',
    );
    await _openssl([
      'req',
      '-new',
      '-key',
      keyFile,
      '-out',
      csr,
      '-subj',
      '/CN=leaf',
    ]);
    await _sign(csr, signer, id, ext, certFile);
    return TestCert._(
      await File(certFile).readAsString(),
      await File(keyFile).readAsString(),
      keyFile,
      _rootPem,
      viaIntermediate ? _intermediatePem : null,
    );
  }

  Future<String> _ensureIntermediate() async {
    if (_intermediatePem == null) {
      final key = '${_dir.path}/intermediate.key.pem';
      final csr = '${_dir.path}/intermediate.csr.pem';
      final ext = '${_dir.path}/intermediate.ext';
      final cert = '${_dir.path}/intermediate.cert.pem';
      await _genKey(key);
      await File(ext).writeAsString(
        'basicConstraints=critical,CA:TRUE,pathlen:0\n'
        'keyUsage=critical,keyCertSign,cRLSign\n'
        'subjectKeyIdentifier=hash\n'
        'authorityKeyIdentifier=keyid\n',
      );
      await _openssl([
        'req',
        '-new',
        '-key',
        key,
        '-out',
        csr,
        '-subj',
        '/CN=Toi Noi Test Intermediate CA',
      ]);
      await _sign(csr, 'root', _serial++, ext, cert);
      _intermediatePem = await File(cert).readAsString();
    }
    return 'intermediate';
  }

  Future<void> _sign(
    String csr,
    String signer,
    int serial,
    String ext,
    String out,
  ) => _openssl([
    'x509',
    '-req',
    '-in',
    csr,
    '-CA',
    '${_dir.path}/$signer.cert.pem',
    '-CAkey',
    '${_dir.path}/$signer.key.pem',
    '-sha256',
    '-set_serial',
    '$serial',
    '-days',
    '2',
    '-extfile',
    ext,
    '-out',
    out,
  ]);

  /// SPKI hash tính bằng chính `openssl`, để đối chiếu với code của ta.
  Future<String> opensslSpkiHash(TestCert cert) async {
    final certFile = '${_dir.path}/check.cert.pem';
    final pubFile = '${_dir.path}/check.pub.pem';
    final derFile = '${_dir.path}/check.pub.der';
    await File(certFile).writeAsString(cert.certPem);
    final pubkey = await _openssl([
      'x509',
      '-in',
      certFile,
      '-pubkey',
      '-noout',
    ]);
    await File(pubFile).writeAsString(pubkey);
    await _openssl([
      'pkey',
      '-pubin',
      '-in',
      pubFile,
      '-outform',
      'der',
      '-out',
      derFile,
    ]);
    final digest = await Process.run('openssl', [
      'dgst',
      '-sha256',
      '-binary',
      derFile,
    ], stdoutEncoding: null);
    return base64.encode(digest.stdout as List<int>);
  }

  Future<void> dispose() => _dir.delete(recursive: true);

  static Future<void> _genKey(String file) => _openssl([
    'ecparam',
    '-name',
    'prime256v1',
    '-genkey',
    '-noout',
    '-out',
    file,
  ]);

  static Future<String> _openssl(List<String> args) async {
    final result = await Process.run('openssl', args);
    if (result.exitCode != 0) {
      throw StateError('openssl ${args.first} lỗi: ${result.stderr}');
    }
    return result.stdout as String;
  }
}

Uint8List _pemToDer(String pem) {
  final body = pem
      .split('\n')
      .where((line) => line.isNotEmpty && !line.startsWith('-----'))
      .join();
  return base64.decode(body);
}
