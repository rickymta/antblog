import 'dart:async';

import 'package:dio/dio.dart' show BaseOptions;
import 'package:flutter_test/flutter_test.dart';
import 'package:shelf/shelf.dart' show Response;
import 'package:toi_noi_24/auth/auth_api.dart';
import 'package:toi_noi_24/auth/auth_interceptor.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'support.dart';

TypeMatcher<ClientFailure> _client(int status) =>
    isA<ClientFailure>().having((f) => f.statusCode, 'status', status);

void main() {
  test('Dio của /auth/* không có AuthInterceptor lẫn RetryInterceptor', () {
    final rig = Rig(retry: RetryPlacement.afterAuth);
    final interceptors = rig.authApi.dio.interceptors;
    expect(interceptors.whereType<AuthInterceptor>(), isEmpty);
    expect(interceptors.whereType<RetryInterceptor>(), isEmpty);
    expect(
      rig.client.dio.interceptors.whereType<AuthInterceptor>(),
      hasLength(1),
    );
  });

  group('requestOtp', () {
    test('trả mã yêu cầu và thời hạn', () async {
      final rig = Rig();
      final challenge = await rig.authApi.requestOtp(
        phone: testPhone,
        app: 'customer',
      );
      expect(challenge.requestId, 'otp-000001');
      expect(challenge.expiresIn, const Duration(minutes: 5));
    });

    test('số sai dạng: ClientFailure 422 theo trường', () async {
      final rig = Rig();
      await expectLater(
        rig.authApi.requestOtp(
          phone: 'khong-phai-so-dien-thoai',
          app: 'customer',
        ),
        throwsA(
          _client(422).having((f) => f.fieldErrors.keys, 'fields', ['phone']),
        ),
      );
    });
  });

  group('verifyOtp', () {
    test('mã đúng trả cặp token', () async {
      final rig = Rig();
      final challenge = await rig.authApi.requestOtp(
        phone: testPhone,
        app: 'customer',
      );
      final tokens = await rig.authApi.verifyOtp(
        requestId: challenge.requestId,
        code: fakeOtpCode,
      );
      expect(tokens.accessToken.split('.'), hasLength(3));
      expect(tokens.refreshToken, startsWith('rt_'));
    });

    test('mã sai 422, sai ba lần 429', () async {
      final rig = Rig();
      final challenge = await rig.authApi.requestOtp(
        phone: testPhone,
        app: 'customer',
      );
      Future<Object?> wrong() => rig.authApi
          .verifyOtp(requestId: challenge.requestId, code: '000000')
          .then<Object?>((_) => null, onError: (Object e) => e);
      expect(await wrong(), _client(422));
      expect(await wrong(), _client(422));
      expect(await wrong(), _client(429));
    });
  });

  group('refresh và logout', () {
    test('refresh token lạ: ClientFailure 401', () async {
      final rig = Rig();
      await expectLater(
        rig.authApi.refresh('rt_khong-co'),
        throwsA(_client(401)),
      );
    });

    test('logout trả xong không giá trị; mất mạng là NetworkFailure', () async {
      final rig = Rig(
        wrap: (fake) =>
            (r) => r.url.path == 'auth/logout' ? offline(r) : fake(r),
      );
      await expectLater(
        rig.authApi.logout('rt_x'),
        throwsA(isA<NetworkFailure>()),
      );
      final ok = Rig();
      await ok.authApi.logout('rt_x');
    });

    test('body 200 sai hợp đồng: ParseFailure, không phải lỗi lạ', () async {
      for (final body in [
        '<html>',
        '[]',
        '{"accessToken":"a"}',
        '{"accessToken":"","refreshToken":"r"}',
      ]) {
        final rig = Rig(
          wrap: (fake) =>
              (r) => r.url.path == 'auth/refresh' ? Response.ok(body) : fake(r),
        );
        await expectLater(
          rig.authApi.refresh('rt_x'),
          throwsA(isA<ParseFailure>()),
          reason: body,
        );
      }
    });

    test('refresh quá hạn nhận: TimeoutFailure', () async {
      final rig = Rig(
        wrap: (fake) =>
            (r) => r.url.path == 'auth/refresh'
            ? Completer<Response>().future
            : fake(r),
      );
      final api = AuthApi.create(
        baseUrl: Uri.parse('http://fake.test'),
        adapter: rig.adapter,
        receiveTimeout: const Duration(milliseconds: 20),
      );
      await expectLater(api.refresh('rt_x'), throwsA(isA<TimeoutFailure>()));
      api.dio.close(force: true);
    });
  });

  test('AuthApi.create chuẩn hoá baseUrl có dấu / cuối', () {
    final api = AuthApi.create(baseUrl: Uri.parse('http://fake.test/v1/'));
    expect(api.dio.options.baseUrl, 'http://fake.test/v1');
    expect(api.dio.options, isA<BaseOptions>());
  });
}
