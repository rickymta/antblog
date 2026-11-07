import 'dart:convert';
import 'dart:io';

import 'package:shelf/shelf.dart';
import 'package:shelf/shelf_io.dart' as shelf_io;
import 'package:toi_noi_36/realtime_client.dart';
import 'package:toi_noi_36/realtime_transport.dart';
import 'package:toi_noi_36/backoff.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

/// Fake API thật (shelf_io) nghe ở localhost, kèm Realtime Gateway giả.
final class GatewayHarness {
  GatewayHarness._(this.server, this.handler, this.realtime);

  final HttpServer server;
  final Handler handler;
  final RealtimeModule realtime;

  Uri get url => Uri.parse('ws://127.0.0.1:${server.port}/realtime');

  static Future<GatewayHarness> start({
    bool rejectBeforeUpgrade = false,
    Duration latency = Duration.zero,
  }) async {
    final auth = AuthModule();
    final realtime = RealtimeModule(
      verifyToken: auth.verifyAccessToken,
      rejectBeforeUpgrade: rejectBeforeUpgrade,
    );
    var tripSeq = 0;
    final handler = createFakeApiHandler(
      newId: () => 'T-${++tripSeq}',
      faults: FaultConfig(latency: latency),
      modules: [auth, realtime],
    );
    final server = await shelf_io.serve(
      handler,
      InternetAddress.loopbackIPv4,
      0,
    );
    return GatewayHarness._(server, handler, realtime);
  }

  Future<Map<String, Object?>> _post(
    String path,
    Map<String, Object?> body, {
    Map<String, String> headers = const {},
  }) async {
    final response = await handler(
      Request(
        'POST',
        Uri.parse('http://fake.test$path'),
        headers: {'content-type': 'application/json', ...headers},
        body: jsonEncode(body),
      ),
    );
    return jsonDecode(await response.readAsString()) as Map<String, Object?>;
  }

  /// Đăng nhập OTP giả, trả access token.
  Future<String> login({String app = 'customer'}) async {
    final challenge = await _post('/auth/otp/request', {
      'phone': '+84000000001',
      'app': app,
    });
    final session = await _post('/auth/otp/verify', {
      'requestId': challenge['requestId'],
      'code': fakeOtpCode,
    });
    return session['accessToken']! as String;
  }

  /// Đặt chuyến 5 km, trả mã chuyến.
  Future<String> createTrip() async {
    final trip = await _post(
      '/trips',
      {'serviceKind': 'ride', 'distanceKm': 5},
      headers: {
        'idempotency-key': 'k-${DateTime.now().microsecondsSinceEpoch}',
      },
    );
    return trip['id']! as String;
  }

  Future<void> close() => server.close(force: true);
}

/// Transport thật bọc lại để đếm số lần `connect`.
final class CountingTransport implements RealtimeTransport {
  CountingTransport([this.inner = const WebSocketTransport()]);

  final RealtimeTransport inner;
  int connects = 0;

  @override
  Future<RealtimeConnection> connect(Uri url, {required String token}) {
    connects++;
    return inner.connect(url, token: token);
  }
}

/// Backoff cực ngắn cho test dùng socket thật: không để test chờ lâu.
Backoff fastBackoff() => Backoff(
  base: const Duration(milliseconds: 5),
  cap: const Duration(milliseconds: 20),
);

/// Đợi tới khi [condition] đúng; thất bại sau 5 giây.
Future<void> eventually(bool Function() condition, {String? reason}) async {
  final stopwatch = Stopwatch()..start();
  while (!condition()) {
    if (stopwatch.elapsed > const Duration(seconds: 5)) {
      throw StateError(
        'Hết 5 giây mà điều kiện chưa đúng${reason == null ? '' : ': $reason'}',
      );
    }
    await Future<void>.delayed(const Duration(milliseconds: 5));
  }
}

/// Đợi trạng thái thoả [test] (đọc từ `state` rồi tới `states`).
Future<RealtimeState> waitForState(
  RealtimeClient client,
  bool Function(RealtimeState) test,
) async {
  if (test(client.state)) return client.state;
  return client.states.firstWhere(test).timeout(const Duration(seconds: 5));
}
