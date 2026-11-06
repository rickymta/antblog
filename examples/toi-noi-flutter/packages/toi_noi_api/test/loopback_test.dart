import 'package:dio/dio.dart' hide Response;
import 'package:shelf/shelf_io.dart' as shelf_io;
import 'package:test/test.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'support.dart';

/// Đối chứng: cùng kịch bản chạy qua socket loopback thật (adapter mặc định
/// của dio) phải cho đúng kết quả như adapter trong tiến trình. Đây là bằng
/// chứng rằng `InProcessAdapter` không nói dối về hành vi HTTP.
void main() {
  test('socket loopback và InProcessAdapter cho cùng kết quả', () async {
    var seq = 0;
    final server = await shelf_io.serve(
      createFakeApiHandler(clock: () => fixedNow, newId: () => 'T-${++seq}'),
      'localhost',
      0,
    );
    addTearDown(() => server.close(force: true));
    final real = ToiNoiApiClient(
      baseUrl: Uri.parse('http://localhost:${server.port}'),
    );
    addTearDown(real.close);
    final realDriver = ToiNoiApiClient(
      baseUrl: Uri.parse('http://localhost:${server.port}'),
      interceptors: [
        InterceptorsWrapper(
          onRequest: (options, handler) {
            options.headers[fakeDriverIdHeader] = 'D-1';
            handler.next(options);
          },
        ),
      ],
    );
    addTearDown(realDriver.close);
    final lab = Lab();

    const request = QuoteRequestDto(
      serviceKind: ServiceKind.ride,
      distanceKm: 2.001,
    );
    Future<List<Object?>> run(
      ToiNoiApiClient client,
      ToiNoiApiClient driver,
    ) async {
      final quote = await client.quote(request);
      final booked = await client.createTrip(request, idempotencyKey: 'k-1');
      final accepted = await driver.acceptTrip(booked.id, expectedVersion: 0);
      final seen = await client.getTrip(booked.id);
      Object? failure;
      try {
        await client.getTrip('khong-co');
      } on ApiException catch (e) {
        failure = '${e.statusCode} ${e.title}';
      }
      return [
        quote.toJson(),
        booked.toJson(),
        accepted.toJson(),
        seen.toJson(),
        failure,
      ];
    }

    final overSocket = await run(real, realDriver);
    final overMemory = await run(lab.client, lab.driver('D-1'));
    expect(overSocket, overMemory);
    expect((overSocket[0]! as Map<String, Object?>)['fare'], 12005);
    expect(overSocket.last, '404 Trip not found');
  });
}
