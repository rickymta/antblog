import 'dart:async';
import 'dart:convert';

import 'package:dio/dio.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shelf/shelf.dart';
import 'package:toi_noi_24/auth/token_store.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

bool _isTrips(Request request) => request.url.path == 'trips';

void main() {
  group('gắn token', () {
    test(
      'đã đăng nhập: request mang Bearer của access token hiện tại',
      () async {
        final rig = Rig();
        final tokens = await rig.signIn();
        final trip = await rig.book('k-1');
        expect(trip.fare, calculateFare(5));
        expect(rig.bearersTo('/trips'), [tokens.accessToken]);
        expect(rig.refreshCalls, 0);
      },
    );

    test(
      'chưa đăng nhập: không có header, 401 trả nguyên và không refresh',
      () async {
        final rig = Rig();
        await expectLater(
          rig.book('k-1'),
          throwsA(
            isA<ClientFailure>().having((f) => f.statusCode, 'status', 401),
          ),
        );
        expect(rig.bearersTo('/trips'), [null]);
        expect(rig.refreshCalls, 0);
        expect(rig.signedOutCount, 0);
      },
    );

    test('đường dẫn công khai (/quotes) không mang token', () async {
      final rig = Rig();
      await rig.signIn();
      final quote = await rig.client.quote(
        const QuoteRequestDto(serviceKind: ServiceKind.ride, distanceKm: 5),
      );
      expect(quote.toMoney(), calculateFare(5));
      expect(rig.bearersTo('/quotes'), [null]);
    });

    test('lỗi không phải 401 (404) đi qua, không refresh', () async {
      final rig = Rig();
      await rig.signIn();
      await expectLater(
        guardApi(() => rig.client.getTrip('khong-co')),
        throwsA(
          isA<ClientFailure>().having((f) => f.statusCode, 'status', 404),
        ),
      );
      expect(rig.refreshCalls, 0);
    });
  });

  group('401 và refresh', () {
    test('access hết hạn: refresh một lần rồi phát lại, người gọi chỉ thấy thành công', () async {
      final rig = Rig();
      final old = await rig.signIn();
      rig.expireAccessTokens();

      final trip = await rig.book('k-1');

      expect(trip.fare, calculateFare(5));
      expect(rig.refreshCalls, 1);
      final saved = (await rig.tokenStore.read())!;
      expect(saved, isNot(old));
      expect(rig.bearersTo('/trips'), [old.accessToken, saved.accessToken]);
      final body = jsonDecode(
        rig.requestsTo('/auth/refresh').single.body,
      ) as Map<String, Object?>;
      expect(body['refreshToken'], old.refreshToken);
    });

    test('phát lại giữ nguyên method, body và Idempotency-Key; chuyến chỉ tạo một lần', () async {
      final rig = Rig();
      await rig.signIn();
      rig.expireAccessTokens();
      await rig.book('k-1');

      final sent = rig.requestsTo('/trips', method: 'POST');
      expect(sent, hasLength(2));
      expect(sent[1].body, sent[0].body);
      expect(sent[1].headers['idempotency-key'], 'k-1');
      expect(
        sent[0].headers['authorization'],
        isNot(sent[1].headers['authorization']),
      );
      expect(rig.store.trips, hasLength(1));
    });

    test('5 request song song cùng 401 chỉ gọi refresh đúng 1 lần', () async {
      final rig = Rig(
        wrap: (fake) => holdUntil(fake, count: 5, when: _isTrips),
      );
      final old = await rig.signIn();
      rig.expireAccessTokens();

      final trips = await Future.wait([
        for (var i = 0; i < 5; i++) rig.book('k-$i'),
      ]);

      expect(trips.map((t) => t.id).toSet(), hasLength(5));
      expect(rig.refreshCalls, 1);
      final fresh = (await rig.tokenStore.read())!.accessToken;
      final bearers = rig.bearersTo('/trips');
      expect(bearers, hasLength(10));
      expect(bearers.where((t) => t == old.accessToken), hasLength(5));
      expect(bearers.where((t) => t == fresh), hasLength(5));
      expect(rig.auth.activeFamilies, 1);
      expect(rig.auth.revokedFamilies, 0);
      expect(rig.signedOutCount, 0);
    });

    test('401 về muộn khi token đã đổi: phát lại với token mới, không refresh lần hai', () async {
      final release = Completer<void>();
      var held = 0;
      final rig = Rig(
        wrap: (fake) => (request) async {
          if (_isTrips(request) && held++ == 0) await release.future;
          return fake(request);
        },
      );
      final old = await rig.signIn();
      rig.expireAccessTokens();

      final slow = rig.book('slow'); // bị giữ ở server, mang token cũ
      final fast = await rig.book('fast'); // refresh xong trước
      expect(fast.fare, calculateFare(5));
      expect(rig.refreshCalls, 1);

      release.complete();
      expect((await slow).fare, calculateFare(5));

      expect(rig.refreshCalls, 1);
      final fresh = (await rig.tokenStore.read())!.accessToken;
      expect(rig.bearersTo('/trips'), [
        old.accessToken, // slow, lần đầu
        old.accessToken, // fast, lần đầu
        fresh, // fast, phát lại
        fresh, // slow, phát lại (không refresh thêm)
      ]);
    });

    test('401 của phiên cũ về sau khi đã đăng nhập lại: không phát lại bằng token của phiên mới', () async {
      final release = Completer<void>();
      var held = 0;
      final rig = Rig(
        wrap: (fake) => (request) async {
          if (_isTrips(request) && held++ == 0) await release.future;
          return fake(request);
        },
      );
      final a = await rig.signIn();
      final slow = rig.book('slow'); // phiên A, bị giữ ở server
      final outcome = expectLater(
        slow,
        throwsA(
          isA<ClientFailure>().having((f) => f.statusCode, 'status', 401),
        ),
      );
      await pumpEventQueue();
      rig.advance(const Duration(minutes: 10));
      await rig.session.signOut();
      await rig.signIn(); // phiên B
      rig.advance(const Duration(minutes: 6)); // token A hết hạn, token B còn

      release.complete();
      await outcome;

      expect(rig.store.trips, isEmpty, reason: 'không tạo chuyến thay B');
      expect(rig.bearersTo('/trips'), [a.accessToken]);
      expect(rig.refreshCalls, 0);
      expect((await rig.book('next')).fare, calculateFare(5));
    });

    test(
      'phát lại vẫn 401: không lặp vô hạn, chỉ một lần refresh, phiên còn',
      () async {
        final rig = Rig(
          wrap: (fake) =>
              (request) => _isTrips(request) ? problem(401) : fake(request),
        );
        await rig.signIn();
        await expectLater(
          rig.book('k-1'),
          throwsA(
            isA<ClientFailure>().having((f) => f.statusCode, 'status', 401),
          ),
        );
        expect(rig.refreshCalls, 1);
        expect(rig.requestsTo('/trips'), hasLength(2));
        expect(await rig.tokenStore.read(), isNotNull);
        expect(rig.signedOutCount, 0);
      },
    );

    test(
      'đang chờ refresh thì request mới đợi và mang token mới ngay lần đầu',
      () async {
        final release = Completer<void>();
        final rig = Rig(
          wrap: (fake) => (request) async {
            final response = await fake(request);
            if (request.url.path == 'auth/refresh') await release.future;
            return response;
          },
        );
        await rig.signIn();
        rig.expireAccessTokens();

        final first = rig.book('a'); // 401, rồi chờ refresh (đang bị giữ)
        await pumpEventQueue();
        final second = rig.book('b'); // bắt đầu giữa lúc refresh
        await pumpEventQueue();
        expect(rig.requestsTo('/trips'), hasLength(1));

        release.complete();
        await Future.wait([first, second]);
        expect(rig.refreshCalls, 1);
        final fresh = (await rig.tokenStore.read())!.accessToken;
        // a: cũ rồi mới; b: chỉ gửi một lần, và là với token mới.
        expect(rig.bearersTo('/trips').where((t) => t == fresh), hasLength(2));
        expect(rig.requestsTo('/trips'), hasLength(3));
      },
    );

    test('huỷ request trong lúc chờ refresh: không phát lại', () async {
      final release = Completer<void>();
      final rig = Rig(
        wrap: (fake) => (request) async {
          final response = await fake(request);
          if (request.url.path == 'auth/refresh') await release.future;
          return response;
        },
      );
      await rig.signIn();
      rig.expireAccessTokens();
      final token = CancelToken();
      final call = rig.client.dio.post<String>(
        '/trips',
        data: {'serviceKind': 'ride', 'distanceKm': 5},
        options: Options(headers: {'idempotency-key': 'k-1'}),
        cancelToken: token,
      );
      final outcome = expectLater(
        call,
        throwsA(
          isA<DioException>().having(
            (e) => e.type,
            'type',
            DioExceptionType.cancel,
          ),
        ),
      );
      await pumpEventQueue();
      token.cancel();
      release.complete();
      await outcome;
      await pumpEventQueue();
      expect(rig.requestsTo('/trips'), hasLength(1));
    });
  });

  group('refresh thất bại', () {
    test('server từ chối (họ token đã bị thu hồi): đăng xuất, xoá kho, request nhận 401', () async {
      final rig = Rig();
      final tokens = await rig.signIn();
      await rig.authApi.logout(tokens.refreshToken); // thu hồi từ phía khác
      rig.expireAccessTokens();

      await expectLater(
        rig.book('k-1'),
        throwsA(
          isA<ClientFailure>().having((f) => f.statusCode, 'status', 401),
        ),
      );
      expect(await rig.tokenStore.read(), isNull);
      expect(await rig.session.read(), isNull);
      expect(rig.signedOutCount, 1);
      expect(rig.refreshCalls, 1);
    });

    test('5 request song song, refresh bị từ chối: một lần refresh, một lần đăng xuất', () async {
      final rig = Rig(
        wrap: (fake) => holdUntil(fake, count: 5, when: _isTrips),
      );
      final tokens = await rig.signIn();
      await rig.authApi.logout(tokens.refreshToken);
      rig.expireAccessTokens();

      final results = await Future.wait([
        for (var i = 0; i < 5; i++)
          rig.book('k-$i').then<Object>((t) => t, onError: (Object e) => e),
      ]);

      expect(results, everyElement(isA<ClientFailure>()));
      expect(rig.refreshCalls, 1);
      expect(rig.signedOutCount, 1);
      expect(await rig.tokenStore.read(), isNull);
    });

    test(
      'dùng lại refresh token đã xoay: server thu hồi cả họ, app đăng xuất',
      () async {
        final rig = Rig();
        final old = await rig.signIn();
        rig.expireAccessTokens();
        await rig.authApi.refresh(
          old.refreshToken,
        ); // bên khác đã xoay token này

        await expectLater(rig.book('k-1'), throwsA(isA<ClientFailure>()));
        expect(rig.auth.revokedFamilies, 1);
        expect(rig.signedOutCount, 1);
        expect(await rig.tokenStore.read(), isNull);
      },
    );

    for (final (name, handler, matcher)
        in <(String, Handler Function(Handler), Matcher)>[
          (
            '5xx',
            (Handler h) =>
                (r) => r.url.path == 'auth/refresh' ? problem(500) : h(r),
            isA<ServerFailure>(),
          ),
          (
            'mất mạng',
            (Handler h) =>
                (r) => r.url.path == 'auth/refresh' ? offline(r) : h(r),
            isA<NetworkFailure>(),
          ),
          (
            '429',
            (Handler h) =>
                (r) => r.url.path == 'auth/refresh' ? problem(429) : h(r),
            isA<ClientFailure>().having((f) => f.statusCode, 'status', 429),
          ),
        ]) {
      test(
        'refresh gặp $name: giữ phiên, request nhận lỗi thật, không thử lại refresh',
        () async {
          var broken = true;
          final rig = Rig(
            retry: RetryPlacement.afterAuth,
            wrap: (fake) {
              final faulty = handler(fake);
              return (r) => broken ? faulty(r) : fake(r);
            },
          );
          final old = await rig.signIn();
          rig.expireAccessTokens();

          await expectLater(rig.book('k-1'), throwsA(matcher));
          expect(await rig.tokenStore.read(), old);
          expect(rig.signedOutCount, 0);
          expect(rig.refreshCalls, 1);

          // Hết sự cố: lần sau refresh thành công và request đi tiếp.
          broken = false;
          expect((await rig.book('k-1')).fare, calculateFare(5));
          expect(rig.refreshCalls, 2);
        },
      );
    }
  });

  group('thứ tự với RetryInterceptor', () {
    for (final placement in [
      RetryPlacement.afterAuth,
      RetryPlacement.beforeAuth,
    ]) {
      group(placement.name, () {
        test('5 request song song cùng 401: vẫn một lần refresh', () async {
          final rig = Rig(
            retry: placement,
            wrap: (fake) => holdUntil(fake, count: 5, when: _isTrips),
          );
          await rig.signIn();
          rig.expireAccessTokens();
          final trips = await Future.wait([
            for (var i = 0; i < 5; i++) rig.book('k-$i'),
          ]);
          expect(trips, hasLength(5));
          expect(rig.refreshCalls, 1);
          expect(rig.waits, isEmpty); // 401 không bao giờ bị Retry thử lại
        });

        test(
          '503 rồi 401: Retry thử lại, lần thử gặp 401 thì refresh một lần',
          () async {
            var first = true;
            final rig = Rig(
              retry: placement,
              wrap: (fake) => (request) {
                if (_isTrips(request) && first) {
                  first = false;
                  return problem(503);
                }
                return fake(request);
              },
            );
            await rig.signIn();
            rig.expireAccessTokens();

            final trip = await rig.book('k-1');

            expect(trip.fare, calculateFare(5));
            expect(rig.refreshCalls, 1);
            expect(rig.waits, [const Duration(milliseconds: 500)]);
            final paths = [for (final r in rig.adapter.requests) r.uri.path];
            expect(paths.sublist(paths.length - 4), [
              '/trips', // 503
              '/trips', // thử lại: 401
              '/auth/refresh',
              '/trips', // phát lại: 201
            ]);
            expect(rig.store.trips, hasLength(1));
          },
        );

        test('phát lại sau refresh gặp 503: Retry vẫn thử lại', () async {
          var served = 0;
          final rig = Rig(
            retry: placement,
            wrap: (fake) => (request) {
              // Lần thứ hai tới /trips (phát lại sau refresh) bị 503.
              if (_isTrips(request) && ++served == 2) return problem(503);
              return fake(request);
            },
          );
          await rig.signIn();
          rig.expireAccessTokens();
          expect((await rig.book('k-1')).fare, calculateFare(5));
          expect(rig.refreshCalls, 1);
          expect(rig.waits, hasLength(1));
        });
      });
    }

    test(
      'log thấy 401 trước khi AuthInterceptor xử lý, rồi từng lần phát lại',
      () async {
        final rig = Rig(retry: RetryPlacement.afterAuth);
        await rig.signIn();
        rig.logs.clear();
        rig.expireAccessTokens();
        await rig.book('k-1');

        final compact = [
          for (final line in rig.logs)
            line.replaceAll(RegExp(r' \(\d+ ms\).*| \{.*'), ''),
        ];
        expect(compact, [
          '--> POST /trips',
          '<-- ERROR badResponse 401 POST /trips',
          '--> POST /auth/refresh',
          '<-- 200 POST /auth/refresh',
          '--> POST /trips',
          '<-- 201 POST /trips',
        ]);
        // Log không bao giờ chứa giá trị token.
        final tokens = (await rig.tokenStore.read())!;
        final all = rig.logs.join('\n');
        expect(all, isNot(contains(tokens.accessToken)));
        expect(all, isNot(contains(tokens.refreshToken)));
      },
    );
  });

  test('sau khi đăng nhập lại, request dùng token của phiên mới', () async {
    final rig = Rig();
    final first = await rig.signIn();
    await rig.session.signOut();
    final second = await rig.signIn();
    await rig.book('k-1');
    expect(rig.bearersTo('/trips'), [second.accessToken]);
    expect(second, isNot(first));
    expect(rig.refreshCalls, 0);
  });

  test(
    'token mới được lưu vào kho thật (SecureTokenStore) trước khi dùng',
    () async {
      TestWidgetsFlutterBinding.ensureInitialized();
      FlutterSecureStorage.setMockInitialValues({});
      final rig = Rig(
        tokenStore: const SecureTokenStore(FlutterSecureStorage()),
      );
      await rig.signIn();
      rig.expireAccessTokens();
      await rig.book('k-1');
      final all = await const FlutterSecureStorage().readAll();
      expect(all.keys, [SecureTokenStore.storageKey]);
      final saved =
          jsonDecode(all[SecureTokenStore.storageKey]!) as Map<String, Object?>;
      expect(saved['access'], rig.bearersTo('/trips').last);
    },
  );
}
