// Test với socket thật ở localhost (loopback): các kiểu lỗi mạng mà adapter
// thật của dio ném ra, để chắc phân loại và retry đúng thứ có ngoài đời chứ
// không chỉ thứ ta giả lập. Không ra khỏi máy.
import 'dart:async';
import 'dart:io';

import 'package:dio/dio.dart';
import 'package:test/test.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Server TCP thô ở loopback: mỗi kết nối đếm một lần rồi giao cho [onConnect].
final class _RawServer {
  _RawServer._(this._server);

  static Future<_RawServer> start(
    FutureOr<void> Function(Socket socket) onConnect,
  ) async {
    final server = await ServerSocket.bind(InternetAddress.loopbackIPv4, 0);
    final raw = _RawServer._(server);
    server.listen((socket) {
      raw.connections++;
      onConnect(socket);
    });
    return raw;
  }

  final ServerSocket _server;
  int connections = 0;

  int get port => _server.port;

  Future<void> close() => _server.close();
}

/// Số lần RetryInterceptor quyết định thử lại, đếm ở tầng dio. Không đếm kết
/// nối TCP: bản thân `HttpClient` của Dart đôi khi tự mở thêm một kết nối khi
/// bắt tay TLS hỏng (đã gặp ở lần chạy thật), nên số kết nối không ổn định.
final List<int> retries = [];

/// Client trỏ vào [port] bằng [scheme], gắn retry không chờ thật.
ToiNoiApiClient _client(String scheme, int port) {
  final client = ToiNoiApiClient(
    baseUrl: Uri.parse('$scheme://127.0.0.1:$port'),
    connectTimeout: const Duration(seconds: 2),
    receiveTimeout: const Duration(seconds: 2),
  );
  client.dio.interceptors.add(
    RetryInterceptor(
      dio: client.dio,
      sleep: (_) async {},
      onRetry: (event) => retries.add(event.attempt),
    ),
  );
  return client;
}

/// Đọc hết request rồi trả đầu response với content-length 100 nhưng chỉ
/// gửi 3 byte, rồi đóng: kết nối đứt giữa body.
Future<void> _cutMidBody(Socket socket) async {
  await socket.first;
  socket.write(
    'HTTP/1.1 200 OK\r\ncontent-length: 100\r\ncontent-type: application/json'
    '\r\n\r\nabc',
  );
  await socket.flush();
  socket.destroy();
}

void main() {
  late _RawServer server;
  setUp(retries.clear);
  tearDown(() => server.close());

  test(
    'server đóng kết nối trước khi có header: NetworkFailure, thử lại 2 lần',
    () async {
      server = await _RawServer.start((socket) => socket.destroy());
      final client = _client('http', server.port);
      await expectLater(
        guardApi(() => client.getTrip('T-1')),
        throwsA(
          isA<NetworkFailure>().having(
            (f) => f.reason,
            'reason',
            NetworkReason.unreachable,
          ),
        ),
      );
      expect(retries, [2, 3]);
    },
  );

  test('kết nối đứt giữa body: dio để ở unknown, ta vẫn nhận NetworkFailure và thử lại 2 lần', () async {
    server = await _RawServer.start(_cutMidBody);
    final client = _client('http', server.port);
    // Điều kiện đầu: đây đúng là kiểu lỗi dio không gán connectionError.
    final raw = ToiNoiApiClient(
      baseUrl: Uri.parse('http://127.0.0.1:${server.port}'),
    );
    await expectLater(
      raw.getTrip('T-1'),
      throwsA(
        isA<DioException>()
            .having((e) => e.type, 'type', DioExceptionType.unknown)
            .having((e) => e.error, 'error', isA<HttpException>()),
      ),
    );
    await expectLater(
      guardApi(() => client.getTrip('T-1')),
      throwsA(isA<NetworkFailure>()),
    );
    expect(retries, [2, 3]);
  });

  test(
    'POST không khoá, đứt giữa body: không gửi lại, đúng một kết nối',
    () async {
      server = await _RawServer.start(_cutMidBody);
      final client = _client('http', server.port);
      await expectLater(
        guardApi(
          () => client.dio.post<Object?>('/orders', data: const {'a': 1}),
        ),
        throwsA(isA<DioException>()),
      );
      expect(retries, isEmpty);
    },
  );

  test(
    'POST có Idempotency-Key, đứt giữa body: thử lại với cùng khoá',
    () async {
      final keys = <String>[];
      server = await _RawServer.start((socket) async {
        final request = String.fromCharCodes(await socket.first);
        final match = RegExp(
          r'idempotency-key: (\S+)',
          caseSensitive: false,
        ).firstMatch(request);
        keys.add(match?[1] ?? '');
        socket.write('HTTP/1.1 201 Created\r\ncontent-length: 100\r\n\r\nabc');
        await socket.flush();
        socket.destroy();
      });
      final client = _client('http', server.port);
      await expectLater(
        guardApi(
          () => client.createTrip(
            const QuoteRequestDto(serviceKind: ServiceKind.ride, distanceKm: 5),
            idempotencyKey: 'book-1',
          ),
        ),
        throwsA(isA<NetworkFailure>()),
      );
      expect(retries, [2, 3]);
      expect(keys.toSet(), {'book-1'});
      expect(keys.length, greaterThanOrEqualTo(3));
    },
  );

  test('HTTPS tới server không nói TLS: HandshakeException, badCertificate, không thử lại', () async {
    // Phải đọc ClientHello trước khi đóng: đóng khi còn dữ liệu chưa đọc làm
    // hệ điều hành gửi RST, và client lúc đó thấy lỗi reset chứ không phải bắt tay.
    server = await _RawServer.start((socket) async {
      await socket.first;
      socket.write('HTTP/1.1 400 Bad Request\r\n\r\n');
      await socket.flush();
      socket.destroy();
    });
    final client = _client('https', server.port);
    await expectLater(
      guardApi(() => client.getTrip('T-1')),
      throwsA(
        isA<NetworkFailure>()
            .having((f) => f.reason, 'reason', NetworkReason.badCertificate)
            .having((f) => f.retryable, 'retryable', isFalse)
            .having(
              (f) => (f.cause! as DioException).error,
              'cause',
              isA<HandshakeException>(),
            ),
      ),
    );
    expect(retries, isEmpty);
  });

  test(
    'cổng không có ai nghe: connectionError từ SocketException, thử lại 2 lần',
    () async {
      final probe = await ServerSocket.bind(InternetAddress.loopbackIPv4, 0);
      final port = probe.port;
      await probe.close(); // trả cổng lại: không còn ai nghe
      server = await _RawServer.start((socket) => socket.destroy());
      final client = _client('http', port);
      await expectLater(
        guardApi(() => client.getTrip('T-1')),
        throwsA(isA<NetworkFailure>()),
      );
    },
  );

  test(
    'server nhận request mà không bao giờ trả lời: TimeoutFailure(receive)',
    () async {
      server = await _RawServer.start((socket) async {
        await socket.first; // đọc rồi im lặng
      });
      final client = ToiNoiApiClient(
        baseUrl: Uri.parse('http://127.0.0.1:${server.port}'),
        receiveTimeout: const Duration(milliseconds: 300),
      );
      await expectLater(
        guardApi(() => client.getTrip('T-1')),
        throwsA(
          isA<TimeoutFailure>().having(
            (f) => f.phase,
            'phase',
            TimeoutPhase.receive,
          ),
        ),
      );
    },
  );
}
