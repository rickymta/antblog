import 'dart:convert';

import 'package:shelf/shelf.dart';
import 'package:test/test.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'support.dart';

const _phone = '+84000000001';

Map<String, Object?> _payloadOf(String jwt) {
  final payload = jwt.split('.')[1];
  return jsonDecode(utf8.decode(base64Url.decode(base64Url.normalize(payload))))
      as Map<String, Object?>;
}

void main() {
  late DateTime now;
  late AuthModule auth;
  late Handler handler;

  Handler build({bool withMiddleware = true, AuthModule? module}) {
    auth = module ?? AuthModule();
    return createFakeApiHandler(
      clock: () => now,
      newId: () => 'T-1',
      modules: [auth],
      middleware: [if (withMiddleware) auth.requireBearer],
    );
  }

  setUp(() {
    now = DateTime.utc(2026, 11, 6, 3, 30);
    handler = build();
  });

  Future<Reply> requestOtp({
    Object? phone = _phone,
    Object? app = 'customer',
  }) => call(
    handler,
    'POST',
    '/auth/otp/request',
    body: {'phone': ?phone, 'app': ?app},
  );

  Future<Reply> verify(String requestId, String code) => call(
    handler,
    'POST',
    '/auth/otp/verify',
    body: {'requestId': requestId, 'code': code},
  );

  /// Đăng nhập bằng OTP đúng; trả body của `verify`.
  Future<Map<String, Object?>> login({
    String phone = _phone,
    String app = 'customer',
  }) async {
    final challenge = asMap(await requestOtp(phone: phone, app: app));
    final reply = await verify(challenge['requestId']! as String, fakeOtpCode);
    expect(reply.status, 200);
    return asMap(reply);
  }

  Future<Reply> refresh(Object? token) =>
      call(handler, 'POST', '/auth/refresh', body: {'refreshToken': ?token});

  Future<Reply> getTrip({String? token}) => call(
    handler,
    'GET',
    '/trips/missing',
    headers: {'authorization': ?token == null ? null : 'Bearer $token'},
  );

  group('POST /auth/otp/request', () {
    test(
      '200 với requestId và thời hạn; không có mã OTP trong response',
      () async {
        final reply = await requestOtp();
        expect(reply.status, 200);
        expect(asMap(reply), {
          'requestId': 'otp-000001',
          'expiresInSeconds': 300,
        });
        expect(jsonEncode(reply.json), isNot(contains(fakeOtpCode)));
      },
    );

    test('số điện thoại sai dạng hoặc app lạ: 422 theo từng trường', () async {
      final bad = await requestOtp(
        phone: 'khong-phai-so-dien-thoai',
        app: 'admin',
      );
      expect(bad.status, 422);
      expect(errorsOf(bad).keys, unorderedEquals(['phone', 'app']));
      expect((await requestOtp(phone: null)).status, 422);
      expect((await requestOtp(phone: 12345)).status, 422);
    });

    test('body không phải JSON: 400', () async {
      final reply = await call(
        handler,
        'POST',
        '/auth/otp/request',
        rawBody: 'khong phai json',
      );
      expect(reply.status, 400);
    });
  });

  group('POST /auth/otp/verify', () {
    test('mã đúng: cặp token, user và hạn dùng', () async {
      final body = await login(app: 'driver');
      expect(body['tokenType'], 'Bearer');
      expect(body['expiresIn'], 900);
      expect(body['user'], {'id': 'U-1001', 'role': 'driver'});
      expect(body['refreshToken'], startsWith('rt_'));
    });

    test(
      'access token là JWT không ký: alg none, chữ ký trống, claim đủ',
      () async {
        final access = (await login())['accessToken']! as String;
        final parts = access.split('.');
        expect(parts, hasLength(3));
        expect(parts[2], isEmpty);
        final header = jsonDecode(
          utf8.decode(base64Url.decode(base64Url.normalize(parts[0]))),
        );
        expect(header, {'alg': 'none', 'typ': 'JWT'});
        final claims = _payloadOf(access);
        final iat = now.millisecondsSinceEpoch ~/ 1000;
        expect(claims['sub'], 'U-1001');
        expect(claims['role'], 'customer');
        expect(claims['iat'], iat);
        expect(claims['exp'], iat + 900);
        expect(claims['jti'], isA<String>());
      },
    );

    test(
      'hai lần đăng nhập cùng số và app: cùng user, token khác nhau',
      () async {
        final a = await login();
        final b = await login();
        expect((a['user']! as Map<String, Object?>)['id'], 'U-1001');
        expect((b['user']! as Map<String, Object?>)['id'], 'U-1001');
        expect(a['accessToken'], isNot(b['accessToken']));
        expect(a['refreshToken'], isNot(b['refreshToken']));
        expect(auth.activeFamilies, 2);
      },
    );

    test('mã sai: 422; lần sai thứ ba: 429 và yêu cầu bị huỷ', () async {
      final id = asMap(await requestOtp())['requestId']! as String;
      expect((await verify(id, '000000')).status, 422);
      expect((await verify(id, '000000')).status, 422);
      expect((await verify(id, '000000')).status, 429);
      // Mã đúng cũng không cứu được nữa.
      final late = await verify(id, fakeOtpCode);
      expect(late.status, 422);
      expect(errorsOf(late)['code'], ['Invalid or expired code']);
    });

    test('mã dùng được đúng một lần', () async {
      final id = asMap(await requestOtp())['requestId']! as String;
      expect((await verify(id, fakeOtpCode)).status, 200);
      expect((await verify(id, fakeOtpCode)).status, 422);
    });

    test('OTP hết hạn sau 5 phút', () async {
      final id = asMap(await requestOtp())['requestId']! as String;
      now = now.add(const Duration(minutes: 5));
      final reply = await verify(id, fakeOtpCode);
      expect(reply.status, 422);
      expect(errorsOf(reply)['code'], ['Invalid or expired code']);
    });

    test('requestId lạ hoặc thiếu trường', () async {
      expect((await verify('otp-999999', fakeOtpCode)).status, 422);
      final reply = await call(
        handler,
        'POST',
        '/auth/otp/verify',
        body: {'code': fakeOtpCode},
      );
      expect(reply.status, 422);
    });

    test('mã OTP và hằng số có thể cấu hình cho từng module', () async {
      handler = build(
        module: AuthModule(
          otpCode: '654321',
          maxOtpAttempts: 1,
          otpTtl: const Duration(minutes: 1),
        ),
      );
      final id = asMap(await requestOtp())['requestId']! as String;
      expect(asMap(await requestOtp())['expiresInSeconds'], 60);
      expect((await verify(id, '000000')).status, 429);
      final id2 = asMap(await requestOtp())['requestId']! as String;
      expect((await verify(id2, '654321')).status, 200);
    });
  });

  group('POST /auth/refresh: xoay vòng', () {
    test('trả cặp token mới, token cũ không dùng lại được', () async {
      final first = await login();
      final reply = await refresh(first['refreshToken']);
      expect(reply.status, 200);
      final second = asMap(reply);
      expect(second['refreshToken'], isNot(first['refreshToken']));
      expect(second['accessToken'], isNot(first['accessToken']));
      expect(second['user'], first['user']);
    });

    test(
      'dùng lại token đã xoay: 401, thu hồi cả họ, token mới cũng hỏng',
      () async {
        final first = await login();
        final second = asMap(await refresh(first['refreshToken']));

        final reuse = await refresh(first['refreshToken']);
        expect(reuse.status, 401);
        expect(asMap(reuse)['title'], 'Refresh token reuse detected');
        expect(auth.revokedFamilies, 1);
        expect(auth.activeFamilies, 0);

        final newest = await refresh(second['refreshToken']);
        expect(newest.status, 401);
        expect(asMap(newest)['title'], 'Refresh token revoked');
      },
    );

    test('thu hồi một họ không đụng tới họ khác', () async {
      final a = await login();
      final b = await login();
      await refresh(a['refreshToken']);
      expect((await refresh(a['refreshToken'])).status, 401);
      expect((await refresh(b['refreshToken'])).status, 200);
      expect(auth.activeFamilies, 1);
    });

    test('token lạ: 401; thiếu trường: 422', () async {
      final unknown = await refresh('rt_khong-co');
      expect(unknown.status, 401);
      expect(asMap(unknown)['title'], 'Invalid refresh token');
      expect((await refresh(null)).status, 422);
      expect((await refresh('')).status, 422);
    });

    test('refresh token hết hạn sau 30 ngày', () async {
      final first = await login();
      now = now.add(const Duration(days: 30));
      final reply = await refresh(first['refreshToken']);
      expect(reply.status, 401);
      expect(asMap(reply)['title'], 'Refresh token expired');
    });

    test('mỗi lần xoay cấp access token mới với hạn mới', () async {
      final first = await login();
      now = now.add(const Duration(minutes: 10));
      final second = asMap(await refresh(first['refreshToken']));
      expect(
        _payloadOf(second['accessToken']! as String)['exp'],
        now.millisecondsSinceEpoch ~/ 1000 + 900,
      );
    });
  });

  group('POST /auth/logout', () {
    test('204 và refresh token của họ đó không dùng được nữa', () async {
      final body = await login();
      final out = await call(
        handler,
        'POST',
        '/auth/logout',
        body: {'refreshToken': body['refreshToken']},
      );
      expect(out.status, 204);
      expect(auth.revokedFamilies, 1);
      final reply = await refresh(body['refreshToken']);
      expect(reply.status, 401);
      expect(asMap(reply)['title'], 'Refresh token revoked');
    });

    test('token lạ vẫn 204 (không để dò token), thiếu trường 422', () async {
      final unknown = await call(
        handler,
        'POST',
        '/auth/logout',
        body: {'refreshToken': 'rt_khong-co'},
      );
      expect(unknown.status, 204);
      final missing = await call(handler, 'POST', '/auth/logout', body: {});
      expect(missing.status, 422);
    });

    test(
      'access token vẫn dùng được tới hết hạn: JWT không bị thu hồi',
      () async {
        final body = await login();
        await call(
          handler,
          'POST',
          '/auth/logout',
          body: {'refreshToken': body['refreshToken']},
        );
        expect(
          (await getTrip(token: body['accessToken']! as String)).status,
          404,
        );
        now = now.add(const Duration(minutes: 15));
        expect(
          (await getTrip(token: body['accessToken']! as String)).status,
          401,
        );
      },
    );
  });

  group('requireBearer', () {
    test(
      'không gắn middleware: /trips vẫn mở như trước (hành vi mặc định)',
      () async {
        handler = build(withMiddleware: false);
        expect((await getTrip()).status, 404);
        final plain = createFakeApiHandler(clock: () => now);
        expect((await call(plain, 'GET', '/trips/missing')).status, 404);
      },
    );

    test('thiếu, sai dạng hoặc lạ: 401 kèm www-authenticate', () async {
      for (final header in [null, 'Basic abc', 'Bearer', 'Bearer rac-roi']) {
        final reply = await call(
          handler,
          'GET',
          '/trips/missing',
          headers: {'authorization': ?header},
        );
        expect(reply.status, 401, reason: '$header');
        expect(reply.headers['www-authenticate'], contains('Bearer'));
      }
    });

    test('token hợp lệ đi qua tới route; hết hạn thì 401', () async {
      final access = (await login())['accessToken']! as String;
      expect((await getTrip(token: access)).status, 404);
      now = now.add(const Duration(minutes: 14, seconds: 59));
      expect((await getTrip(token: access)).status, 404);
      now = now.add(const Duration(seconds: 1));
      expect((await getTrip(token: access)).status, 401);
    });

    test('token bị sửa payload (đổi role) bị từ chối: server không tin payload', () async {
      final access = (await login())['accessToken']! as String;
      final parts = access.split('.');
      final claims = _payloadOf(access)..['role'] = 'driver';
      final forged =
          '${parts[0]}.'
          '${base64Url.encode(utf8.encode(jsonEncode(claims))).replaceAll('=', '')}.';
      expect((await getTrip(token: access)).status, 404);
      expect((await getTrip(token: forged)).status, 401);
    });

    test(
      'chỉ bảo vệ tiền tố khai báo: /health, /quotes, /auth không cần token',
      () async {
        expect((await call(handler, 'GET', '/health')).status, 200);
        final quote = await call(
          handler,
          'POST',
          '/quotes',
          body: {'serviceKind': 'ride', 'distanceKm': 5},
        );
        expect(quote.status, 200);
        expect((await requestOtp()).status, 200);
      },
    );

    test('tiền tố khớp theo đoạn đường dẫn, không theo chuỗi con', () async {
      handler = build(module: AuthModule(protectedPrefixes: {'/trips'}));
      // "/tripsx" không thuộc "/trips": rơi xuống 404 của router, không 401.
      expect((await call(handler, 'GET', '/tripsx')).status, 404);
      expect((await call(handler, 'GET', '/trips')).status, 401);
    });

    test('đặt chuyến với token hợp lệ: 201 và cước 25.500', () async {
      final access = (await login())['accessToken']! as String;
      final reply = await call(
        handler,
        'POST',
        '/trips',
        body: {'serviceKind': 'ride', 'distanceKm': 5},
        headers: {'authorization': 'Bearer $access', 'idempotency-key': 'k-1'},
      );
      expect(reply.status, 201);
      expect(asMap(reply)['fare'], 25500);
    });
  });

  group('tuỳ chọn dòng lệnh', () {
    test('--with-auth mặc định tắt và bật được', () {
      expect(parseServerOptions(const []).withAuth, isFalse);
      expect(parseServerOptions(const ['--with-auth']).withAuth, isTrue);
    });
  });
}
