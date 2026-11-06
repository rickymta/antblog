import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:toi_noi_37/app.dart';
import 'package:toi_noi_37/push_message.dart';
import 'package:toi_noi_37/push_navigator.dart';
import 'package:toi_noi_37/push_registrar.dart';
import 'package:toi_noi_37/push_router.dart';

import 'support.dart';

void main() {
  late FakePushSource source;
  late FakeLocalNotifier notifier;
  late FakeDeviceApi api;
  late PushRegistrar registrar;
  late GoRouter router;
  late PushNavigator navigator;

  /// Gọi ở đầu mỗi test: nếu dựng trong setUp thì StreamController nằm ngoài
  /// vùng FakeAsync của testWidgets, microtask của chúng không chạy theo pump.
  void build() {
    source = FakePushSource(fidOnRegister: 'fid-1');
    notifier = FakeLocalNotifier();
    api = FakeDeviceApi();
    registrar = PushRegistrar(
      source: source,
      api: api,
      deviceId: 'dev-A',
      platform: 'android',
    );
    router = buildRouter(registrar: registrar);
    navigator = PushNavigator(
      source: source,
      notifier: notifier,
      open: router.go,
    );
  }

  tearDown(() async {
    await navigator.dispose();
    router.dispose();
  });

  Future<void> pumpApp(WidgetTester tester) async {
    await tester.pumpWidget(ToiNoiPushApp(router: router));
    await tester.pumpAndSettle();
  }

  final tripBody = find.byKey(const Key('trip-body'));

  group('ba trạng thái tới đúng màn hình chuyến', () {
    testWidgets('terminated: getInitialMessage mở chuyến, Back về Trang chủ', (
      tester,
    ) async {
      build();
      source.initialMessage = tripMessage('T-1002');
      await pumpApp(tester);
      await navigator.start();
      await tester.pumpAndSettle();

      expect(find.text('Chuyến T-1002'), findsOneWidget);

      router.pop();
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('enable-push')), findsOneWidget);
    });

    testWidgets('background: chạm thông báo mở chuyến', (tester) async {
      build();
      await pumpApp(tester);
      await navigator.start();

      source.tapFromBackground(tripMessage('T-2001'));
      await tester.pumpAndSettle();
      expect(find.text('Chuyến T-2001'), findsOneWidget);
    });

    testWidgets(
      'foreground: tin tới không đổi màn hình, chạm thông báo cục bộ mới đổi',
      (tester) async {
        build();
        await pumpApp(tester);
        await navigator.start();

        source.emitForeground(tripMessage('T-3001'));
        await tester.pumpAndSettle();
        expect(tripBody, findsNothing);
        expect(notifier.shown, hasLength(1));

        notifier.tap(notifier.shown.single.payload);
        await tester.pumpAndSettle();
        expect(find.text('Chuyến T-3001'), findsOneWidget);
      },
    );
  });

  testWidgets('payload lạ: ở lại Trang chủ', (tester) async {
    build();
    await pumpApp(tester);
    await navigator.start();

    source.tapFromBackground(
      const PushMessage(data: {'type': 'trip_update', 'tripId': '../admin'}),
    );
    await tester.pumpAndSettle();
    expect(tripBody, findsNothing);
    expect(find.byKey(const Key('enable-push')), findsOneWidget);
  });

  test('đường dẫn mà PushRouter sinh ra khớp một route đã khai báo', () {
    build();
    const decision = OpenTrip('T-1002');
    final match = router.configuration.findMatch(Uri.parse(decision.location));
    expect(match.isError, isFalse);
  });

  group('màn hình Trang chủ', () {
    testWidgets('bấm Bật thông báo: trạng thái chuyển sang đã bật', (
      tester,
    ) async {
      build();
      await pumpApp(tester);
      expect(find.text('Thông báo: tắt'), findsOneWidget);

      await tester.tap(find.byKey(const Key('enable-push')));
      await tester.pumpAndSettle();

      expect(find.text('Thông báo: đã bật'), findsOneWidget);
      expect(api.upserts.single.pushId, 'fid-1');
    });

    for (final answer in [
      PushPermission.denied,
      PushPermission.deniedForever,
    ]) {
      testWidgets('$answer: chỉ đường tới Cài đặt, không đăng ký', (
        tester,
      ) async {
        build();
        source.nextPermissionAnswer = answer;
        await pumpApp(tester);

        await tester.tap(find.byKey(const Key('enable-push')));
        await tester.pumpAndSettle();

        expect(find.byKey(const Key('push-denied-hint')), findsOneWidget);
        expect(find.text('Thông báo: tắt'), findsOneWidget);
        expect(api.calls, isEmpty);
      });
    }

    testWidgets('bấm Đăng xuất: backend xoá, FCM huỷ đăng ký, trạng thái tắt', (
      tester,
    ) async {
      build();
      await pumpApp(tester);
      await tester.tap(find.byKey(const Key('enable-push')));
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('sign-out')));
      // Đã thử: chỉ pump thì signOut dừng sau bước huỷ subscription (backend
      // chưa nhận remove). Cho event loop thật chạy một lượt thì xong.
      await tester.runAsync(pumpEventQueue);
      await tester.pumpAndSettle();

      expect(api.calls, ['upsert:fid-1', 'remove:dev-A']);
      expect(source.calls.last, 'unregister');
      expect(find.text('Thông báo: tắt'), findsOneWidget);
    });
  });
}
