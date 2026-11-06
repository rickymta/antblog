import 'package:shelf/shelf.dart';
import 'package:shelf_router/shelf_router.dart';
import 'package:test/test.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'support.dart';

Future<Reply> _quote(Handler handler) => call(
  handler,
  'POST',
  '/quotes',
  body: {'serviceKind': 'ride', 'distanceKm': 5},
);

final class _PingModule implements FakeApiModule {
  const _PingModule();

  @override
  void mount(Router router, FakeApiContext context) {
    router.get('/ping', (Request request) => jsonResponse(200, {'pong': true}));
  }
}

void main() {
  group('FaultConfig', () {
    test('failFirst: N request đầu trả 500, sau đó bình thường', () async {
      final handler = createFakeApiHandler(
        faults: const FaultConfig(failFirst: 2),
      );
      expect((await _quote(handler)).status, 500);
      expect((await _quote(handler)).status, 500);
      expect((await _quote(handler)).status, 200);
      expect((await _quote(handler)).status, 200);
    });

    test('failEvery: cứ N request thì một request lỗi', () async {
      final handler = createFakeApiHandler(
        faults: const FaultConfig(failEvery: 3),
      );
      final statuses = [
        for (var i = 0; i < 7; i++) (await _quote(handler)).status,
      ];
      expect(statuses, [200, 200, 500, 200, 200, 500, 200]);
    });

    test('lỗi 500 là problem+json, có traceId', () async {
      final handler = createFakeApiHandler(
        faults: const FaultConfig(failFirst: 1),
      );
      final reply = await _quote(handler);
      expect(asMap(reply)['title'], 'Unexpected server error');
      expect(asMap(reply)['traceId'], isA<String>());
    });

    test('/health được miễn và không làm tiêu bộ đếm', () async {
      final handler = createFakeApiHandler(
        faults: const FaultConfig(failFirst: 1),
      );
      expect((await call(handler, 'GET', '/health')).status, 200);
      expect((await _quote(handler)).status, 500);
      expect((await call(handler, 'GET', '/health')).status, 200);
      expect((await _quote(handler)).status, 200);
    });

    test('độ trễ đi qua hàm delay được tiêm, không chờ thật', () async {
      final waited = <Duration>[];
      final handler = createFakeApiHandler(
        faults: const FaultConfig(latency: Duration(seconds: 3)),
        delay: (d) async => waited.add(d),
      );
      expect((await _quote(handler)).status, 200);
      expect(waited, [const Duration(seconds: 3)]);
      // Không có độ trễ thì không gọi delay.
      final quiet = createFakeApiHandler(delay: (d) async => waited.add(d));
      await _quote(quiet);
      expect(waited, hasLength(1));
    });

    test('mỗi handler có bộ đếm riêng', () async {
      final a = createFakeApiHandler(faults: const FaultConfig(failFirst: 1));
      final b = createFakeApiHandler(faults: const FaultConfig(failFirst: 1));
      expect((await _quote(a)).status, 500);
      expect((await _quote(b)).status, 500);
      expect((await _quote(a)).status, 200);
    });
  });

  group('điểm mở rộng', () {
    test('module thêm route mới mà không sửa handler gốc', () async {
      final handler = createFakeApiHandler(modules: const [_PingModule()]);
      final reply = await call(handler, 'GET', '/ping');
      expect(reply.status, 200);
      expect(asMap(reply), {'pong': true});
      // Route cũ vẫn chạy.
      expect((await _quote(handler)).status, 200);
    });

    test(
      'middleware thêm vào chạy trước router (chỗ gắn kiểm token)',
      () async {
        final handler = createFakeApiHandler(
          middleware: [
            (Handler inner) => (Request request) {
              if (request.headers['authorization'] == null) {
                return problemResponse(request, 401, 'Unauthorized');
              }
              return inner(request);
            },
          ],
        );
        expect((await _quote(handler)).status, 401);
        final ok = await call(
          handler,
          'POST',
          '/quotes',
          body: {'serviceKind': 'ride', 'distanceKm': 5},
          headers: {'authorization': 'Bearer giá-trị-thử'},
        );
        expect(ok.status, 200);
      },
    );

    test('ProblemException trong module được đổi thành problem+json', () async {
      final handler = createFakeApiHandler(modules: [_ThrowingModule()]);
      final reply = await call(handler, 'GET', '/boom');
      expect(reply.status, 418);
      expect(asMap(reply)['title'], 'Ấm trà');
    });

    test('lỗi lập trình trong module không làm sập handler: 500', () async {
      final handler = createFakeApiHandler(modules: [_BuggyModule()]);
      final reply = await call(handler, 'GET', '/bug');
      expect(reply.status, 500);
    });
  });

  group('parseServerOptions', () {
    test('mặc định: loopback, cổng 8787, không tiêm lỗi', () {
      final options = parseServerOptions(const []);
      expect(options.host, '127.0.0.1');
      expect(options.port, 8787);
      expect(options.faults.latency, Duration.zero);
      expect(options.faults.failFirst, 0);
      expect(options.faults.failEvery, 0);
      expect(options.showHelp, isFalse);
    });

    test('đọc đủ các tuỳ chọn', () {
      final options = parseServerOptions(const [
        '--host=0.0.0.0',
        '--port=9000',
        '--latency-ms=250',
        '--fail-first=2',
        '--fail-every=5',
      ]);
      expect(options.host, '0.0.0.0');
      expect(options.port, 9000);
      expect(options.faults.latency, const Duration(milliseconds: 250));
      expect(options.faults.failFirst, 2);
      expect(options.faults.failEvery, 5);
    });

    test('--help', () {
      expect(parseServerOptions(const ['--help']).showHelp, isTrue);
    });

    for (final bad in [
      ['--port=abc'],
      ['--port=70000'],
      ['--port=-1'],
      ['--latency-ms=-5'],
      ['--fail-first=x'],
      ['--khong-co-tuy-chon'],
    ]) {
      test('${bad.single} -> FormatException', () {
        expect(() => parseServerOptions(bad), throwsFormatException);
      });
    }
  });
}

final class _ThrowingModule implements FakeApiModule {
  @override
  void mount(Router router, FakeApiContext context) {
    router.get('/boom', (Request request) {
      throw const ProblemException(418, 'Ấm trà');
    });
  }
}

final class _BuggyModule implements FakeApiModule {
  @override
  void mount(Router router, FakeApiContext context) {
    router.get('/bug', (Request request) {
      throw StateError('lỗi lập trình');
    });
  }
}
