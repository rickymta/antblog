import 'dart:async';
import 'dart:io' show SocketException;

import 'package:dio/dio.dart' hide Response;
import 'package:shelf/shelf.dart';
import 'package:test/test.dart';
import 'package:toi_noi_fake_api/testing.dart';

import 'support.dart';

Dio _dio(Handler handler, {Duration? receiveTimeout}) => Dio(
  BaseOptions(baseUrl: 'http://stub.test', receiveTimeout: receiveTimeout),
)..httpClientAdapter = InProcessAdapter(handler);

void main() {
  test('chuyển method, đường dẫn, query, header và body cho handler', () async {
    late Request seen;
    late String seenBody;
    final dio = _dio((request) async {
      seen = request;
      seenBody = await request.readAsString();
      return okJson({'ok': true}, status: 201);
    });
    final response = await dio.post<Object?>(
      '/echo',
      queryParameters: {'a': '1'},
      data: {'x': 1},
      options: Options(headers: {'X-Thu': 'gia-tri'}),
    );
    expect(seen.method, 'POST');
    expect(seen.requestedUri.toString(), 'http://stub.test/echo?a=1');
    expect(seen.headers['x-thu'], 'gia-tri');
    expect(seen.headers['content-type'], startsWith('application/json'));
    expect(seenBody, '{"x":1}');
    expect(response.statusCode, 201);
    expect(response.data, {'ok': true});
  });

  test('ghi lại request theo thứ tự, header chữ thường', () async {
    final adapter = InProcessAdapter((request) => okJson({}));
    final dio = Dio(BaseOptions(baseUrl: 'http://stub.test'))
      ..httpClientAdapter = adapter;
    await dio.get<Object?>('/a', options: Options(headers: {'X-Mot': '1'}));
    await dio.post<Object?>('/b', data: {'k': 'v'});
    expect(adapter.requests.map((r) => '${r.method} ${r.uri.path}'), [
      'GET /a',
      'POST /b',
    ]);
    expect(adapter.requests[0].headers['x-mot'], '1');
    expect(adapter.requests[0].body, isEmpty);
    expect(adapter.requests[1].body, '{"k":"v"}');
  });

  test(
    'mã không phải 2xx thành DioException.badResponse như adapter thật',
    () async {
      final dio = _dio((request) => Response.notFound('khong co'));
      await expectLater(
        dio.get<Object?>('/x'),
        throwsA(
          isA<DioException>()
              .having((e) => e.type, 'type', DioExceptionType.badResponse)
              .having((e) => e.response?.statusCode, 'status', 404),
        ),
      );
    },
  );

  test('body là chữ tiếng Việt có dấu đi qua UTF-8 nguyên vẹn', () async {
    final dio = _dio(
      (request) async => okJson({'echo': await request.readAsString()}),
    );
    final response = await dio.post<Map<String, Object?>>(
      '/echo',
      data: {'diaChi': 'Số 1 Đường Lê Lợi'},
    );
    expect(response.data!['echo'], '{"diaChi":"Số 1 Đường Lê Lợi"}');
  });

  test(
    'receiveTimeout: handler không trả lời thì ném receiveTimeout',
    () async {
      final never = Completer<Response>();
      final dio = _dio(
        (request) => never.future,
        receiveTimeout: const Duration(milliseconds: 20),
      );
      await expectLater(
        dio.get<Object?>('/x'),
        throwsA(
          isA<DioException>().having(
            (e) => e.type,
            'type',
            DioExceptionType.receiveTimeout,
          ),
        ),
      );
    },
  );

  test('không đặt receiveTimeout thì chờ tới khi handler trả lời', () async {
    final dio = _dio(
      (request) => Future<Response>.delayed(
        const Duration(milliseconds: 30),
        () => okJson({'late': true}),
      ),
    );
    final response = await dio.get<Object?>('/x');
    expect(response.data, {'late': true});
  });

  test(
    'handler ném SocketException: connectionError như socket thật',
    () async {
      final dio = _dio(
        (request) => throw const SocketException('Connection refused'),
      );
      await expectLater(
        dio.get<Object?>('/x'),
        throwsA(
          isA<DioException>()
              .having((e) => e.type, 'type', DioExceptionType.connectionError)
              .having((e) => e.error, 'error', isA<SocketException>())
              .having((e) => e.response, 'response', isNull),
        ),
      );
    },
  );

  test('SocketException ném bất đồng bộ cũng thành connectionError', () async {
    final dio = _dio(
      (request) async => throw const SocketException('Connection reset'),
    );
    await expectLater(
      dio.get<Object?>('/x'),
      throwsA(
        isA<DioException>().having(
          (e) => e.type,
          'type',
          DioExceptionType.connectionError,
        ),
      ),
    );
  });

  test('lỗi khác của handler không bị đổi thành connectionError', () async {
    final dio = _dio((request) => throw StateError('lỗi lập trình'));
    await expectLater(
      dio.get<Object?>('/x'),
      throwsA(
        isA<DioException>().having(
          (e) => e.type,
          'type',
          isNot(DioExceptionType.connectionError),
        ),
      ),
    );
  });
}
