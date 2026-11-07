import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_36/realtime_client.dart';
import 'package:toi_noi_36/realtime_lifecycle.dart';

import 'support.dart';

void main() {
  late FakeTransport transport;
  late RealtimeClient client;
  late RealtimeLifecycleBinding binding;

  Future<void> start(WidgetTester tester) async {
    transport = FakeTransport();
    client = buildClient(
      transport,
      heartbeatInterval: const Duration(hours: 1),
    );
    binding = RealtimeLifecycleBinding(client, binding: tester.binding);
    client.connect();
    client.subscribe('T-1');
    await tester.pump();
    transport.last.serverSend(eventFrame('T-1', 1));
    await tester.pump();
    expect(client.state.status, RealtimeStatus.connected);
  }

  Future<void> stop(WidgetTester tester) async {
    binding.dispose();
    await client.dispose();
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    await tester.pump();
  }

  Future<void> lifecycle(WidgetTester tester, AppLifecycleState state) async {
    tester.binding.handleAppLifecycleStateChanged(state);
    await tester.pump();
  }

  testWidgets(
    'paused ngắt kết nối; resumed nối lại và subscribe tiếp từ sequence cuối',
    (tester) async {
      await start(tester);

      await lifecycle(tester, AppLifecycleState.paused);
      expect(
        client.state,
        const RealtimeState(
          RealtimeStatus.closed,
          cause: CloseCause.background,
        ),
      );
      expect(transport.last.closedByClient, isTrue);
      expect(transport.last.clientCloseCode, 1000);

      await lifecycle(tester, AppLifecycleState.resumed);
      expect(client.state.status, RealtimeStatus.connected);
      expect(transport.connections, hasLength(2));
      expect(transport.last.sentOfType('subscribe'), [
        {'type': 'subscribe', 'tripId': 'T-1', 'lastSequence': 1},
      ]);
      await stop(tester);
    },
  );

  testWidgets('inactive và hidden là thoáng qua: không ngắt', (tester) async {
    await start(tester);

    await lifecycle(tester, AppLifecycleState.inactive);
    await lifecycle(tester, AppLifecycleState.hidden);

    expect(client.state.status, RealtimeStatus.connected);
    expect(transport.connections, hasLength(1));
    await stop(tester);
  });

  testWidgets('người dùng đã close thì resumed không tự nối lại', (
    tester,
  ) async {
    await start(tester);
    await client.close();

    await lifecycle(tester, AppLifecycleState.paused);
    await lifecycle(tester, AppLifecycleState.resumed);

    expect(client.state.cause, CloseCause.client);
    expect(transport.connections, hasLength(1));
    await stop(tester);
  });

  testWidgets('đã bị từ chối token thì resumed không nối lại', (tester) async {
    await start(tester);
    transport.last.serverClose(4401);
    await tester.pump();
    expect(client.state.cause, CloseCause.unauthorized);

    await lifecycle(tester, AppLifecycleState.paused);
    await lifecycle(tester, AppLifecycleState.resumed);

    expect(client.state.cause, CloseCause.unauthorized);
    expect(transport.connections, hasLength(1));
    await stop(tester);
  });

  testWidgets('pause khi đang chờ nối lại: huỷ Timer chờ, resume nối ngay', (
    tester,
  ) async {
    await start(tester);
    transport.last.serverDrop();
    await tester.pump();
    expect(client.state.status, RealtimeStatus.reconnecting);

    await lifecycle(tester, AppLifecycleState.paused);
    await tester.pump(const Duration(minutes: 5));
    expect(transport.connections, hasLength(1)); // không nối lại lúc ở nền

    await lifecycle(tester, AppLifecycleState.resumed);
    expect(client.state.status, RealtimeStatus.connected);
    expect(transport.connections, hasLength(2));
    await stop(tester);
  });

  testWidgets(
    'sau binding.dispose, vòng đời app không còn tác động tới client',
    (tester) async {
      await start(tester);
      binding.dispose();

      await lifecycle(tester, AppLifecycleState.paused);

      expect(client.state.status, RealtimeStatus.connected);
      await stop(tester);
    },
  );
}
