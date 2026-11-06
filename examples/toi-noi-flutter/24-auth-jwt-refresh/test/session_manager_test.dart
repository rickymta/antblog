import 'dart:async';

import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_24/auth/session_manager.dart';
import 'package:toi_noi_24/auth/token_store.dart';
import 'package:toi_noi_api/toi_noi_api.dart';

import 'support.dart';

/// Đếm số lần đọc kho.
final class _CountingStore extends InMemoryTokenStore {
  int reads = 0;

  @override
  Future<AuthTokens?> read() {
    reads++;
    return super.read();
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  final stores = <String, TokenStore Function()>{
    'InMemoryTokenStore': InMemoryTokenStore.new,
    'SecureTokenStore': () {
      FlutterSecureStorage.setMockInitialValues({});
      return const SecureTokenStore(FlutterSecureStorage());
    },
  };

  for (final MapEntry(key: name, value: create) in stores.entries) {
    group('đăng xuất với $name', () {
      test(
        'xoá kho, xoá bộ nhớ, phát signedOut và thu hồi họ token ở server',
        () async {
          final rig = Rig(tokenStore: create());
          final tokens = await rig.signIn();
          expect(await rig.tokenStore.read(), tokens);

          await rig.session.signOut();
          await pumpEventQueue();

          expect(await rig.tokenStore.read(), isNull);
          expect(await rig.session.read(), isNull);
          expect(rig.signedOutCount, 1);
          final logout = rig.requestsTo('/auth/logout').single;
          expect(logout.body, contains(tokens.refreshToken));
          expect(rig.auth.revokedFamilies, 1);
          // Refresh token cũ không còn dùng được.
          await expectLater(
            rig.authApi.refresh(tokens.refreshToken),
            throwsA(isA<ClientFailure>()),
          );
        },
      );
    });
  }

  group('SessionManager', () {
    test(
      'mất mạng khi báo server: vẫn đăng xuất cục bộ và không ném lỗi',
      () async {
        final rig = Rig(
          wrap: (fake) =>
              (r) => r.url.path == 'auth/logout' ? offline(r) : fake(r),
        );
        await rig.signIn();
        await rig.session.signOut();
        expect(await rig.tokenStore.read(), isNull);
        expect(rig.signedOutCount, 1);
        expect(rig.requestsTo('/auth/logout'), hasLength(1));
      },
    );

    test(
      'đã đăng xuất rồi thì đăng xuất nữa không gọi mạng, không phát sự kiện',
      () async {
        final rig = Rig();
        await rig.session.signOut();
        await rig.signIn();
        await rig.session.signOut();
        await rig.session.signOut();
        expect(rig.requestsTo('/auth/logout'), hasLength(1));
        expect(rig.signedOutCount, 1);
      },
    );

    test('đọc kho đúng một lần dù nhiều nơi cùng hỏi', () async {
      final store = _CountingStore();
      final rig = Rig(tokenStore: store);
      await Future.wait([for (var i = 0; i < 5; i++) rig.session.read()]);
      await rig.session.read();
      expect(store.reads, 1);
    });

    test('mở lại app: đọc phiên cũ từ kho rồi dùng được ngay', () async {
      final store = InMemoryTokenStore();
      final first = Rig(tokenStore: store);
      final tokens = await first.signIn();
      final restarted = Rig(tokenStore: store);
      expect(await restarted.session.read(), tokens);
    });

    test(
      'không có token thì refreshAfterUnauthorized ném SessionExpired',
      () async {
        final rig = Rig();
        await expectLater(
          rig.session.refreshAfterUnauthorized('bat-ky'),
          throwsA(isA<SessionExpired>()),
        );
        expect(rig.refreshCalls, 0);
      },
    );

    test('đăng xuất trong lúc refresh đang bay: kết quả về muộn bị bỏ, phiên không sống lại', () async {
      final release = Completer<void>();
      final rig = Rig(
        wrap: (fake) => (request) async {
          final response = await fake(request); // server đã xoay token
          if (request.url.path == 'auth/refresh') await release.future;
          return response;
        },
      );
      await rig.signIn();
      rig.expireAccessTokens();

      final call = rig.book('k-1');
      final outcome = expectLater(call, throwsA(isA<ClientFailure>()));
      await pumpEventQueue();
      expect(rig.refreshCalls, 1);

      await rig.session.signOut();
      release.complete();
      await outcome;
      await pumpEventQueue();

      expect(await rig.tokenStore.read(), isNull);
      expect(await rig.session.read(), isNull);
      expect(rig.signedOutCount, 1);
    });

    for (final (status, ends) in [
      (401, true),
      (403, true),
      (400, false),
      (422, false),
      (429, false),
    ]) {
      test(
        'refresh trả $status: ${ends ? 'phiên kết thúc' : 'phiên giữ nguyên'}',
        () async {
          final rig = Rig(
            wrap: (fake) =>
                (r) => r.url.path == 'auth/refresh' ? problem(status) : fake(r),
          );
          final tokens = await rig.signIn();
          await expectLater(
            rig.session.refreshAfterUnauthorized(tokens.accessToken),
            throwsA(ends ? isA<SessionExpired>() : isA<ClientFailure>()),
          );
          expect(await rig.tokenStore.read(), ends ? isNull : tokens);
          expect(rig.signedOutCount, ends ? 1 : 0);
        },
      );
    }

    test('ghi kho lỗi khi refresh: phiên chạy tiếp bằng cặp mới trong bộ nhớ, lỗi báo qua onStoreError', () async {
      final store = _FailingSaveStore();
      final errors = <Object>[];
      final rig = Rig(tokenStore: store, onStoreError: errors.add);
      final tokens = await rig.signIn();
      rig.expireAccessTokens();
      store.failSave = true;

      final trip = await rig.book('k-1'); // refresh lỗi ghi kho, vẫn đặt được

      expect(trip.id, isNotEmpty);
      final current = (await rig.session.read())!;
      expect(current, isNot(tokens));
      expect(errors.single, isA<StateError>());
      // Kho còn cặp cũ (đã bị server xoay): lần mở app sau sẽ phải đăng nhập lại.
      expect(await store.read(), tokens);
      // Trong phiên này, lần refresh kế tiếp dùng đúng cặp mới nên không bị coi là dùng lại.
      rig.expireAccessTokens();
      expect((await rig.book('k-2')).id, isNotEmpty);
      expect(rig.auth.revokedFamilies, 0);
    });

    test('đăng xuất đúng lúc đang ghi kho: lệnh ghi đến sau lệnh xoá, kho vẫn phải sạch', () async {
      final store = _GatedSaveStore();
      final rig = Rig(tokenStore: store);
      await rig.signIn();
      rig.expireAccessTokens();
      store.gate = Completer<void>();

      final call = rig.book('k-1');
      final outcome = expectLater(call, throwsA(isA<ClientFailure>()));
      await store.saveStarted.future; // refresh xong, đang ghi kho
      await rig.session.signOut(); // xoá kho trong lúc lệnh ghi còn treo
      store.gate!.complete();
      await outcome;

      expect(await store.read(), isNull);
      expect(await rig.session.read(), isNull);
      expect(rig.store.trips, isEmpty);
      expect(rig.signedOutCount, 1);
    });

    test('đăng nhập lại đúng lúc đang ghi kho: kho giữ phiên mới, không phải phiên cũ', () async {
      final store = _GatedSaveStore();
      final rig = Rig(tokenStore: store);
      await rig.signIn();
      rig.expireAccessTokens();
      store.gate = Completer<void>();

      final call = rig.book('k-1');
      final outcome = expectLater(call, throwsA(isA<ClientFailure>()));
      await store.saveStarted.future;
      store.gate = null; // các lệnh ghi sau đi thẳng
      await rig.session.signOut();
      final second = await rig.signIn();
      store.releaseFirst();
      await outcome;

      expect(await store.read(), second);
      expect(await rig.session.read(), second);
    });

    test('mất response của lần refresh: phiên còn, nhưng lần Thử lại bị coi là dùng lại và đăng xuất', () async {
      var lose = true;
      final rig = Rig(
        wrap: (fake) => (request) async {
          final response = await fake(request); // server đã xoay token
          if (lose && request.url.path == 'auth/refresh') {
            return offline(request);
          }
          return response;
        },
      );
      final old = await rig.signIn();
      rig.expireAccessTokens();

      await expectLater(rig.book('k-1'), throwsA(isA<NetworkFailure>()));
      expect(await rig.tokenStore.read(), old, reason: 'phiên còn');
      expect(rig.signedOutCount, 0);

      lose = false;
      await expectLater(rig.book('k-1'), throwsA(isA<ClientFailure>()));
      expect(rig.auth.revokedFamilies, 1);
      expect(rig.signedOutCount, 1);
      expect(await rig.tokenStore.read(), isNull);
    });
  });
}

/// Kho mà lệnh ghi có thể bị giữ lại: [saveStarted] báo lệnh ghi đầu tiên đã
/// bắt đầu, nó chỉ xong khi [gate] hoàn tất (hoặc [releaseFirst]).
final class _GatedSaveStore extends InMemoryTokenStore {
  Completer<void>? gate;
  final saveStarted = Completer<void>();
  Completer<void>? _first;

  void releaseFirst() => _first?.complete();

  @override
  Future<void> save(AuthTokens tokens) async {
    final held = gate;
    if (held != null) {
      _first = held;
      if (!saveStarted.isCompleted) saveStarted.complete();
      await held.future;
    }
    return super.save(tokens);
  }
}

final class _FailingSaveStore extends InMemoryTokenStore {
  bool failSave = false;

  @override
  Future<void> save(AuthTokens tokens) async {
    if (failSave) throw StateError('không ghi được');
    return super.save(tokens);
  }
}
