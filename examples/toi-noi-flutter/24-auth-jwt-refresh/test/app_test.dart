import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_24/app.dart';
import 'package:toi_noi_24/auth/token_store.dart';
import 'package:toi_noi_24/auth/unverified_claims.dart';
import 'package:toi_noi_24/login_screen.dart';
import 'package:toi_noi_24/providers.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'support.dart';

Future<void> _pumpApp(WidgetTester tester, Rig rig) async {
  var keys = 0;
  await tester.pumpWidget(
    ProviderScope(
      retry: (_, _) => null,
      overrides: [
        tokenStoreProvider.overrideWithValue(rig.tokenStore),
        httpAdapterProvider.overrideWithValue(rig.adapter),
        apiBaseUrlProvider.overrideWithValue(Uri.parse('http://fake.test')),
        logSinkProvider.overrideWithValue(rig.logs.add),
        idempotencyKeyProvider.overrideWithValue(() => 'book-${++keys}'),
        retrySleepProvider.overrideWithValue((_) async {}),
      ],
      child: const ToiNoiApp(),
    ),
  );
  await tester.pumpAndSettle();
}

Future<void> _login(WidgetTester tester, {String code = fakeOtpCode}) async {
  await tester.tap(find.byKey(const Key('request-otp')));
  await tester.pumpAndSettle();
  await tester.enterText(find.byKey(const Key('code')), code);
  await tester.tap(find.byKey(const Key('verify-otp')));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('chưa có phiên: màn hình đăng nhập, chưa gọi mạng', (
    tester,
  ) async {
    final rig = Rig();
    await _pumpApp(tester, rig);
    expect(find.byType(LoginScreen), findsOneWidget);
    expect(rig.adapter.requests, isEmpty);
  });

  testWidgets('đăng nhập OTP đúng: vào màn hình chính, token nằm trong kho', (
    tester,
  ) async {
    final rig = Rig();
    await _pumpApp(tester, rig);
    await _login(tester);

    expect(find.byType(LoginScreen), findsNothing);
    expect(
      find.textContaining('Tài khoản U-1001 (vai trò customer'),
      findsOneWidget,
    );
    expect(await rig.tokenStore.read(), isNotNull);
    expect(rig.requestsTo('/auth/otp/request'), hasLength(1));
  });

  testWidgets(
    'mã sai: báo lỗi tiếng Việt, ba lần sai thì quay về bước nhập số',
    (tester) async {
      final rig = Rig();
      await _pumpApp(tester, rig);
      await _login(tester, code: '000000');
      expect(find.text('Mã không đúng hoặc đã hết hạn.'), findsOneWidget);
      expect(find.byKey(const Key('code')), findsOneWidget);
      expect(find.textContaining('Invalid'), findsNothing);

      await tester.tap(find.byKey(const Key('verify-otp')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('verify-otp')));
      await tester.pumpAndSettle();

      expect(find.textContaining('sai quá số lần'), findsOneWidget);
      expect(find.byKey(const Key('request-otp')), findsOneWidget);
      expect(await rig.tokenStore.read(), isNull);
    },
  );

  testWidgets('số điện thoại sai dạng: báo đúng chỗ sai', (tester) async {
    final rig = Rig();
    await _pumpApp(tester, rig);
    await tester.enterText(
      find.byKey(const Key('phone')),
      'khong-phai-so-dien-thoai',
    );
    await tester.tap(find.byKey(const Key('request-otp')));
    await tester.pumpAndSettle();
    expect(find.textContaining('định dạng quốc tế'), findsOneWidget);
  });

  testWidgets('đặt chuyến sau đăng nhập: 25.500 VND', (tester) async {
    final rig = Rig();
    await _pumpApp(tester, rig);
    await _login(tester);
    await tester.tap(find.byKey(const Key('book')));
    await tester.pumpAndSettle();
    expect(find.text('25.500 VND'), findsOneWidget);
    expect(rig.refreshCalls, 0);
  });

  testWidgets(
    'access token hết hạn lúc đặt chuyến: refresh âm thầm, người dùng vẫn thấy giá',
    (tester) async {
      final rig = Rig();
      await _pumpApp(tester, rig);
      await _login(tester);
      rig.expireAccessTokens();

      await tester.tap(find.byKey(const Key('book')));
      await tester.pumpAndSettle();

      expect(find.text('25.500 VND'), findsOneWidget);
      expect(find.byKey(const Key('booking-error')), findsNothing);
      expect(rig.refreshCalls, 1);
    },
  );

  testWidgets(
    'đăng xuất: về màn hình đăng nhập, kho sạch, server thu hồi họ token',
    (tester) async {
      final rig = Rig();
      await _pumpApp(tester, rig);
      await _login(tester);

      await tester.tap(find.byKey(const Key('sign-out')));
      await tester.pumpAndSettle();

      expect(find.byType(LoginScreen), findsOneWidget);
      expect(await rig.tokenStore.read(), isNull);
      expect(rig.auth.revokedFamilies, 1);
    },
  );

  testWidgets(
    'mở lại app với phiên trong kho: vào thẳng màn hình chính, không gọi mạng',
    (tester) async {
      final rig = Rig();
      // Gọi mạng của test, nên chạy ngoài đồng hồ giả của testWidgets.
      await tester.runAsync(rig.signIn);
      final before = rig.adapter.requests.length;
      await _pumpApp(tester, rig);
      expect(find.byKey(const Key('who')), findsOneWidget);
      expect(find.byType(LoginScreen), findsNothing);
      expect(rig.adapter.requests, hasLength(before));
    },
  );

  testWidgets(
    'server thu hồi phiên giữa chừng: refresh bị từ chối, app tự về màn hình đăng nhập',
    (tester) async {
      final rig = Rig();
      await _pumpApp(tester, rig);
      await _login(tester);
      final tokens = (await rig.tokenStore.read())!;
      // thu hồi từ thiết bị khác; gọi mạng thật của test nên chạy ngoài đồng hồ giả
      await tester.runAsync(() => rig.authApi.logout(tokens.refreshToken));
      rig.expireAccessTokens();

      await tester.tap(find.byKey(const Key('book')));
      await tester.pumpAndSettle();

      expect(find.byType(LoginScreen), findsOneWidget);
      expect(await rig.tokenStore.read(), isNull);
    },
  );

  testWidgets(
    'mất mạng khi refresh: ở lại màn hình chính, phiên còn, báo chưa đặt được',
    (tester) async {
      var offlineNow = false;
      final rig = Rig(
        wrap: (fake) =>
            (r) => offlineNow && r.url.path == 'auth/refresh'
            ? offline(r)
            : fake(r),
      );
      await _pumpApp(tester, rig);
      await _login(tester);
      rig.expireAccessTokens();
      offlineNow = true;

      await tester.tap(find.byKey(const Key('book')));
      await tester.pumpAndSettle();

      expect(find.byType(LoginScreen), findsNothing);
      expect(find.byKey(const Key('booking-error')), findsOneWidget);
      expect(await rig.tokenStore.read(), isNotNull);
      expect(find.textContaining('Network'), findsNothing);
      expect(rig.signedOutCount, 0);
    },
  );

  testWidgets(
    'role trong token chỉ để hiển thị: token bị sửa thành driver bị server từ chối',
    (tester) async {
      final rig = Rig();
      final real = (await tester.runAsync(rig.signIn))!;
      // Kẻ sửa payload để đổi role thành driver; server không đọc payload.
      final parts = real.accessToken.split('.');
      final claims = jsonDecode(
        utf8.decode(base64Url.decode(base64Url.normalize(parts[1]))),
      ) as Map<String, Object?>;
      claims['role'] = 'driver';
      final payload = base64Url
          .encode(utf8.encode(jsonEncode(claims)))
          .replaceAll('=', '');
      final forged = '${parts[0]}.$payload.${parts[2]}';
      await rig.tokenStore.save(
        AuthTokens(accessToken: forged, refreshToken: real.refreshToken),
      );

      await _pumpApp(tester, rig);
      // Client đọc ra đúng thứ nó được đưa: driver.
      expect(find.textContaining('vai trò driver'), findsOneWidget);

      await tester.tap(find.byKey(const Key('book')));
      await tester.pumpAndSettle();

      // Server từ chối token giả (401), app đổi lấy token thật rồi phát lại.
      expect(rig.refreshCalls, 1);
      expect(find.text('25.500 VND'), findsOneWidget);
      final saved = (await rig.tokenStore.read())!;
      expect(UnverifiedClaims.tryParse(saved.accessToken)!.role, 'customer');
    },
  );
}
