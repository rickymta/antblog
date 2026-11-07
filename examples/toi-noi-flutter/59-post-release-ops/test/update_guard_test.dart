import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_59/app_version.dart';
import 'package:toi_noi_59/force_update_screen.dart';
import 'package:toi_noi_59/reminder_throttle.dart';
import 'package:toi_noi_59/store_platform.dart';
import 'package:toi_noi_59/update_gate.dart';
import 'package:toi_noi_59/update_guard.dart';

import 'support.dart';

Map<String, Object?> policy({String message = 'Bản mới đã sẵn sàng'}) => {
  'android': {
    'minSupported': '1.4.0+60',
    'recommended': '1.6.0+87',
    'latestPublished': '1.6.0+87',
    'message': message,
  },
};

void main() {
  late FakeOpener opener;
  late MemoryReminderStore store;
  late FakeClock clock;

  setUp(() {
    opener = FakeOpener();
    store = MemoryReminderStore();
    clock = FakeClock(DateTime.utc(2026, 11, 7, 9));
  });

  UpdateGuard guard(PolicySource source, String current, Widget child) =>
      UpdateGuard(
        gate: UpdateGate(
          source: source,
          throttle: ReminderThrottle(store: store, clock: clock.call),
        ),
        current: AppVersion.parse(current),
        platform: StorePlatform.android,
        opener: opener,
        child: child,
      );

  // Cách dùng đúng: guard bao cả Navigator qua MaterialApp.builder.
  Widget app(PolicySource source, String current) => MaterialApp(
    builder: (context, child) => guard(source, current, child!),
    home: const Scaffold(body: Text('Màn hình chính')),
  );

  Widget homeWithPush(Widget Function(Widget home) wrap) => MaterialApp(
    builder: null,
    home: wrap(
      Builder(
        builder: (context) => Scaffold(
          body: TextButton(
            onPressed: () => Navigator.of(context).push(
              MaterialPageRoute<void>(
                builder: (_) => const Scaffold(body: Text('Trang đã push')),
              ),
            ),
            child: const Text('Mở'),
          ),
        ),
      ),
    ),
  );

  testWidgets('trong lúc chờ server người dùng đã thấy app', (tester) async {
    await tester.pumpWidget(app(FakePolicySource.hanging(), '1.0.0+1'));

    expect(find.text('Màn hình chính'), findsOneWidget);
    expect(find.byType(ForceUpdateScreen), findsNothing);

    // Hết 3 giây chờ: vẫn là app, không có lỗi, không còn Timer treo.
    await tester.pump(const Duration(seconds: 3));
    expect(find.text('Màn hình chính'), findsOneWidget);
  });

  testWidgets('server lỗi thì vẫn vào app dù bản rất cũ', (tester) async {
    await tester.pumpWidget(
      app(FakePolicySource.throwing(StateError('503')), '1.0.0+1'),
    );
    await tester.pump();

    expect(find.text('Màn hình chính'), findsOneWidget);
    expect(find.byType(ForceUpdateScreen), findsNothing);
  });

  testWidgets('bản dưới minSupported bị thay bằng màn hình chặn', (
    tester,
  ) async {
    await tester.pumpWidget(app(FakePolicySource.value(policy()), '1.3.0+10'));
    await tester.pump();

    expect(find.byType(ForceUpdateScreen), findsOneWidget);
    expect(find.text('Màn hình chính'), findsNothing);
    expect(find.text('Bản mới đã sẵn sàng'), findsOneWidget);
  });

  testWidgets('màn hình chặn mở link do chính sách (đã kiểm) cung cấp', (
    tester,
  ) async {
    await tester.pumpWidget(app(FakePolicySource.value(policy()), '1.3.0+10'));
    await tester.pump();
    await tester.tap(find.text('Mở cửa hàng'));
    await tester.pump();

    expect(opener.opened.single.host, 'play.google.com');
    expect(opener.opened.single.scheme, 'https');
  });

  testWidgets('recommend hiện banner phía trên, app vẫn dùng được', (
    tester,
  ) async {
    await tester.pumpWidget(app(FakePolicySource.value(policy()), '1.5.0+70'));
    await tester.pump();

    expect(find.text('Bản mới đã sẵn sàng'), findsOneWidget);
    expect(find.text('Màn hình chính'), findsOneWidget);
    expect(find.byType(ForceUpdateScreen), findsNothing);
  });

  testWidgets('"Để sau" ẩn banner; "Cập nhật" mở cửa hàng', (tester) async {
    await tester.pumpWidget(app(FakePolicySource.value(policy()), '1.5.0+70'));
    await tester.pump();

    await tester.tap(find.text('Cập nhật'));
    await tester.pump();
    expect(opener.opened, hasLength(1));

    await tester.tap(find.text('Để sau'));
    await tester.pump();
    expect(find.text('Bản mới đã sẵn sàng'), findsNothing);
    expect(find.text('Màn hình chính'), findsOneWidget);
  });

  testWidgets('lỗi khi mở link từ banner không làm văng app', (tester) async {
    opener.error = StateError('lỗi');
    await tester.pumpWidget(app(FakePolicySource.value(policy()), '1.5.0+70'));
    await tester.pump();

    await tester.tap(find.text('Cập nhật'));
    await tester.pump();

    expect(tester.takeException(), isNull);
  });

  testWidgets('mở lại app trong 3 ngày: không còn banner', (tester) async {
    await tester.pumpWidget(app(FakePolicySource.value(policy()), '1.5.0+70'));
    await tester.pump();
    expect(find.text('Bản mới đã sẵn sàng'), findsOneWidget);

    // "Mở lại app": dựng một cây mới, bộ nhớ nhắc vẫn là store cũ.
    await tester.pumpWidget(const SizedBox());
    clock.advance(const Duration(days: 1));
    await tester.pumpWidget(app(FakePolicySource.value(policy()), '1.5.0+70'));
    await tester.pump();

    expect(find.text('Bản mới đã sẵn sàng'), findsNothing);
    expect(find.text('Màn hình chính'), findsOneWidget);
  });

  testWidgets('widget bị gỡ trước khi server trả lời thì không lỗi', (
    tester,
  ) async {
    await tester.pumpWidget(app(FakePolicySource.value(policy()), '1.3.0+10'));
    await tester.pumpWidget(const SizedBox());
    await tester.pump();

    expect(tester.takeException(), isNull);
  });

  group('vị trí đặt guard', () {
    final forcePolicy = policy();

    testWidgets(
      'ở builder: route push lúc đang chờ cũng bị chặn khi có force',
      (tester) async {
        final source = FakePolicySource.manual();
        await tester.pumpWidget(
          MaterialApp(
            builder: (context, child) => guard(source, '1.3.0+10', child!),
            home: Builder(
              builder: (context) => Scaffold(
                body: TextButton(
                  onPressed: () => Navigator.of(context).push(
                    MaterialPageRoute<void>(
                      builder: (_) =>
                          const Scaffold(body: Text('Trang đã push')),
                    ),
                  ),
                  child: const Text('Mở'),
                ),
              ),
            ),
          ),
        );
        await tester.tap(find.text('Mở'));
        await tester.pumpAndSettle();
        expect(find.text('Trang đã push'), findsOneWidget);

        source.complete(forcePolicy);
        await tester.pump(); // gate xong, gọi setState
        await tester.pump(); // dựng lại cây

        expect(find.byType(ForceUpdateScreen), findsOneWidget);
        expect(find.text('Trang đã push', skipOffstage: false), findsNothing);
        expect(find.text('Mở', skipOffstage: false), findsNothing);
      },
    );

    testWidgets('đối chứng, ở home: route push lúc đang chờ vẫn dùng được', (
      tester,
    ) async {
      final source = FakePolicySource.manual();
      await tester.pumpWidget(
        homeWithPush((home) => guard(source, '1.3.0+10', home)),
      );
      await tester.tap(find.text('Mở'));
      await tester.pumpAndSettle();

      source.complete(forcePolicy);
      await tester.pump(); // gate xong, gọi setState
      await tester.pump(); // dựng lại cây

      // Màn hình chặn có trong cây nhưng nằm dưới route đã push.
      expect(
        find.byType(ForceUpdateScreen, skipOffstage: false),
        findsOneWidget,
      );
      expect(find.text('Trang đã push'), findsOneWidget);
    });

    testWidgets('ở builder: Back không có route để đóng, hệ thống xử lý Back', (
      tester,
    ) async {
      final calls = <String>[];
      tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
        SystemChannels.platform,
        (call) async {
          calls.add(call.method);
          return null;
        },
      );
      addTearDown(
        () => tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
          SystemChannels.platform,
          null,
        ),
      );
      await tester.pumpWidget(
        app(FakePolicySource.value(forcePolicy), '1.3.0+10'),
      );
      await tester.pump();
      await tester.pump();

      await tester.binding.handlePopRoute();
      await tester.pump();

      expect(calls, contains('SystemNavigator.pop'));
      expect(find.byType(ForceUpdateScreen), findsOneWidget);
      expect(find.text('Màn hình chính', skipOffstage: false), findsNothing);
    });
  });
}
