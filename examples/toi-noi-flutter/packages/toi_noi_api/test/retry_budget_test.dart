import 'dart:io' show HttpException;

import 'package:dio/dio.dart' as dio show Response;
import 'package:dio/dio.dart' hide Response;
import 'package:shelf/shelf.dart' show Handler, Response;
import 'package:test/test.dart';
import 'package:toi_noi_api/fault_injection.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_fake_api/testing.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'retry_support.dart';
import 'support.dart' show okJson;

/// Interceptor đếm số lần được gọi và ghi lại header nó thấy.
final class _Probe extends Interceptor {
  int requests = 0;
  int errors = 0;
  int responses = 0;
  final List<String?> hops = [];

  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    requests++;
    // Nối thêm vào header ở mỗi lần chạy: nếu request đã qua đây một lần rồi
    // mới được gửi lại, giá trị sẽ dài ra.
    final seen = options.headers['x-hop'] as String?;
    options.headers['x-hop'] = '${seen ?? ''}+';
    hops.add(options.headers['x-hop'] as String?);
    handler.next(options);
  }

  @override
  void onResponse(
    dio.Response<dynamic> response,
    ResponseInterceptorHandler h,
  ) {
    responses++;
    h.next(response);
  }

  @override
  void onError(DioException err, ErrorInterceptorHandler handler) {
    errors++;
    handler.next(err);
  }
}

/// Client nối vào một handler tự viết, có `RetryInterceptor` đứng giữa log và probe.
({ToiNoiApiClient client, List<Duration> waits, InProcessAdapter adapter})
_stub(
  Handler handler, {
  int maxAttempts = 3,
  Duration maxDelay = const Duration(seconds: 8),
}) {
  final adapter = InProcessAdapter(handler);
  final client = ToiNoiApiClient(
    baseUrl: Uri.parse('http://stub.test'),
    adapter: adapter,
  );
  final waits = <Duration>[];
  client.dio.interceptors.add(
    RetryInterceptor(
      dio: client.dio,
      maxAttempts: maxAttempts,
      maxDelay: maxDelay,
      random: () => 1.0,
      sleep: (d) async => waits.add(d),
    ),
  );
  return (client: client, waits: waits, adapter: adapter);
}

void main() {
  group('mã 408 và 429 là tạm thời, 4xx khác thì không', () {
    for (final status in [408, 429]) {
      test('$status được thử lại rồi thành công', () async {
        var calls = 0;
        final stub = _stub(
          (request) => ++calls == 1
              ? Response(status, body: '{"title":"x","status":$status}')
              : okJson({
                  'serviceKind': 'ride',
                  'distanceMeters': 5000,
                  'fare': 25500,
                  'currency': 'VND',
                }),
        );
        expect((await stub.client.quote(ride5km)).fareVnd, 25500);
        expect(calls, 2);
      });
    }

    for (final status in [400, 401, 403, 404, 409, 422]) {
      test('$status không được thử lại', () async {
        var calls = 0;
        final stub = _stub((request) {
          calls++;
          return Response(status, body: '{"title":"x","status":$status}');
        });
        await expectLater(
          stub.client.getTrip('T-1'),
          throwsA(isA<ApiException>()),
        );
        expect(calls, 1);
      });
    }

    test('ClientFailure 408 và 429 là retryable, 4xx khác thì không', () {
      for (final status in [408, 429]) {
        expect(ClientFailure(statusCode: status, title: 't').retryable, isTrue);
      }
      for (final status in [400, 401, 404, 409, 422]) {
        expect(
          ClientFailure(statusCode: status, title: 't').retryable,
          isFalse,
        );
      }
    });
  });

  group('Retry-After', () {
    Handler limited(String retryAfter) {
      var calls = 0;
      return (request) => ++calls == 1
          ? Response(
              429,
              body: '{"title":"x","status":429}',
              headers: {'retry-after': retryAfter},
            )
          : okJson({
              'serviceKind': 'ride',
              'distanceMeters': 5000,
              'fare': 25500,
              'currency': 'VND',
            });
    }

    test('chờ ít nhất bằng Retry-After khi nó lớn hơn backoff', () async {
      final stub = _stub(limited('3'));
      await stub.client.quote(ride5km);
      // backoff lần hai chạm trần 500 ms, Retry-After đòi 3 giây.
      expect(stub.waits, [const Duration(seconds: 3)]);
    });

    test('Retry-After nhỏ hơn backoff thì giữ backoff', () async {
      final stub = _stub(limited('0'));
      await stub.client.quote(ride5km);
      expect(stub.waits, [const Duration(milliseconds: 500)]);
    });

    test(
      'Retry-After vượt maxDelay thì bỏ cuộc, không treo người dùng',
      () async {
        final stub = _stub(limited('120'));
        await expectLater(
          stub.client.quote(ride5km),
          throwsA(
            isA<ApiException>().having((e) => e.statusCode, 'status', 429),
          ),
        );
        expect(stub.adapter.requests, hasLength(1));
        expect(stub.waits, isEmpty);
      },
    );

    test('Retry-After không phải số giây thì bị bỏ qua', () async {
      final stub = _stub(limited('Wed, 21 Oct 2026 07:28:00 GMT'));
      await stub.client.quote(ride5km);
      expect(stub.waits, [const Duration(milliseconds: 500)]);
    });
  });

  group('lỗi mạng mà dio để ở unknown', () {
    test('HttpException (kết nối đứt giữa body) được thử lại', () async {
      final lab = RetryLab(script: [NetworkFault.reset]);
      final quote = await lab.client.quote(ride5km);
      expect(quote.fareVnd, 25500);
      expect(lab.faulty.calls, 2);
      // Server đã xử lý lần đầu: reset xảy ra sau khi nó chạy xong.
      expect(lab.inProcess.requests, hasLength(2));
    });

    test('reset với POST có khoá: gửi lại, vẫn một chuyến', () async {
      final lab = RetryLab(script: [NetworkFault.reset]);
      final trip = await lab.client.createTrip(ride5km, idempotencyKey: 'k-1');
      expect(trip.id, 'T-1');
      expect(lab.store.trips, hasLength(1));
    });

    test('reset với POST không khoá: không gửi lại', () async {
      final lab = RetryLab(script: [NetworkFault.reset]);
      await expectLater(
        lab.client.dio.post<Object?>(
          '/trips',
          data: const {'serviceKind': 'ride', 'distanceKm': 5},
        ),
        throwsA(isA<DioException>()),
      );
      expect(lab.faulty.calls, 1);
    });

    test('lỗi TLS không được thử lại', () async {
      final lab = RetryLab(script: [NetworkFault.tlsFailure]);
      await expectLater(
        lab.client.quote(ride5km),
        throwsA(isA<DioException>()),
      );
      expect(lab.faulty.calls, 1);
    });

    test('HttpException tự dựng ở adapter khác cũng được nhận ra', () async {
      var calls = 0;
      final adapter = _Throwing(() {
        calls++;
        return const HttpException('Connection closed while receiving data');
      });
      final client = ToiNoiApiClient(
        baseUrl: Uri.parse('http://stub.test'),
        adapter: adapter,
      );
      client.dio.interceptors.add(
        RetryInterceptor(dio: client.dio, sleep: (_) async {}),
      );
      await expectLater(
        guardApi(() => client.getTrip('T-1')),
        throwsA(isA<NetworkFailure>()),
      );
      expect(calls, 3);
    });
  });

  group('ngân sách tổng maxElapsed (đồng hồ giả)', () {
    test('lần thử cuối bị chặn timeout bằng phần hạn còn lại', () {
      // hang x3, receiveTimeout 10 s, backoff 0,5 s và 1 s, hạn tổng 25 s:
      // lần 1 tới 10 s, lần 2 tới 20,5 s, lần 3 chỉ còn 3,5 s.
      final lab = RetryLab(
        script: [NetworkFault.hang, NetworkFault.hang, NetworkFault.hang],
        receiveTimeout: const Duration(seconds: 10),
        maxElapsed: const Duration(seconds: 25),
        realSleep: true,
      );
      final run = runFake(() => lab.client.quote(ride5km));
      expect(run.error, isA<DioException>());
      expect(run.elapsed, const Duration(seconds: 25));
      expect(lab.faulty.calls, 3);
    });

    test('không đủ hạn cho lần kế thì dừng, không bắt đầu lần thử mới', () {
      // Sau lần 1 (10 s) còn 2 s, trừ 0,5 s chờ vẫn còn 1,5 s: lần 2 chạy với
      // timeout 1,5 s. Sau đó hết hạn nên không có lần 3.
      final lab = RetryLab(
        script: [NetworkFault.hang, NetworkFault.hang, NetworkFault.hang],
        receiveTimeout: const Duration(seconds: 10),
        maxElapsed: const Duration(seconds: 12),
        realSleep: true,
      );
      final run = runFake(() => lab.client.quote(ride5km));
      expect(run.error, isA<DioException>());
      expect(run.elapsed, const Duration(seconds: 12));
      expect(lab.faulty.calls, 2);
    });

    test('lần đầu đã quá hạn: không thử lại', () {
      final lab = RetryLab(
        script: [NetworkFault.hang],
        receiveTimeout: const Duration(seconds: 10),
        maxElapsed: const Duration(seconds: 5),
        realSleep: true,
      );
      final run = runFake(() => lab.client.quote(ride5km));
      expect(run.error, isA<DioException>());
      expect(run.elapsed, const Duration(seconds: 10));
      expect(lab.faulty.calls, 1);
    });

    test('không đặt maxElapsed thì giữ nguyên 31,5 giây', () {
      final lab = RetryLab(
        script: [NetworkFault.hang, NetworkFault.hang, NetworkFault.hang],
        receiveTimeout: const Duration(seconds: 10),
        realSleep: true,
      );
      final run = runFake(() => lab.client.quote(ride5km));
      expect(run.elapsed, const Duration(seconds: 31, milliseconds: 500));
    });
  });

  group('thứ tự interceptor', () {
    test(
      'interceptor đứng sau retry: onRequest mỗi lần thử, onError đúng một lần',
      () async {
        final probe = _Probe();
        final lab = RetryLab(
          faults: const FaultConfig(failFirst: 99),
          after: [probe],
        );
        await expectLater(
          lab.client.quote(ride5km),
          throwsA(isA<ApiException>()),
        );
        expect(probe.requests, 3);
        expect(probe.errors, 1); // chỉ lỗi cuối, không phải ba lần
        expect(probe.responses, 0);
      },
    );

    test(
      'thành công ở lần ba: onResponse đúng một lần, onError không lần nào',
      () async {
        final probe = _Probe();
        final lab = RetryLab(
          faults: const FaultConfig(failFirst: 2),
          after: [probe],
        );
        await lab.client.quote(ride5km);
        expect(probe.requests, 3);
        expect(probe.responses, 1);
        expect(probe.errors, 0);
      },
    );

    test('header mà interceptor sau retry nối thêm không tích luỹ', () async {
      final probe = _Probe();
      final lab = RetryLab(
        faults: const FaultConfig(failFirst: 2),
        after: [probe],
      );
      await lab.client.quote(ride5km);
      expect(probe.hops, ['+', '+', '+']);
      expect(lab.inProcess.requests.map((r) => r.headers['x-hop']), [
        '+',
        '+',
        '+',
      ]);
    });

    test(
      'lỗi không thử lại được vẫn tới interceptor sau retry đúng một lần',
      () async {
        final probe = _Probe();
        final lab = RetryLab(after: [probe]);
        await expectLater(
          lab.client.getTrip('khong-co'),
          throwsA(isA<ApiException>()),
        );
        expect(probe.requests, 1);
        expect(probe.errors, 1);
      },
    );
  });
}

final class _Throwing implements HttpClientAdapter {
  _Throwing(this.make);

  final Object Function() make;

  @override
  Future<ResponseBody> fetch(
    RequestOptions options,
    Stream<dynamic>? requestStream,
    Future<void>? cancelFuture,
  ) async => throw DioException(requestOptions: options, error: make());

  @override
  void close({bool force = false}) {}
}
