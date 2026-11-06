import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:toi_noi_23/auth.dart';

import 'support.dart';

Future<void> signInViaForm(WidgetTester tester) async {
  await tester.tap(find.byKey(const Key('sign-in')));
  await tester.pumpAndSettle();
}

void main() {
  group('guard đăng nhập', () {
    testWidgets('chưa đăng nhập vào /history: sang /login, nhớ from', (
      tester,
    ) async {
      final c = await pumpApp(tester);
      c.router.go('/history');
      await tester.pumpAndSettle();

      expect(find.text('Đăng nhập'), findsWidgets);
      expect(find.byKey(const Key('sign-in')), findsOneWidget);
      expect(locationOf(c.router), '/login?from=%2Fhistory');
    });

    testWidgets(
      'đăng nhập xong: tự quay lại from, không cần màn hình login làm gì',
      (tester) async {
        final c = await pumpApp(tester);
        c.router.go('/history');
        await tester.pumpAndSettle();

        await signInViaForm(tester);

        expect(locationOf(c.router), '/history');
        expect(find.text('T-1001 · Đặt xe'), findsOneWidget);
        expect(find.byKey(const Key('sign-in')), findsNothing);
      },
    );

    testWidgets('from có query được giữ nguyên sau đăng nhập', (tester) async {
      final c = await pumpApp(tester);
      c.router.go('/trip/T-1002?ref=push');
      await tester.pumpAndSettle();
      expect(locationOf(c.router), '/login?from=%2Ftrip%2FT-1002%3Fref%3Dpush');

      await signInViaForm(tester);
      expect(locationOf(c.router), '/trip/T-1002?ref=push');
      expect(find.text('Chuyến T-1002'), findsOneWidget);
    });

    testWidgets('from giả mạo trỏ ra ngoài: sau đăng nhập về trang chủ', (
      tester,
    ) async {
      final c = await pumpApp(tester);
      c.router.go('/login?from=%2F%2Fevil.example%2Ftrip%2F1');
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('sign-in')), findsOneWidget);

      await signInViaForm(tester);
      expect(locationOf(c.router), '/');
      expect(find.text('Đặt chuyến'), findsOneWidget);
    });

    testWidgets('đã đăng nhập mà mở /login: đẩy về trang chủ', (tester) async {
      final c = await pumpApp(tester, signedInAs: alice);
      c.router.go('/login');
      await tester.pumpAndSettle();
      expect(locationOf(c.router), '/');
      expect(find.byKey(const Key('sign-in')), findsNothing);
    });

    testWidgets('đăng xuất khi đang ở /account: redirect đưa về /login', (
      tester,
    ) async {
      final c = await pumpApp(tester, signedInAs: alice);
      c.router.go('/account');
      await tester.pumpAndSettle();
      expect(find.text('Đang đăng nhập: An'), findsOneWidget);

      await tester.tap(find.byKey(const Key('sign-out')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('sign-in')), findsOneWidget);
      expect(locationOf(c.router), '/login?from=%2Faccount');
    });

    testWidgets('đăng xuất rồi đăng nhập lại: về đúng /account', (
      tester,
    ) async {
      final c = await pumpApp(tester, signedInAs: alice);
      c.router.go('/account');
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('sign-out')));
      await tester.pumpAndSettle();

      await signInViaForm(tester);
      expect(locationOf(c.router), '/account');
    });

    testWidgets('đăng xuất khi trang riêng đang hiện từ push: không còn lộ', (
      tester,
    ) async {
      final c = await pumpApp(tester, signedInAs: alice);
      c.router.push('/trip/T-1001');
      await tester.pumpAndSettle();
      expect(find.text('Chuyến T-1001'), findsOneWidget);
      expect(locationOf(c.router), '/trip/T-1001');

      await c.auth.signOut();
      await tester.pumpAndSettle();

      expect(find.text('Chuyến T-1001'), findsNothing);
      expect(find.byKey(const Key('sign-in')), findsOneWidget);
      expect(locationOf(c.router), '/login?from=%2Ftrip%2FT-1001');

      await signInViaForm(tester);
      expect(locationOf(c.router), '/trip/T-1001');
      expect(find.text('Chuyến T-1001'), findsOneWidget);
    });

    testWidgets('chỉ refreshListenable thì trang push vẫn còn sau đăng xuất', (
      tester,
    ) async {
      // Bằng chứng cho giới hạn: redirect chạy với vị trí khai báo (/), nên
      // không thấy trang /secret đang nằm trên cùng.
      final signedIn = ValueNotifier(true);
      final router = GoRouter(
        refreshListenable: signedIn,
        redirect: (_, state) =>
            !signedIn.value &&
                state.uri.path != '/login' &&
                state.uri.path != '/'
            ? '/login'
            : null,
        routes: [
          GoRoute(path: '/', builder: (_, _) => const Text('home')),
          GoRoute(path: '/login', builder: (_, _) => const Text('login')),
          GoRoute(path: '/secret', builder: (_, _) => const Text('secret')),
        ],
      );
      addTearDown(() {
        router.dispose();
        signedIn.dispose();
      });
      await tester.pumpWidget(MaterialApp.router(routerConfig: router));
      router.push('/secret');
      await tester.pumpAndSettle();

      signedIn.value = false;
      await tester.pumpAndSettle();
      expect(find.text('secret'), findsOneWidget, reason: 'vẫn lộ');
      expect(router.state.uri.path, '/secret');
    });

    testWidgets(
      'tab Lịch sử khi chưa đăng nhập: bị chặn, trang chủ vẫn mở được',
      (tester) async {
        final c = await pumpApp(tester);
        expect(locationOf(c.router), '/');

        await tester.tap(tab('Lịch sử'));
        await tester.pumpAndSettle();
        expect(locationOf(c.router), '/login?from=%2Fhistory');
      },
    );

    testWidgets('nút đặt chuyến khi chưa đăng nhập dẫn tới /login', (
      tester,
    ) async {
      final c = await pumpApp(tester);
      expect(find.text('Đăng nhập để đặt chuyến'), findsOneWidget);

      await tester.tap(find.byKey(const Key('place-trip')));
      await tester.pumpAndSettle();
      expect(locationOf(c.router), '/login');
    });
  });

  group('phiên khởi động (redirect async)', () {
    testWidgets(
      'đang đọc kho phiên: chưa dựng màn hình nào, xong thì vào đúng nơi',
      (tester) async {
        launchWithLink(tester, Uri.parse('/history'));
        await tester.pumpWidget(
          pumpAppWidget(
            signedInAs: alice,
            restoreLatency: const Duration(seconds: 2),
          ),
        );
        await tester.pump(const Duration(seconds: 1));
        expect(
          find.byType(Scaffold),
          findsNothing,
          reason: 'redirect còn chờ kho phiên, router chưa có route để dựng',
        );

        await tester.pump(const Duration(seconds: 2));
        await tester.pumpAndSettle();
        expect(find.text('T-1001 · Đặt xe'), findsOneWidget);
        expect(find.byKey(const Key('sign-in')), findsNothing);
      },
    );

    testWidgets('kho có phiên: khởi động ở /history không phải đăng nhập lại', (
      tester,
    ) async {
      launchWithLink(tester, Uri.parse('/history'));
      final c = await pumpApp(tester, signedInAs: alice);
      expect(locationOf(c.router), '/history');
    });

    testWidgets('khởi động ở trang chủ không phải chờ kho phiên', (
      tester,
    ) async {
      await tester.pumpWidget(
        pumpAppWidget(restoreLatency: const Duration(seconds: 30)),
      );
      await tester.pump();
      final button = find.byKey(const Key('place-trip'));
      expect(button, findsOneWidget, reason: 'trang chủ đã dựng');
      expect(
        tester.widget<FilledButton>(button).onPressed,
        isNull,
        reason: 'chưa biết đã đăng nhập hay chưa nên chưa cho bấm',
      );

      await tester.pump(const Duration(seconds: 31));
      expect(tester.widget<FilledButton>(button).onPressed, isNotNull);
    });

    testWidgets('kho phiên hỏng: coi như chưa đăng nhập, không treo app', (
      tester,
    ) async {
      launchWithLink(tester, Uri.parse('/history'));
      final c = await pumpApp(tester, signedInAs: alice, storeFails: true);
      expect(locationOf(c.router), '/login?from=%2Fhistory');
      expect(c.read(authProvider).value, isNull);
    });
  });
}
