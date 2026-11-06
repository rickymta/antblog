import 'dart:async';
import 'dart:io' show HandshakeException, HttpException, SocketException;

import 'package:dio/dio.dart' as dio show Response;
import 'package:dio/dio.dart' hide Response;
import 'package:shelf/shelf.dart' show Response;
import 'package:test/test.dart';
import 'package:toi_noi_api/fault_injection.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'retry_support.dart';
import 'support.dart';

/// Nếu `ApiFailure` thêm một biến thể mà hàm này không xử lý, file không biên dịch.
String _label(ApiFailure failure) => switch (failure) {
  NetworkFailure() => 'network',
  TimeoutFailure() => 'timeout',
  ServerFailure() => 'server',
  ClientFailure() => 'client',
  ParseFailure() => 'parse',
};

RequestOptions _options() => RequestOptions(
  path: '/quotes',
  connectTimeout: const Duration(seconds: 5),
  sendTimeout: const Duration(seconds: 6),
  receiveTimeout: const Duration(seconds: 10),
);

DioException _badResponse(int status, Object? data) => DioException.badResponse(
  statusCode: status,
  requestOptions: _options(),
  response: dio.Response<Object?>(
    requestOptions: _options(),
    statusCode: status,
    data: data,
  ),
);

Never _boom() => throw StateError('lỗi lập trình');

void main() {
  group('toApiFailure: DioException theo type', () {
    test('ba loại hết thời gian chờ, kèm giai đoạn và hạn đã đặt', () {
      final connect = toApiFailure(
        DioException.connectionTimeout(
          timeout: const Duration(seconds: 5),
          requestOptions: _options(),
        ),
      );
      final send = toApiFailure(
        DioException.sendTimeout(
          timeout: const Duration(seconds: 6),
          requestOptions: _options(),
        ),
      );
      final receive = toApiFailure(
        DioException.receiveTimeout(
          timeout: const Duration(seconds: 10),
          requestOptions: _options(),
        ),
      );
      expect(connect, isA<TimeoutFailure>());
      expect((connect as TimeoutFailure).phase, TimeoutPhase.connect);
      expect(connect.limit, const Duration(seconds: 5));
      expect((send as TimeoutFailure).phase, TimeoutPhase.send);
      expect(send.limit, const Duration(seconds: 6));
      expect((receive as TimeoutFailure).phase, TimeoutPhase.receive);
      expect(receive.limit, const Duration(seconds: 10));
      expect([connect, send, receive].every((f) => f.retryable), isTrue);
    });

    test('connectionError: NetworkFailure thử lại được', () {
      final failure = toApiFailure(
        DioException.connectionError(
          requestOptions: _options(),
          reason: 'Failed host lookup',
        ),
      );
      expect(failure, isA<NetworkFailure>());
      expect((failure as NetworkFailure).reason, NetworkReason.unreachable);
      expect(failure.retryable, isTrue);
      expect(failure.cause, isA<DioException>());
    });

    test('badCertificate: NetworkFailure nhưng không thử lại', () {
      final failure = toApiFailure(
        DioException.badCertificate(requestOptions: _options()),
      );
      expect(failure, isA<NetworkFailure>());
      expect((failure as NetworkFailure).reason, NetworkReason.badCertificate);
      expect(failure.retryable, isFalse);
    });

    test('badResponse 503 có problem+json: ServerFailure mang traceId', () {
      final failure = toApiFailure(
        _badResponse(503, {
          'title': 'Service unavailable',
          'status': 503,
          'traceId': 'fake-000042',
        }),
      );
      expect(failure, isA<ServerFailure>());
      failure as ServerFailure;
      expect(failure.statusCode, 503);
      expect(failure.title, 'Service unavailable');
      expect(failure.traceId, 'fake-000042');
      expect(failure.retryable, isTrue);
    });

    test('badResponse 502 với body HTML của proxy: vẫn ServerFailure', () {
      final failure = toApiFailure(
        _badResponse(502, '<html>Bad gateway</html>'),
      );
      expect(failure, isA<ServerFailure>());
      expect((failure as ServerFailure).title, 'HTTP 502');
      expect(failure.traceId, isNull);
    });

    test('badResponse 422: ClientFailure mang fieldErrors, không thử lại', () {
      final failure = toApiFailure(
        _badResponse(422, {
          'title': 'Unprocessable request',
          'errors': {
            'distanceKm': ['Expected 0..100 km'],
          },
        }),
      );
      expect(failure, isA<ClientFailure>());
      failure as ClientFailure;
      expect(failure.statusCode, 422);
      expect(failure.fieldErrors['distanceKm'], ['Expected 0..100 km']);
      expect(failure.retryable, isFalse);
    });

    test('ranh giới: 499 là client, 500 là server', () {
      expect(toApiFailure(_badResponse(499, null)), isA<ClientFailure>());
      expect(toApiFailure(_badResponse(500, null)), isA<ServerFailure>());
    });

    test(
      'unknown mang HttpException hoặc SocketException: mất mạng, thử lại được',
      () {
        for (final inner in <Object>[
          const HttpException('Connection closed while receiving data'),
          const SocketException('Connection reset by peer'),
        ]) {
          final failure = toApiFailure(
            DioException(requestOptions: _options(), error: inner),
          );
          expect(failure, isA<NetworkFailure>(), reason: '$inner');
          failure as NetworkFailure;
          expect(failure.reason, NetworkReason.unreachable);
          expect(failure.retryable, isTrue);
        }
      },
    );

    test('unknown mang HandshakeException: lỗi TLS, không thử lại', () {
      final failure = toApiFailure(
        DioException(
          requestOptions: _options(),
          error: const HandshakeException(
            'Connection terminated during handshake',
          ),
        ),
      );
      expect(failure, isA<NetworkFailure>());
      expect((failure as NetworkFailure).reason, NetworkReason.badCertificate);
      expect(failure.retryable, isFalse);
    });

    test('transformTimeout được xếp vào TimeoutFailure(receive)', () {
      final failure = toApiFailure(
        DioException.transformTimeout(
          timeout: const Duration(seconds: 1),
          requestOptions: _options(),
        ),
      );
      expect(failure, isA<TimeoutFailure>());
      expect((failure as TimeoutFailure).phase, TimeoutPhase.receive);
    });

    test('unknown mà lỗi gốc là DtoFormatException: ParseFailure', () {
      final failure = toApiFailure(
        DioException(
          requestOptions: _options(),
          error: DtoFormatException('QuoteDto', 'fare', 'thiếu trường'),
        ),
      );
      expect(failure, isA<ParseFailure>());
    });

    test('cancel: ném lại nguyên vẹn, không phải thất bại', () {
      final cancel = DioException.requestCancelled(
        requestOptions: _options(),
        reason: 'thoát màn hình',
      );
      expect(() => toApiFailure(cancel), throwsA(same(cancel)));
    });

    test('unknown với lỗi lạ: ném lại nguyên vẹn thay vì giả làm lỗi mạng', () {
      final odd = DioException(
        requestOptions: _options(),
        error: const FormatException('lạ'),
      );
      expect(() => toApiFailure(odd), throwsA(same(odd)));
    });
  });

  group('toApiFailure: các loại lỗi khác', () {
    test('ApiException 500 và 404', () {
      final server = toApiFailure(
        const ApiException(statusCode: 500, title: 't', traceId: 'x'),
      );
      final client = toApiFailure(
        const ApiException(statusCode: 404, title: 't'),
      );
      expect(server, isA<ServerFailure>());
      expect(client, isA<ClientFailure>());
      expect(server.cause, isA<ApiException>());
    });

    test('DtoFormatException: ParseFailure nêu DTO, trường và thông báo', () {
      final failure = toApiFailure(
        DtoFormatException('TripDto', 'status', 'giá trị lạ'),
      );
      expect(failure, isA<ParseFailure>());
      failure as ParseFailure;
      expect(failure.dto, 'TripDto');
      expect(failure.field, 'status');
      expect(failure.message, contains('giá trị lạ'));
      expect(failure.retryable, isFalse);
    });

    test('ApiFailure đi qua nguyên vẹn', () {
      const failure = NetworkFailure();
      expect(toApiFailure(failure), same(failure));
    });

    test('lỗi lập trình (StateError, TypeError) không bị che', () {
      expect(() => toApiFailure(StateError('x')), throwsStateError);
      expect(() => toApiFailure(TypeError()), throwsA(isA<TypeError>()));
    });

    test('năm biến thể, switch vét cạn biên dịch được', () {
      final all = <ApiFailure>[
        const NetworkFailure(),
        const TimeoutFailure(phase: TimeoutPhase.receive),
        const ServerFailure(statusCode: 500, title: 't'),
        const ClientFailure(statusCode: 400, title: 't'),
        const ParseFailure(dto: 'D', field: 'f', message: 'm'),
      ];
      expect(all.map(_label), [
        'network',
        'timeout',
        'server',
        'client',
        'parse',
      ]);
      expect(all.map((f) => f.retryable), [true, true, true, false, false]);
    });
  });

  group('guardApi', () {
    test('thành công: trả giá trị', () async {
      expect(await guardApi(() async => 7), 7);
    });

    test('lỗi lập trình ném lại nguyên vẹn, giữ stack trace gốc', () async {
      Object? caught;
      StackTrace? stack;
      try {
        await guardApi<void>(() async => _boom());
      } catch (e, s) {
        caught = e;
        stack = s;
      }
      expect(caught, isA<StateError>());
      expect(stack.toString(), contains('_boom'));
    });

    test('giữ stack trace gốc khi đổi lỗi sang ApiFailure', () async {
      Future<void> fail() async =>
          throw const ApiException(statusCode: 500, title: 'x');
      StackTrace? stack;
      try {
        await guardApi<void>(fail);
      } on ApiFailure catch (_, s) {
        stack = s;
      }
      expect(stack.toString(), contains('fail'));
    });

    test('báo giá 422: ClientFailure kèm fieldErrors', () async {
      final lab = RetryLab();
      await expectLater(
        guardApi(
          () => lab.client.quote(
            const QuoteRequestDto(
              serviceKind: ServiceKind.ride,
              distanceKm: 100.5,
            ),
          ),
        ),
        throwsA(
          isA<ClientFailure>()
              .having((f) => f.statusCode, 'status', 422)
              .having(
                (f) => f.fieldErrors.keys,
                'fields',
                contains('distanceKm'),
              ),
        ),
      );
    });

    test(
      '500 tiêm bằng FaultConfig: ServerFailure mang traceId của fake',
      () async {
        final lab = RetryLab(
          faults: const FaultConfig(failFirst: 99),
          maxAttempts: 1,
        );
        await expectLater(
          guardApi(() => lab.client.quote(ride5km)),
          throwsA(
            isA<ServerFailure>()
                .having((f) => f.statusCode, 'status', 500)
                .having((f) => f.traceId, 'traceId', 'fake-000001'),
          ),
        );
      },
    );

    test('200 nhưng là HTML (captive portal): ParseFailure', () async {
      final stub = clientFor(
        (request) => Response.ok(
          '<html>Đăng nhập Wi-Fi</html>',
          headers: {'content-type': 'text/html'},
        ),
      );
      await expectLater(
        guardApi(() => stub.client.quote(ride5km)),
        throwsA(isA<ParseFailure>().having((f) => f.dto, 'dto', 'QuoteDto')),
      );
    });

    test('kết nối bị từ chối: NetworkFailure', () async {
      final lab = RetryLab(script: [NetworkFault.refuse], maxAttempts: 1);
      await expectLater(
        guardApi(() => lab.client.quote(ride5km)),
        throwsA(isA<NetworkFailure>()),
      );
    });

    test(
      'server treo 10 giây: TimeoutFailure(receive, 10 s) trên đồng hồ giả',
      () {
        final lab = RetryLab(
          script: [NetworkFault.hang],
          maxAttempts: 1,
          receiveTimeout: const Duration(seconds: 10),
          realSleep: true,
        );
        final run = runFake(() => guardApi(() => lab.client.quote(ride5km)));
        expect(run.error, isA<TimeoutFailure>());
        final failure = run.error! as TimeoutFailure;
        expect(failure.phase, TimeoutPhase.receive);
        expect(failure.limit, const Duration(seconds: 10));
        expect(run.elapsed, const Duration(seconds: 10));
      },
    );

    test('huỷ request: CancelToken thì lỗi cancel đi thẳng ra', () async {
      final token = CancelToken()..cancel('đóng màn hình');
      final lab = RetryLab();
      await expectLater(
        guardApi(
          () => lab.client.dio.get<Object?>('/trips/x', cancelToken: token),
        ),
        throwsA(
          isA<DioException>().having(
            (e) => e.type,
            'type',
            DioExceptionType.cancel,
          ),
        ),
      );
    });

    test('sau retry hết lượt: lỗi cuối vẫn thành ServerFailure', () async {
      final lab = RetryLab(faults: const FaultConfig(failFirst: 99));
      await expectLater(
        guardApi(() => lab.client.quote(ride5km)),
        throwsA(isA<ServerFailure>()),
      );
      expect(lab.inProcess.requests, hasLength(3));
    });
  });
}
