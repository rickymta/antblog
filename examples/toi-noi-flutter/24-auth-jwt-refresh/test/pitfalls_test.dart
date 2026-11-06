import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shelf/shelf.dart' show Request;
import 'package:toi_noi_24/auth/auth_interceptor.dart';
import 'package:toi_noi_24/auth/token_store.dart';
import 'package:toi_noi_api/toi_noi_api.dart';

import 'support.dart';

/// Phiên bản ngây thơ #1: mỗi request bị 401 tự đi refresh, không dùng chung.
final class _RefreshPerRequest extends Interceptor {
  _RefreshPerRequest(this.dio, this.rig);

  final Dio dio;
  final Rig rig;

  @override
  Future<void> onRequest(
    RequestOptions options,
    RequestInterceptorHandler handler,
  ) async {
    final tokens = await rig.tokenStore.read();
    if (tokens != null) {
      options.headers['authorization'] = 'Bearer ${tokens.accessToken}';
    }
    handler.next(options);
  }

  @override
  Future<void> onError(
    DioException err,
    ErrorInterceptorHandler handler,
  ) async {
    if (err.response?.statusCode != 401 ||
        err.requestOptions.extra['again'] == true) {
      handler.next(err);
      return;
    }
    final tokens = (await rig.tokenStore.read())!;
    try {
      final fresh = await rig.authApi.refresh(tokens.refreshToken);
      await rig.tokenStore.save(fresh);
      final response = await dio.fetch<dynamic>(
        err.requestOptions.copyWith(
          headers: {
            ...err.requestOptions.headers,
            'authorization': 'Bearer ${fresh.accessToken}',
          },
          extra: {'again': true},
        ),
      );
      handler.resolve(response);
    } on ClientFailure {
      await rig.tokenStore.clear(); // "refresh hỏng thì đăng xuất"
      handler.next(err);
    } on DioException catch (e) {
      handler.next(e);
    }
  }
}

/// Phiên bản ngây thơ #2: `QueuedInterceptor` và refresh bằng chính `Dio` này.
final class _QueuedSameDio extends QueuedInterceptor {
  _QueuedSameDio(this.dio, this.refreshToken);

  final Dio dio;
  final String refreshToken;

  @override
  Future<void> onError(
    DioException err,
    ErrorInterceptorHandler handler,
  ) async {
    if (err.response?.statusCode != 401 ||
        err.requestOptions.path == '/auth/refresh') {
      handler.next(err);
      return;
    }
    try {
      // Đi qua chính interceptor này: nếu refresh cũng nhận 401, lỗi đó xếp
      // hàng sau task đang chờ nó.
      await dio.post<String>(
        '/auth/refresh',
        data: {'refreshToken': refreshToken},
      );
      handler.next(err);
    } on DioException catch (e) {
      handler.next(e);
    }
  }
}

/// Phiên bản ngây thơ #3: refresh đã đi qua `Dio` riêng, nhưng phát lại bằng
/// `Dio` có `QueuedInterceptor` này.
final class _QueuedReplaySameDio extends QueuedInterceptor {
  _QueuedReplaySameDio(this.dio, this.rig);

  final Dio dio;
  final Rig rig;

  @override
  Future<void> onError(
    DioException err,
    ErrorInterceptorHandler handler,
  ) async {
    if (err.response?.statusCode != 401) {
      handler.next(err);
      return;
    }
    final tokens = (await rig.tokenStore.read())!;
    final fresh = await rig.authApi.refresh(tokens.refreshToken);
    try {
      // Phát lại qua chính Dio này: nếu cũng 401, lỗi xếp sau task đang chờ nó.
      handler.resolve(
        await dio.fetch<dynamic>(
          err.requestOptions.copyWith(
            headers: {
              ...err.requestOptions.headers,
              'authorization': 'Bearer ${fresh.accessToken}',
            },
          ),
        ),
      );
    } on DioException catch (e) {
      handler.next(e);
    }
  }
}

/// Ghi lại thứ tự gọi để chứng minh dio chạy onError theo thứ tự danh sách.
final class _Recorder extends Interceptor {
  _Recorder(this.name, this.calls);

  final String name;
  final List<String> calls;

  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    calls.add('$name.onRequest');
    handler.next(options);
  }

  @override
  void onError(DioException err, ErrorInterceptorHandler handler) {
    calls.add('$name.onError');
    handler.next(err);
  }
}

void main() {
  test('dio chạy onRequest và onError theo cùng một thứ tự: đúng thứ tự trong danh sách', () async {
    final rig = Rig();
    final calls = <String>[];
    final dio =
        Dio(
            BaseOptions(
              baseUrl: 'http://fake.test',
              responseType: ResponseType.plain,
            ),
          )
          ..httpClientAdapter = rig.adapter
          ..interceptors.addAll([_Recorder('A', calls), _Recorder('B', calls)]);
    await expectLater(
      dio.get<String>('/trips/x'),
      throwsA(isA<DioException>()),
    );
    expect(calls, ['A.onRequest', 'B.onRequest', 'A.onError', 'B.onError']);
  });

  test('refresh riêng từng request + rotation: 5 request song song bị coi là kẻ trộm và đăng xuất oan', () async {
    final rig = Rig(
      // Mạng chậm: cả năm request và cả năm lời gọi refresh cùng bay một lúc.
      wrap: (fake) {
        final hold401 = holdUntil(
          fake,
          count: 5,
          when: (Request r) => r.url.path == 'trips',
        );
        return holdUntil(
          hold401,
          count: 5,
          when: (Request r) => r.url.path == 'auth/refresh',
        );
      },
    );
    await rig.signIn();
    rig.expireAccessTokens();
    // Từ đây client dùng bản ngây thơ.
    rig.client.dio
      ..interceptors.clear()
      ..interceptors.add(_RefreshPerRequest(rig.client.dio, rig));

    final results = await Future.wait([
      for (var i = 0; i < 5; i++)
        rig.book('k-$i').then<Object>((t) => t, onError: (Object e) => e),
    ]);

    expect(rig.refreshCalls, 5);
    expect(
      rig.auth.revokedFamilies,
      1,
      reason: 'server thấy token cũ bị dùng lại',
    );
    expect(rig.auth.activeFamilies, 0);
    expect(
      await rig.tokenStore.read(),
      isNull,
      reason: 'người dùng bị đăng xuất',
    );
    expect(results.whereType<ClientFailure>(), isNotEmpty);
  });

  test('log đặt sau AuthInterceptor không thấy dòng lỗi 401 (lỗi đã được xử lý trước đó)', () async {
    final rig = Rig();
    rig.client.dio.interceptors
      ..clear()
      ..addAll([
        AuthInterceptor(dio: rig.client.dio, session: rig.session),
        SafeLogInterceptor(rig.logs.add),
      ]);
    await rig.signIn();
    rig.expireAccessTokens();
    await rig.book('k-1');
    final trips = [
      for (final line in rig.logs)
        if (line.contains('/trips')) line.split(' (').first.split(' {').first,
    ];
    // Hai dòng request (lần đầu và lần phát lại), không có dòng ERROR 401.
    expect(trips, [
      '--> POST /trips',
      '--> POST /trips',
      '<-- 201 POST /trips',
    ]);
  });

  test(
    'cùng kịch bản với AuthInterceptor: một refresh, không ai bị đăng xuất',
    () async {
      final rig = Rig(
        wrap: (fake) => holdUntil(
          fake,
          count: 5,
          when: (Request r) => r.url.path == 'trips',
        ),
      );
      await rig.signIn();
      rig.expireAccessTokens();
      final results = await Future.wait([
        for (var i = 0; i < 5; i++) rig.book('k-$i'),
      ]);
      expect(results, hasLength(5));
      expect(rig.refreshCalls, 1);
      expect(rig.auth.revokedFamilies, 0);
      expect(await rig.tokenStore.read(), isNotNull);
    },
  );

  test('QueuedInterceptor refresh qua chính Dio: refresh bị 401 thì treo vĩnh viễn', () async {
    final rig = Rig();
    final tokens = await rig.signIn();
    await rig.authApi.logout(tokens.refreshToken); // refresh sẽ nhận 401
    rig.expireAccessTokens();

    final dio = Dio(
      BaseOptions(
        baseUrl: 'http://fake.test',
        responseType: ResponseType.plain,
      ),
    )..httpClientAdapter = rig.adapter;
    dio.interceptors.add(_QueuedSameDio(dio, tokens.refreshToken));

    final call = dio.get<String>(
      '/trips/x',
      options: Options(
        headers: {'authorization': 'Bearer ${tokens.accessToken}'},
      ),
    );
    expect(await completesWithin(call), isFalse);
    // Lỗi 401 của chính lời gọi refresh đã tới server, rồi xếp hàng mãi.
    expect(rig.refreshCalls, 1);
  });

  test('QueuedInterceptor với refresh trên Dio riêng vẫn treo nếu lần phát lại cũng 401', () async {
    final rig = Rig(
      wrap: (fake) =>
          (request) => request.url.path.startsWith('trips')
          ? problem(401)
          : fake(request),
    );
    final tokens = await rig.signIn();
    final dio = Dio(
      BaseOptions(
        baseUrl: 'http://fake.test',
        responseType: ResponseType.plain,
      ),
    )..httpClientAdapter = rig.adapter;
    dio.interceptors.add(_QueuedReplaySameDio(dio, rig));

    final call = dio.get<String>(
      '/trips/x',
      options: Options(
        headers: {'authorization': 'Bearer ${tokens.accessToken}'},
      ),
    );
    expect(await completesWithin(call), isFalse);
    expect(rig.refreshCalls, 1, reason: 'refresh xong, treo ở lần phát lại');
  });

  test('cùng kịch bản với AuthInterceptor: xong ngay, request nhận 401 và phiên đã kết thúc', () async {
    final rig = Rig();
    final tokens = await rig.signIn();
    await rig.authApi.logout(tokens.refreshToken);
    rig.expireAccessTokens();

    final call = rig
        .book('k-1')
        .then<Object>((t) => t, onError: (Object e) => e);
    expect(await completesWithin(call), isTrue);
    expect(await call, isA<ClientFailure>());
    expect(rig.signedOutCount, 1);
  });

  test('kho giữ token lạ (không phải JWT, refresh không có thật): 401, refresh bị từ chối, đăng xuất', () async {
    final rig = Rig(tokenStore: InMemoryTokenStore());
    await rig.tokenStore.save(
      const AuthTokens(accessToken: 'opaque', refreshToken: 'rt_x'),
    );
    await expectLater(rig.book('k-1'), throwsA(isA<ClientFailure>()));
    expect(rig.signedOutCount, 1);
    expect(await rig.tokenStore.read(), isNull);
  });
}
