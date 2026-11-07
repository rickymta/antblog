import 'dart:io';

import 'test_certs.dart';

/// Server HTTPS trên localhost dùng chứng chỉ test, ghi lại từng request nó
/// thực sự nhận được. Dùng để chứng minh request có tới server hay không.
final class TlsTestServer {
  TlsTestServer._(this._server);

  final HttpServer _server;

  /// Giá trị header `Authorization` của từng request đã tới.
  final List<String?> receivedAuthorization = [];

  int get port => _server.port;

  int get requestCount => receivedAuthorization.length;

  Uri uri(String path) => Uri.parse('https://localhost:$port$path');

  static Future<TlsTestServer> start(TestCert cert) async {
    final server = await HttpServer.bindSecure(
      InternetAddress.loopbackIPv4,
      0,
      cert.serverContext(),
    );
    final result = TlsTestServer._(server);
    server.listen((request) {
      result.receivedAuthorization.add(
        request.headers.value(HttpHeaders.authorizationHeader),
      );
      request.response
        ..headers.contentType = ContentType.json
        ..write('{"status":"ok"}')
        ..close();
    });
    return result;
  }

  Future<void> close() => _server.close(force: true);
}
