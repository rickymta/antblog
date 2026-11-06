import 'dart:io' show HandshakeException, HttpException;

import 'package:dio/dio.dart';
import 'package:test/test.dart';
import 'package:toi_noi_api/fault_injection.dart';

import 'retry_support.dart';

void main() {
  test(
    'kịch bản áp theo thứ tự lần gọi; null và hết kịch bản thì đi thẳng',
    () async {
      final lab = RetryLab(script: [NetworkFault.refuse, null], maxAttempts: 1);
      await expectLater(
        lab.client.quote(ride5km),
        throwsA(isA<DioException>()),
      );
      expect((await lab.client.quote(ride5km)).fareVnd, 25500); // null
      expect((await lab.client.quote(ride5km)).fareVnd, 25500); // hết kịch bản
      expect(lab.faulty.calls, 3);
      expect(lab.inProcess.requests, hasLength(2));
    },
  );

  test('refuse: request không tới server', () async {
    final lab = RetryLab(script: [NetworkFault.refuse], maxAttempts: 1);
    await expectLater(
      lab.client.quote(ride5km),
      throwsA(
        isA<DioException>().having(
          (e) => e.type,
          'type',
          DioExceptionType.connectionError,
        ),
      ),
    );
    expect(lab.inProcess.requests, isEmpty);
  });

  test(
    'loseResponse: server có nhận và xử lý, client vẫn nhận lỗi kết nối',
    () async {
      final lab = RetryLab(script: [NetworkFault.loseResponse], maxAttempts: 1);
      await expectLater(
        lab.client.createTrip(ride5km, idempotencyKey: 'k-1'),
        throwsA(
          isA<DioException>().having(
            (e) => e.type,
            'type',
            DioExceptionType.connectionError,
          ),
        ),
      );
      expect(lab.inProcess.requests, hasLength(1));
      expect(lab.store.trips, hasLength(1));
    },
  );

  test('connectTimeout chờ đúng hạn kết nối trên đồng hồ giả', () {
    final lab = RetryLab(
      script: [NetworkFault.connectTimeout],
      maxAttempts: 1,
      connectTimeout: const Duration(seconds: 7),
    );
    final run = runFake(() => lab.client.quote(ride5km));
    expect(
      (run.error! as DioException).type,
      DioExceptionType.connectionTimeout,
    );
    expect(run.elapsed, const Duration(seconds: 7));
    expect(lab.inProcess.requests, isEmpty);
  });

  test('hang chờ đúng hạn nhận và request không tới server', () {
    final lab = RetryLab(
      script: [NetworkFault.hang],
      maxAttempts: 1,
      receiveTimeout: const Duration(seconds: 4),
    );
    final run = runFake(() => lab.client.quote(ride5km));
    expect((run.error! as DioException).type, DioExceptionType.receiveTimeout);
    expect(run.elapsed, const Duration(seconds: 4));
    expect(lab.inProcess.requests, isEmpty);
  });

  test(
    'reset: server có xử lý, client nhận HttpException ở kiểu unknown',
    () async {
      final lab = RetryLab(script: [NetworkFault.reset], maxAttempts: 1);
      await expectLater(
        lab.client.createTrip(ride5km, idempotencyKey: 'k-2'),
        throwsA(
          isA<DioException>()
              .having((e) => e.type, 'type', DioExceptionType.unknown)
              .having((e) => e.error, 'error', isA<HttpException>()),
        ),
      );
      expect(lab.store.trips, hasLength(1));
    },
  );

  test(
    'tlsFailure: HandshakeException ở kiểu unknown, không tới server',
    () async {
      final lab = RetryLab(script: [NetworkFault.tlsFailure], maxAttempts: 1);
      await expectLater(
        lab.client.quote(ride5km),
        throwsA(
          isA<DioException>().having(
            (e) => e.error,
            'error',
            isA<HandshakeException>(),
          ),
        ),
      );
      expect(lab.inProcess.requests, isEmpty);
    },
  );
}
