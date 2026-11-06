import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_36/realtime_client.dart';

import 'support.dart';

void main() {
  group('close', () {
    test('đóng kết nối bằng mã 1000, closed(client), không nối lại, không còn Timer', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = buildClient(transport);
        client.connect();
        client.subscribe('T-1');
        settle(async);

        unawaited(client.close());
        settle(async);
        async.elapse(const Duration(hours: 1));

        expect(transport.last.closedByClient, isTrue);
        expect(transport.last.clientCloseCode, 1000);
        expect(
          client.state,
          const RealtimeState(RealtimeStatus.closed, cause: CloseCause.client),
        );
        expect(transport.attempts, 1);
        expect(async.pendingTimers, isEmpty);
      });
    });

    test('close khi đang chờ nối lại huỷ luôn Timer chờ', () {
      fakeAsync((async) {
        final transport = FakeTransport()..failures = 5;
        final client = buildClient(transport);
        client.connect();
        settle(async);
        expect(client.state.status, RealtimeStatus.reconnecting);

        unawaited(client.close());
        async.elapse(const Duration(hours: 1));

        expect(transport.attempts, 1);
        expect(async.pendingTimers, isEmpty);
      });
    });

    test('close khi đang bắt tay: kết nối tới muộn bị đóng ngay, không rò', () {
      fakeAsync((async) {
        final gate = Completer<void>();
        final transport = FakeTransport()..gate = gate;
        final client = buildClient(transport);
        client.connect();
        settle(async);
        expect(client.state.status, RealtimeStatus.connecting);

        unawaited(client.close());
        gate.complete(); // bắt tay xong sau khi đã close
        settle(async);

        expect(transport.last.closedByClient, isTrue);
        expect(client.state.cause, CloseCause.client);
        expect(async.pendingTimers, isEmpty);
      });
    });

    test('đóng rồi connect lại: nghe lại mọi chuyến từ sequence đã nhận', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = buildClient(transport);
        client.connect();
        client.subscribe('T-1');
        settle(async);
        transport.last.serverSend(eventFrame('T-1', 1));
        settle(async);

        unawaited(client.close());
        settle(async);
        client.connect();
        settle(async);

        expect(transport.last.sentOfType('subscribe'), [
          {'type': 'subscribe', 'tripId': 'T-1', 'lastSequence': 1},
        ]);
      });
    });
  });

  group('dispose', () {
    test('đóng kết nối, huỷ mọi Timer và đóng các stream', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = buildClient(transport);
        var statesDone = false;
        var eventsDone = false;
        client.states.listen((_) {}, onDone: () => statesDone = true);
        client.events.listen((_) {}, onDone: () => eventsDone = true);
        client.connect();
        client.subscribe('T-1');
        settle(async);
        transport.last.serverSend(locationFrame('T-1', 10.1));
        settle(async);
        expect(async.pendingTimers, isNotEmpty); // heartbeat, ổn định, cửa sổ

        unawaited(client.dispose());
        settle(async);

        expect(transport.last.closedByClient, isTrue);
        expect(async.pendingTimers, isEmpty);
        expect(statesDone, isTrue);
        expect(eventsDone, isTrue);
      });
    });

    test('dispose khi đang bắt tay: không rò kết nối, không phát trạng thái sau khi đóng stream', () {
      fakeAsync((async) {
        final gate = Completer<void>();
        final transport = FakeTransport()..gate = gate;
        final client = buildClient(transport);
        final states = collect(client.states);
        client.connect();
        settle(async);

        unawaited(client.dispose());
        settle(async);
        final seen = states.length;
        gate.complete();
        settle(async);
        async.elapse(const Duration(hours: 1));

        expect(transport.last.closedByClient, isTrue);
        expect(states, hasLength(seen));
        expect(async.pendingTimers, isEmpty);
      });
    });

    test('dispose() rồi connect() ngay (chưa await): lỗi lập trình, không có kết nối hay Timer nào sống', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = buildClient(transport);

        unawaited(client.dispose());
        expect(client.connect, throwsStateError);
        settle(async);
        async.elapse(const Duration(hours: 1));

        expect(transport.attempts, 0);
        expect(async.pendingTimers, isEmpty);
      });
    });

    test('dispose() khi đang connected rồi connect() ngay: không mở kết nối thứ hai', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = buildClient(transport);
        client.connect();
        settle(async);

        unawaited(client.dispose());
        expect(client.connect, throwsStateError);
        settle(async);

        expect(transport.connections, hasLength(1));
        expect(transport.last.closedByClient, isTrue);
        expect(async.pendingTimers, isEmpty);
      });
    });

    test('dispose hai lần không sao; dùng sau dispose là lỗi lập trình', () {
      fakeAsync((async) {
        final client = buildClient(FakeTransport());

        unawaited(client.dispose());
        unawaited(client.dispose());
        settle(async);

        expect(client.connect, throwsStateError);
        expect(() => client.subscribe('T-1'), throwsStateError);
      });
    });
  });
}
