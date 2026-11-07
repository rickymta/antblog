import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_36/realtime_client.dart';

import 'support.dart';

const _s = Duration(seconds: 1);

void main() {
  group('heartbeat', () {
    test('ping đều đặn theo heartbeatInterval, id tăng dần', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = buildClient(transport);
        client.connect();
        settle(async);

        async.elapse(const Duration(seconds: 14));
        expect(transport.last.sentOfType('ping'), isEmpty);
        async.elapse(_s);
        expect(transport.last.sentOfType('ping'), [
          {'type': 'ping', 'id': 'p-1'},
        ]);

        transport.last.serverSend({'type': 'pong', 'id': 'p-1'});
        async.elapse(const Duration(seconds: 15));
        expect(transport.last.sentOfType('ping').last['id'], 'p-2');
      });
    });

    test('gateway trả pong kịp: kết nối sống mãi, không nối lại', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = buildClient(transport);
        client.connect();
        settle(async);

        for (var i = 0; i < 20; i++) {
          async.elapse(const Duration(seconds: 15));
          transport.last.serverSend({'type': 'pong', 'id': 'p-${i + 1}'});
          async.elapse(const Duration(milliseconds: 100));
        }

        expect(client.state.status, RealtimeStatus.connected);
        expect(transport.attempts, 1);
      });
    });

    test('gateway im lặng sau ping: đúng heartbeatTimeout thì coi là chết, đóng và nối lại', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = buildClient(transport);
        client.connect();
        settle(async);
        final first = transport.last;

        async.elapse(const Duration(seconds: 15)); // ping gửi đi
        async.elapse(
          const Duration(seconds: 5) - const Duration(milliseconds: 1),
        );
        expect(client.state.status, RealtimeStatus.connected);

        async.elapse(const Duration(milliseconds: 1));
        expect(client.state.status, RealtimeStatus.reconnecting);
        expect(first.closedByClient, isTrue);

        async.elapse(client.state.nextRetryIn!);
        expect(transport.connections, hasLength(2));
        expect(client.state.status, RealtimeStatus.connected);
      });
    });

    test('frame nào cũng chứng tỏ còn sống, không chỉ pong', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = buildClient(transport);
        client.connect();
        client.subscribe('T-1');
        settle(async);

        async.elapse(const Duration(seconds: 15)); // ping
        async.elapse(const Duration(seconds: 2));
        transport.last.serverSend(
          locationFrame('T-1', 10.7),
        ); // không phải pong
        async.elapse(const Duration(seconds: 10));

        expect(client.state.status, RealtimeStatus.connected);
        expect(transport.attempts, 1);
      });
    });

    test('im lặng trước ping đầu tiên chưa phải chết', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = buildClient(transport);
        client.connect();
        settle(async);

        async.elapse(const Duration(seconds: 14, milliseconds: 999));

        expect(client.state.status, RealtimeStatus.connected);
      });
    });

    test('kết nối đã bị coi là chết thì frame muộn của nó bị bỏ', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = buildClient(transport);
        final events = collect(client.events);
        client.connect();
        client.subscribe('T-1');
        settle(async);
        final zombie = transport.last;

        async.elapse(const Duration(seconds: 20)); // ping không ai trả lời
        expect(zombie.hasListener, isFalse);
        zombie.serverSend(eventFrame('T-1', 1));
        settle(async);

        expect(events, isEmpty);
      });
    });

    test('mất kết nối thì heartbeat dừng: không ping vào kết nối cũ', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = buildClient(transport);
        client.connect();
        settle(async);
        final old = transport.last;

        old.serverDrop();
        settle(async);
        async.elapse(const Duration(minutes: 1));

        expect(old.sentOfType('ping'), isEmpty);
      });
    });
  });
}
