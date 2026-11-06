import 'dart:async';

import 'package:dio/dio.dart' hide Response;
import 'package:shelf/shelf.dart' show Response;
import 'package:test/test.dart';
import 'package:toi_noi_api/toi_noi_api.dart';

import 'retry_support.dart';
import 'support.dart';

void main() {
  test('sendTimeout của client đi vào BaseOptions; mặc định không đặt', () {
    final withSend = ToiNoiApiClient(
      baseUrl: Uri.parse('http://x.test'),
      sendTimeout: const Duration(seconds: 7),
    );
    final without = ToiNoiApiClient(baseUrl: Uri.parse('http://x.test'));
    expect(withSend.dio.options.sendTimeout, const Duration(seconds: 7));
    expect(without.dio.options.sendTimeout, isNull);
  });

  group('cancelToken', () {
    test('quote bị huỷ khi server chưa trả lời: DioException cancel', () async {
      final never = Completer<Response>();
      final stub = clientFor((request) => never.future);
      final token = CancelToken();
      final call = stub.client.quote(ride5km, cancelToken: token);
      token.cancel('đổi ý');
      await expectLater(
        call,
        throwsA(
          isA<DioException>().having(
            (e) => e.type,
            'type',
            DioExceptionType.cancel,
          ),
        ),
      );
    });

    test('guardApi không biến huỷ thành ApiFailure', () async {
      final never = Completer<Response>();
      final stub = clientFor((request) => never.future);
      final token = CancelToken();
      final call = guardApi(
        () => stub.client.getTrip('T-1', cancelToken: token),
      );
      token.cancel();
      await expectLater(
        call,
        throwsA(
          isA<DioException>().having(
            (e) => e.type,
            'type',
            DioExceptionType.cancel,
          ),
        ),
      );
    });

    test('token chưa huỷ không ảnh hưởng kết quả của các lời gọi', () async {
      final lab = RetryLab();
      final token = CancelToken();
      final trip = await lab.client.createTrip(
        ride5km,
        idempotencyKey: 'k-1',
        cancelToken: token,
      );
      expect(
        (await lab.client.getTrip(trip.id, cancelToken: token)).id,
        trip.id,
      );
      expect(
        (await lab.client.quote(ride5km, cancelToken: token)).fareVnd,
        25500,
      );
      expect(token.isCancelled, isFalse);
    });

    test('acceptTrip nhận cancelToken', () async {
      final lab = RetryLab();
      final trip = await lab.client.createTrip(ride5km, idempotencyKey: 'k-1');
      final token = CancelToken()..cancel();
      await expectLater(
        lab.client.acceptTrip(trip.id, expectedVersion: 0, cancelToken: token),
        throwsA(isA<DioException>()),
      );
    });
  });
}
