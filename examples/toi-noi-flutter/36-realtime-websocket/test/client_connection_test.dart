import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_36/realtime_client.dart';

import 'support.dart';

const _ms500 = Duration(milliseconds: 500);

void main() {
  group('trạng thái kết nối', () {
    test('trước khi connect: closed, không có lý do, chưa gọi transport', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = buildClient(transport);

        settle(async);

        expect(client.state, RealtimeState.initial);
        expect(client.state.cause, isNull);
        expect(transport.attempts, 0);
      });
    });

    test('connecting rồi connected, token đi theo lần connect', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = buildClient(transport);
        final states = collect(client.states);

        client.connect();
        settle(async);

        expect(states, [
          const RealtimeState(RealtimeStatus.connecting),
          const RealtimeState(RealtimeStatus.connected),
        ]);
        expect(transport.tokens, ['token-1']);
      });
    });

    test(
      'chỉ welcome mới đổi sang connected: bắt tay WebSocket xong chưa đủ',
      () {
        fakeAsync((async) {
          final transport = FakeTransport()..autoWelcome = false;
          final client = buildClient(transport);

          client.connect();
          settle(async);
          expect(client.state.status, RealtimeStatus.connecting);
          expect(transport.connections, hasLength(1));

          transport.last.serverWelcome();
          settle(async);
          expect(client.state.status, RealtimeStatus.connected);
        });
      },
    );

    test('connect lần hai khi đang mở không tạo thêm kết nối', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = buildClient(transport);

        client
          ..connect()
          ..connect();
        settle(async);
        client.connect();
        settle(async);

        expect(transport.attempts, 1);
      });
    });

    test('lần mở đầu thất bại: reconnecting với thời gian chờ đã chọn, rồi connected', () {
      fakeAsync((async) {
        final transport = FakeTransport()..failures = 2;
        final client = buildClient(transport);
        final states = collect(client.states);

        client.connect();
        settle(async);
        async.elapse(_ms500);
        async.elapse(const Duration(seconds: 1));

        expect(states, [
          const RealtimeState(RealtimeStatus.connecting),
          const RealtimeState(
            RealtimeStatus.reconnecting,
            attempt: 1,
            nextRetryIn: _ms500,
          ),
          const RealtimeState(RealtimeStatus.reconnecting, attempt: 1),
          const RealtimeState(
            RealtimeStatus.reconnecting,
            attempt: 2,
            nextRetryIn: Duration(seconds: 1),
          ),
          const RealtimeState(RealtimeStatus.reconnecting, attempt: 2),
          const RealtimeState(RealtimeStatus.connected),
        ]);
        expect(transport.attempts, 3);
      });
    });

    test(
      'không welcome trong handshakeTimeout: coi là chết, đóng và nối lại',
      () {
        fakeAsync((async) {
          final transport = FakeTransport()..autoWelcome = false;
          final client = buildClient(transport);

          client.connect();
          settle(async);
          async.elapse(
            const Duration(seconds: 10) - const Duration(milliseconds: 1),
          );
          expect(client.state.status, RealtimeStatus.connecting);

          async.elapse(const Duration(milliseconds: 1));
          expect(client.state.status, RealtimeStatus.reconnecting);
          expect(transport.last.closedByClient, isTrue);

          async.elapse(_ms500);
          expect(transport.attempts, 2);
        });
      },
    );

    test('frame hỏng không làm rớt kết nối, chỉ báo ra issues', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = buildClient(transport);
        final issues = collect(client.issues);
        client.connect();
        settle(async);

        transport.last
          ..serverSendRaw('không phải json')
          ..serverSend({'type': 'event', 'tripId': 'T-1'});
        settle(async);

        expect(client.state.status, RealtimeStatus.connected);
        expect(issues.map((i) => i.code), [
          'malformed_frame',
          'malformed_frame',
        ]);
      });
    });

    test('error của gateway đi ra issues; chuyến không có thật bị bỏ khỏi danh sách nghe', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = buildClient(transport);
        final issues = collect(client.issues);
        client.connect();
        client.subscribe('T-404');
        settle(async);

        transport.last.serverSend({
          'type': 'error',
          'code': 'trip_not_found',
          'message': 'Trip not found',
          'tripId': 'T-404',
        });
        settle(async);

        expect(issues.single.code, 'trip_not_found');
        expect(issues.single.tripId, 'T-404');
        expect(client.lastSequence('T-404'), isNull);
      });
    });
  });
}
