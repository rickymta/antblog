import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_36/realtime_client.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

/// Tắt heartbeat: gateway giả của các test này không trả lời ping.
RealtimeClient _client(FakeTransport transport) =>
    buildClient(transport, heartbeatInterval: const Duration(hours: 1));

void main() {
  group('subscribe', () {
    test(
      'gọi trước khi connected: ghi nhớ, gửi ngay sau welcome kèm lastSequence',
      () {
        fakeAsync((async) {
          final transport = FakeTransport();
          final client = _client(transport);

          client
            ..subscribe('T-1')
            ..subscribe('T-2', lastSequence: 5)
            ..connect();
          settle(async);

          expect(transport.last.sentOfType('subscribe'), [
            {'type': 'subscribe', 'tripId': 'T-1', 'lastSequence': 0},
            {'type': 'subscribe', 'tripId': 'T-2', 'lastSequence': 5},
          ]);
        });
      },
    );

    test('gọi khi đã connected: gửi ngay; gọi lại cho chuyến đang nghe thì không gửi thêm', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = _client(transport);
        client.connect();
        settle(async);

        client
          ..subscribe('T-1')
          ..subscribe('T-1');

        expect(transport.last.sentOfType('subscribe'), hasLength(1));
      });
    });

    test(
      'unsubscribe: gửi frame, quên sequence; frame muộn của chuyến đó bị bỏ',
      () {
        fakeAsync((async) {
          final transport = FakeTransport();
          final client = _client(transport);
          final events = collect(client.events);
          client.connect();
          client.subscribe('T-1');
          settle(async);
          transport.last.serverSend(eventFrame('T-1', 1));
          settle(async);
          expect(client.lastSequence('T-1'), 1);

          client.unsubscribe('T-1');
          transport.last.serverSend(eventFrame('T-1', 2)); // đã trên đường tới
          settle(async);

          expect(transport.last.sentOfType('unsubscribe'), hasLength(1));
          expect(client.lastSequence('T-1'), isNull);
          expect(events.map((e) => e.sequence), [1]);
        });
      },
    );
  });

  group('sự kiện bền: đúng thứ tự, không trùng, không lỗ hổng', () {
    test('đưa ra theo thứ tự và nhớ sequence cuối', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = _client(transport);
        final events = collect(client.events);
        client.connect();
        client.subscribe('T-1');
        settle(async);

        transport.last
          ..serverSend(eventFrame('T-1', 1))
          ..serverSend(
            eventFrame(
              'T-1',
              2,
              status: 'accepted',
              event: {'type': 'driverAccepted', 'driverId': 'D-7'},
            ),
          );
        settle(async);

        expect(events.map((e) => e.sequence), [1, 2]);
        expect(events.last.status, TripStatus.accepted);
        expect(events.last.event, const DriverAccepted('D-7'));
        expect(client.lastSequence('T-1'), 2);
      });
    });

    test('mất kết nối rồi nối lại: subscribe lại từ sequence cuối, mỗi sự kiện ra đúng một lần', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = _client(transport);
        final events = collect(client.events);
        client.connect();
        client.subscribe('T-1');
        settle(async);
        transport.last
          ..serverSend(eventFrame('T-1', 1))
          ..serverSend(eventFrame('T-1', 2));
        settle(async);

        transport.last.serverDrop();
        settle(async);
        async.elapse(client.state.nextRetryIn!);

        // Kết nối mới xin lại từ sự kiện cuối đã nhận.
        expect(transport.connections, hasLength(2));
        expect(transport.last.sentOfType('subscribe'), [
          {'type': 'subscribe', 'tripId': 'T-1', 'lastSequence': 2},
        ]);

        // Gateway lỡ phát lại cả 2 (hoặc mạng phát trùng): bị khử; 3 ra một lần.
        transport.last
          ..serverSend(eventFrame('T-1', 2))
          ..serverSend(eventFrame('T-1', 3))
          ..serverSend(eventFrame('T-1', 3));
        settle(async);

        expect(events.map((e) => e.sequence), [1, 2, 3]);
        expect(client.duplicatesDropped, 2);
      });
    });

    test('thiếu sự kiện giữa chừng (1 rồi 3): không đưa 3 ra, xin lại từ 1, nhận 2 và 3', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = _client(transport);
        final events = collect(client.events);
        client.connect();
        client.subscribe('T-1');
        settle(async);

        transport.last
          ..serverSend(eventFrame('T-1', 1))
          ..serverSend(eventFrame('T-1', 3))
          ..serverSend(
            eventFrame('T-1', 4),
          ); // dồn dập: vẫn chỉ xin lại một lần
        settle(async);

        expect(events.map((e) => e.sequence), [1]);
        expect(transport.last.sentOfType('subscribe').skip(1), [
          {'type': 'subscribe', 'tripId': 'T-1', 'lastSequence': 1},
        ]);

        transport.last
          ..serverSend(eventFrame('T-1', 2))
          ..serverSend(eventFrame('T-1', 3))
          ..serverSend(eventFrame('T-1', 4));
        settle(async);

        expect(events.map((e) => e.sequence), [1, 2, 3, 4]);
      });
    });

    test('mỗi chuyến có sequence riêng', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = _client(transport);
        final events = collect(client.events);
        client.connect();
        client
          ..subscribe('T-1')
          ..subscribe('T-2');
        settle(async);

        transport.last
          ..serverSend(eventFrame('T-1', 1))
          ..serverSend(eventFrame('T-2', 1))
          ..serverSend(eventFrame('T-2', 2))
          ..serverSend(eventFrame('T-1', 1)); // trùng của T-1
        settle(async);

        expect(events.map((e) => '${e.tripId}#${e.sequence}'), [
          'T-1#1',
          'T-2#1',
          'T-2#2',
        ]);
        expect(client.duplicatesDropped, 1);
      });
    });

    test('subscribed báo latestSequence nhỏ hơn đã nhận: log_reset, thôi nghe, không đưa sự kiện của log mới', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = _client(transport);
        final issues = collect(client.issues);
        final events = collect(client.events);
        client.connect();
        client.subscribe('T-1', lastSequence: 4);
        settle(async);

        transport.last
          ..serverSend(subscribedFrame('T-1', latest: 1))
          ..serverSend(eventFrame('T-1', 2)) // thuộc log khác
          ..serverSend(eventFrame('T-1', 5));
        settle(async);

        expect(issues.single.code, 'log_reset');
        expect(issues.single.tripId, 'T-1');
        expect(client.lastSequence('T-1'), isNull);
        expect(events, isEmpty);
        expect(transport.last.sentOfType('unsubscribe'), hasLength(1));

        // Nối lại: chuyến đã bị bỏ, không xin lại.
        transport.last.serverDrop();
        settle(async);
        async.elapse(client.state.nextRetryIn!);
        expect(transport.last.sentOfType('subscribe'), isEmpty);
      });
    });

    test('logEpoch đổi dù log mới đã dài hơn (last=2, log mới tới 5): vẫn log_reset, không phát nhầm 3, 4, 5', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = _client(transport);
        final issues = collect(client.issues);
        final events = collect(client.events);
        client.connect();
        client.subscribe('T-1', lastSequence: 2, logEpoch: 'epoch-cu');
        settle(async);

        transport.last
          ..serverSend(
            subscribedFrame(
              'T-1',
              latest: 5,
              replayed: 3,
              logEpoch: 'epoch-moi',
            ),
          )
          ..serverSend(eventFrame('T-1', 3))
          ..serverSend(eventFrame('T-1', 4))
          ..serverSend(eventFrame('T-1', 5));
        settle(async);

        expect(issues.map((i) => i.code), ['log_reset']);
        expect(events, isEmpty);
        expect(client.lastSequence('T-1'), isNull);
      });
    });

    test('log_reset do gateway báo bằng error: thôi nghe và không xin lại khi nối lại', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = _client(transport);
        final issues = collect(client.issues);
        client.connect();
        client.subscribe('T-1', lastSequence: 2, logEpoch: 'epoch-cu');
        settle(async);

        transport.last.serverSend({
          'type': 'error',
          'code': 'log_reset',
          'message': 'The client sequence belongs to another log',
          'tripId': 'T-1',
        });
        settle(async);
        transport.last.serverDrop();
        settle(async);
        async.elapse(client.state.nextRetryIn!);

        expect(issues.single.code, 'log_reset');
        expect(transport.last.sentOfType('subscribe'), isEmpty);
      });
    });

    test('app subscribe lại sau log_reset: một lượt nghe mới, bắt đầu từ lastSequence mới', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = _client(transport);
        final events = collect(client.events);
        client.connect();
        client.subscribe('T-1', lastSequence: 4);
        settle(async);
        transport.last.serverSend(subscribedFrame('T-1', latest: 1));
        settle(async);

        client.subscribe('T-1'); // sau khi tải lại bản chụp
        settle(async);
        transport.last
          ..serverSend(subscribedFrame('T-1', latest: 2, replayed: 2))
          ..serverSend(eventFrame('T-1', 1))
          ..serverSend(eventFrame('T-1', 2));
        settle(async);

        expect(events.map((e) => e.sequence), [1, 2]);
        expect(client.logEpoch('T-1'), 'epoch-1');
      });
    });

    test('nối lại gửi kèm logEpoch đã biết từ frame subscribed', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = _client(transport);
        client.connect();
        client.subscribe('T-1');
        settle(async);
        transport.last.serverSend(subscribedFrame('T-1', logEpoch: 'epoch-7'));
        settle(async);

        transport.last.serverDrop();
        settle(async);
        async.elapse(client.state.nextRetryIn!);

        expect(transport.last.sentOfType('subscribe'), [
          {
            'type': 'subscribe',
            'tripId': 'T-1',
            'lastSequence': 0,
            'logEpoch': 'epoch-7',
          },
        ]);
      });
    });

    test('gateway báo replay_unavailable: thôi nghe, báo issue', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = _client(transport);
        final issues = collect(client.issues);
        client.connect();
        client.subscribe('T-1', lastSequence: 1);
        settle(async);

        transport.last.serverSend({
          'type': 'error',
          'code': 'replay_unavailable',
          'message': 'Events after 1 were trimmed from the log',
          'tripId': 'T-1',
        });
        settle(async);

        expect(issues.single.code, 'replay_unavailable');
        expect(client.lastSequence('T-1'), isNull);
      });
    });

    test('xin lại để lấp lỗ hổng mà gateway trả lời rồi vẫn thiếu: replay_unavailable, không nuốt âm thầm', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = _client(transport);
        final issues = collect(client.issues);
        final events = collect(client.events);
        client.connect();
        client.subscribe('T-1');
        settle(async);

        transport.last
          ..serverSend(eventFrame('T-1', 1))
          ..serverSend(eventFrame('T-1', 3)); // thiếu 2: client xin lại từ 1
        settle(async);
        transport.last
          ..serverSend(subscribedFrame('T-1', latest: 4, replayed: 2))
          ..serverSend(eventFrame('T-1', 4)); // gateway bỏ qua 2 và 3
        settle(async);

        expect(events.map((e) => e.sequence), [1]);
        expect(issues.single.code, 'replay_unavailable');
        expect(client.lastSequence('T-1'), isNull);
        expect(transport.last.sentOfType('unsubscribe'), hasLength(1));
      });
    });

    test('sự kiện tới khi chưa có ai listen events được đệm, không mất', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = _client(transport);
        client.connect();
        client.subscribe('T-1');
        settle(async);

        transport.last
          ..serverSend(eventFrame('T-1', 1))
          ..serverSend(eventFrame('T-1', 2));
        settle(async);
        expect(client.lastSequence('T-1'), 2); // đã tính vào sequence

        final events = collect(client.events); // listen muộn
        settle(async);

        expect(events.map((e) => e.sequence), [1, 2]);
      });
    });

    test('loại frame lạ được bỏ qua, kết nối vẫn dùng được', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = _client(transport);
        final events = collect(client.events);
        client.connect();
        client.subscribe('T-1');
        settle(async);

        transport.last
          ..serverSend({'type': 'presence', 'online': 3})
          ..serverSend(eventFrame('T-1', 1));
        settle(async);

        expect(events, hasLength(1));
        expect(client.state.status, RealtimeStatus.connected);
      });
    });
  });
}
