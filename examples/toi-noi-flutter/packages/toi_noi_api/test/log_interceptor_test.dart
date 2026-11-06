import 'dart:async';

import 'package:dio/dio.dart' hide Response;
import 'package:shelf/shelf.dart';
import 'package:test/test.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'support.dart';

/// Giả lập interceptor xác thực của bài 24: gắn token vào mọi request.
Interceptor _addAuthorization(String token) => InterceptorsWrapper(
  onRequest: (options, handler) {
    options.headers['Authorization'] = 'Bearer $token';
    handler.next(options);
  },
);

const _secretToken = 'tok-DO-NOT-LOG-123';
const _request = QuoteRequestDto(serviceKind: ServiceKind.ride, distanceKm: 5);

void main() {
  late List<String> lines;

  setUp(() => lines = []);

  test('ghi method, đường dẫn, mã trạng thái và thời gian', () async {
    final lab = Lab(interceptors: [SafeLogInterceptor(lines.add)]);
    await lab.client.quote(_request);
    expect(lines, hasLength(2));
    expect(lines[0], startsWith('--> POST /quotes'));
    expect(lines[1], matches(RegExp(r'^<-- 200 POST /quotes \(\d+ ms\)')));
  });

  test(
    'header ngoài danh sách cho phép chỉ lộ tên, không lộ giá trị',
    () async {
      final lab = Lab(
        interceptors: [
          _addAuthorization(_secretToken),
          SafeLogInterceptor(lines.add),
        ],
      );
      await lab.client.createTrip(_request, idempotencyKey: 'book-private-77');
      final all = lines.join('\n');
      expect(all, isNot(contains(_secretToken)));
      expect(all, isNot(contains('Bearer')));
      expect(all, isNot(contains('book-private-77')));
      expect(all, contains('authorization: [redacted]'));
      expect(all, contains('idempotency-key: [redacted]'));
      // Header an toàn vẫn đọc được để gỡ lỗi.
      expect(all, contains('accept: application/json'));
    },
  );

  test(
    'log đặt trước hay sau interceptor xác thực đều không lộ token',
    () async {
      final before = <String>[];
      final labBefore = Lab(
        interceptors: [
          SafeLogInterceptor(before.add),
          _addAuthorization(_secretToken),
        ],
      );
      await labBefore.client.quote(_request);
      expect(before.join(), isNot(contains(_secretToken)));
    },
  );

  test('không ghi body của request lẫn response', () async {
    final lab = Lab(interceptors: [SafeLogInterceptor(lines.add)]);
    await lab.client.createTrip(_request, idempotencyKey: 'k-1');
    final all = lines.join('\n');
    expect(all, isNot(contains('distanceKm')));
    expect(all, isNot(contains('25500')));
    expect(all, isNot(contains('requestedAt')));
  });

  test('query chỉ ghi tên khoá, không ghi giá trị', () async {
    final lab = Lab(interceptors: [SafeLogInterceptor(lines.add)]);
    await lab.client.dio.get<Object?>(
      '/health',
      queryParameters: {'phone': '0900000000', 'otp': '123456'},
    );
    final all = lines.join('\n');
    expect(all, contains('/health?phone=[redacted]&otp=[redacted]'));
    expect(all, isNot(contains('0900000000')));
    expect(all, isNot(contains('123456')));
  });

  test('response: header lạ như Set-Cookie bị che', () async {
    final stub = clientFor(
      (request) => Response.ok(
        '{}',
        headers: const {
          'content-type': 'application/json',
          'set-cookie': 'session=cookie-secret-9',
        },
      ),
      interceptors: [SafeLogInterceptor(lines.add)],
    );
    await stub.client.dio.get<Object?>('/health');
    final all = lines.join('\n');
    expect(all, isNot(contains('cookie-secret-9')));
    expect(all, contains('set-cookie: [redacted]'));
  });

  test('lỗi HTTP: ghi loại lỗi và mã, không ghi nội dung lỗi', () async {
    final lab = Lab(
      interceptors: [SafeLogInterceptor(lines.add)],
      faults: const FaultConfig(failFirst: 1),
    );
    await expectLater(lab.client.quote(_request), throwsA(isA<ApiException>()));
    expect(
      lines.last,
      matches(RegExp(r'^<-- ERROR badResponse 500 POST /quotes \(\d+ ms\)$')),
    );
    expect(lines.join(), isNot(contains('Unexpected server error')));
  });

  test('lỗi mạng (timeout): ghi loại lỗi, không có mã', () async {
    final stub = clientFor(
      (request) => Completer<Response>().future,
      receiveTimeout: const Duration(milliseconds: 10),
      interceptors: [SafeLogInterceptor(lines.add)],
    );
    await expectLater(stub.client.getTrip('T-1'), throwsA(isA<DioException>()));
    expect(lines.last, startsWith('<-- ERROR receiveTimeout GET /trips/T-1'));
  });

  test('visibleHeaders tuỳ chỉnh: thêm tên vào danh sách cho phép', () async {
    final lab = Lab(
      interceptors: [
        SafeLogInterceptor(
          lines.add,
          visibleHeaders: {'accept', 'X-App-Version'},
        ),
        InterceptorsWrapper(
          onRequest: (options, handler) {
            options.headers['x-app-version'] = '1.4.0';
            handler.next(options);
          },
        ),
      ],
    );
    await lab.client.quote(_request);
    // Interceptor log đứng trước interceptor thêm header nên chưa thấy: log chỉ
    // thấy header đã có lúc nó chạy.
    expect(lines.first, isNot(contains('x-app-version')));

    lines.clear();
    final lab2 = Lab(
      interceptors: [
        InterceptorsWrapper(
          onRequest: (options, handler) {
            options.headers['x-app-version'] = '1.4.0';
            handler.next(options);
          },
        ),
        SafeLogInterceptor(
          lines.add,
          visibleHeaders: {'accept', 'X-App-Version'},
        ),
      ],
    );
    await lab2.client.quote(_request);
    expect(lines.first, contains('x-app-version: 1.4.0'));
    expect(lines.first, contains('content-type: [redacted]'));
  });
}
