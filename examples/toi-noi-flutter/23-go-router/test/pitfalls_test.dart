import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:toi_noi_23/auth.dart';
import 'package:toi_noi_23/deep_links.dart';
import 'package:toi_noi_23/router.dart';

import 'support.dart';

/// Router tối giản để chứng minh từng lỗi thường gặp.
Widget _harness(GoRouter router) => MaterialApp.router(routerConfig: router);

void main() {
  group('lỗi 1: quên ghép lại link custom scheme', () {
    testWidgets(
      'router không có bước resolve: toinoi://trip/T-1002 không khớp route',
      (tester) async {
        Object? error;
        final router = GoRouter(
          routes: [
            GoRoute(path: '/', builder: (_, _) => const Text('home')),
            GoRoute(
              path: '/trip/:id',
              builder: (_, state) => Text('trip ${state.pathParameters['id']}'),
            ),
          ],
          errorBuilder: (_, state) {
            error = state.error;
            return const Text('lỗi router');
          },
        );
        addTearDown(router.dispose);
        await tester.pumpWidget(_harness(router));
        await receiveLink(tester, DeepLinks.customTrip('T-1002'));

        expect(find.text('lỗi router'), findsOneWidget);
        expect(error.toString(), contains('no routes for location'));
        expect(
          error.toString(),
          contains('toinoi://trip/T-1002'),
          reason: 'router thấy path /T-1002 nhưng báo cả URI gốc',
        );
      },
    );

    testWidgets('cùng link, nhưng có bước resolve ở redirect: khớp /trip/:id', (
      tester,
    ) async {
      final router = GoRouter(
        redirect: (_, state) => DeepLinks.resolve(state.uri),
        routes: [
          GoRoute(path: '/', builder: (_, _) => const Text('home')),
          GoRoute(
            path: '/trip/:id',
            builder: (_, state) => Text('trip ${state.pathParameters['id']}'),
          ),
        ],
      );
      addTearDown(router.dispose);
      await tester.pumpWidget(_harness(router));
      await receiveLink(tester, DeepLinks.customTrip('T-1002'));
      expect(find.text('trip T-1002'), findsOneWidget);
    });

    testWidgets(
      'universal link thì không cần resolve: go_router chỉ khớp theo path',
      (tester) async {
        final router = GoRouter(
          routes: [
            GoRoute(path: '/', builder: (_, _) => const Text('home')),
            GoRoute(
              path: '/trip/:id',
              builder: (_, state) => Text('trip ${state.pathParameters['id']}'),
            ),
          ],
        );
        addTearDown(router.dispose);
        await tester.pumpWidget(_harness(router));
        await receiveLink(tester, DeepLinks.webTrip('T-1002'));
        expect(find.text('trip T-1002'), findsOneWidget);
      },
    );
  });

  group('lỗi 2: quên refreshListenable', () {
    Future<(GoRouter, ValueNotifier<bool>)> pumpGuarded(
      WidgetTester tester, {
      required bool withRefresh,
    }) async {
      final signedIn = ValueNotifier(false);
      final router = GoRouter(
        refreshListenable: withRefresh ? signedIn : null,
        redirect: (_, state) {
          return switch (state.uri.path) {
            '/secret' when !signedIn.value => '/login',
            '/login' when signedIn.value => '/secret',
            _ => null,
          };
        },
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
      await tester.pumpWidget(_harness(router));
      router.go('/secret');
      await tester.pumpAndSettle();
      expect(find.text('login'), findsOneWidget);
      return (router, signedIn);
    }

    testWidgets(
      'không có: đăng nhập xong vẫn kẹt ở /login tới khi có lần điều hướng sau',
      (tester) async {
        final (router, signedIn) = await pumpGuarded(
          tester,
          withRefresh: false,
        );
        signedIn.value = true;
        await tester.pumpAndSettle();
        expect(
          find.text('login'),
          findsOneWidget,
          reason: 'redirect chưa chạy lại',
        );

        router.refresh();
        await tester.pumpAndSettle();
        expect(find.text('secret'), findsOneWidget);
      },
    );

    testWidgets('có: redirect chạy lại ngay khi trạng thái đổi', (
      tester,
    ) async {
      final (_, signedIn) = await pumpGuarded(tester, withRefresh: true);
      signedIn.value = true;
      await tester.pumpAndSettle();
      expect(find.text('secret'), findsOneWidget);
    });
  });

  group('lỗi 3: ref.watch(authProvider) trong provider của router', () {
    final badRouterProvider = Provider<GoRouter>((ref) {
      final session = ref.watch(authProvider).value; // sai: dựng lại router
      final router = GoRouter(
        redirect: (_, state) =>
            session == null && state.uri.path != '/login' ? '/login' : null,
        routes: [
          GoRoute(path: '/', builder: (_, _) => const Text('home')),
          GoRoute(path: '/login', builder: (_, _) => const Text('login')),
        ],
      );
      ref.onDispose(router.dispose);
      return router;
    });

    test(
      'đổi trạng thái đăng nhập tạo ra GoRouter khác (mất ngăn xếp)',
      () async {
        final container = ProviderContainer(
          overrides: [
            sessionStoreProvider.overrideWithValue(InMemorySessionStore()),
          ],
        );
        addTearDown(container.dispose);
        container.listen(badRouterProvider, (_, _) {});
        await container.read(authProvider.future);

        final before = container.read(badRouterProvider);
        await container.read(authProvider.notifier).signIn('An');
        final after = container.read(badRouterProvider);
        expect(identical(before, after), isFalse);
      },
    );

    test('routerProvider của lab giữ nguyên một GoRouter qua đăng nhập và đăng xuất', () async {
      final container = ProviderContainer(
        overrides: [
          sessionStoreProvider.overrideWithValue(InMemorySessionStore()),
        ],
      );
      addTearDown(container.dispose);
      container.listen(routerProvider, (_, _) {});
      await container.read(authProvider.future);

      final before = container.read(routerProvider);
      await container.read(authProvider.notifier).signIn('An');
      await container.read(authProvider.notifier).signOut();
      expect(identical(container.read(routerProvider), before), isTrue);
    });
  });

  group('lỗi 4: chờ authProvider.future ngay trong lúc state vừa đổi', () {
    test('future trả giá trị cũ trong listener, state thì đã mới', () async {
      final container = ProviderContainer(
        overrides: [
          sessionStoreProvider.overrideWithValue(InMemorySessionStore()),
        ],
      );
      addTearDown(container.dispose);
      await container.read(authProvider.future);

      Future<Session?>? futureSeenInListener;
      Session? stateSeenInListener;
      container.listen(authProvider, (_, next) {
        stateSeenInListener = next.value;
        futureSeenInListener = container.read(authProvider.future);
      });

      await container.read(authProvider.notifier).signIn('An');

      expect(stateSeenInListener, const Session(userName: 'An'));
      expect(
        await futureSeenInListener,
        isNull,
        reason:
            'refreshListenable chạy đúng lúc này: đọc .future sẽ thấy phiên cũ',
      );
      expect(await container.read(authProvider.future), isNotNull);
    });
  });
}
