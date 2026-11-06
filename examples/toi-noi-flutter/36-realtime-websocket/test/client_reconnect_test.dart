import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_36/realtime_client.dart';
import 'package:toi_noi_36/realtime_transport.dart';

import 'support.dart';

void main() {
  group('lịch nối lại', () {
    test('mỗi lần thất bại chờ lâu gấp đôi, tới trần rồi giữ nguyên', () {
      fakeAsync((async) {
        final transport = FakeTransport()..failures = 100;
        final client = buildClient(transport);
        client.connect();
        settle(async);

        final waits = <Duration>[];
        for (var i = 0; i < 7; i++) {
          final wait = client.state.nextRetryIn!;
          waits.add(wait);
          async.elapse(wait);
        }

        // delay = trần / 2 (FixedRandom 0.5); trần: 1, 2, 4, 8, 16, 30, 30 s.
        expect(waits.map((d) => d.inMilliseconds), [
          500,
          1000,
          2000,
          4000,
          8000,
          15000,
          15000,
        ]);
      });
    });

    test('đúng mốc: chưa tới hạn thì chưa gọi, đủ hạn thì gọi', () {
      fakeAsync((async) {
        final transport = FakeTransport()..failures = 1;
        final client = buildClient(transport);
        client.connect();
        settle(async);
        expect(transport.attempts, 1);

        async.elapse(const Duration(milliseconds: 499));
        expect(transport.attempts, 1);
        async.elapse(const Duration(milliseconds: 1));
        expect(transport.attempts, 2);
        expect(client.state.status, RealtimeStatus.connected);
      });
    });

    test('mọi kiểu mất kết nối đều dẫn tới nối lại: rớt đột ngột, lỗi socket, mã đóng khác 4401', () {
      final causes = <String, void Function(FakeConnection)>{
        'rớt không có mã': (c) => c.serverDrop(),
        'lỗi socket': (c) => c.serverError(),
        'mã 1000': (c) => c.serverClose(1000),
        'mã 4001 (gateway khởi động lại)': (c) => c.serverClose(4001),
        'mã 1011': (c) => c.serverClose(1011),
      };
      causes.forEach((name, drop) {
        fakeAsync((async) {
          final transport = FakeTransport();
          final client = buildClient(transport);
          client.connect();
          settle(async);
          expect(client.state.status, RealtimeStatus.connected, reason: name);

          drop(transport.last);
          settle(async);
          expect(
            client.state.status,
            RealtimeStatus.reconnecting,
            reason: name,
          );

          async.elapse(client.state.nextRetryIn!);
          expect(client.state.status, RealtimeStatus.connected, reason: name);
          expect(transport.attempts, 2, reason: name);
        });
      });
    });

    test('kết nối chập chờn (sống ngắn hơn stableAfter): backoff không quay về đầu', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = buildClient(transport);
        client.connect();
        settle(async);

        final waits = <int>[];
        for (var i = 0; i < 4; i++) {
          async.elapse(const Duration(seconds: 1)); // sống 1 s rồi rớt
          transport.last.serverDrop();
          settle(async);
          waits.add(client.state.nextRetryIn!.inMilliseconds);
          async.elapse(client.state.nextRetryIn!);
        }

        expect(waits, [500, 1000, 2000, 4000]);
      });
    });

    test('sống đủ stableAfter thì backoff về lần đầu; thiếu 1 ms thì chưa', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        // Tắt heartbeat: gateway giả của test này không trả lời ping.
        final client = buildClient(
          transport,
          heartbeatInterval: const Duration(hours: 1),
        );
        client.connect();
        settle(async);
        // Hai lần chập chờn để backoff leo lên.
        for (var i = 0; i < 2; i++) {
          transport.last.serverDrop();
          settle(async);
          async.elapse(client.state.nextRetryIn!);
        }
        transport.last.serverDrop();
        settle(async);
        expect(client.state.nextRetryIn, const Duration(seconds: 2));
        async.elapse(client.state.nextRetryIn!);

        // Thiếu 1 ms: chưa ổn định, lần chờ kế vẫn là 4 s.
        async.elapse(
          const Duration(seconds: 30) - const Duration(milliseconds: 1),
        );
        transport.last.serverDrop();
        settle(async);
        expect(client.state.nextRetryIn, const Duration(seconds: 4));
        async.elapse(client.state.nextRetryIn!);

        // Đủ 30 s: ổn định, lần chờ quay về 500 ms.
        async.elapse(const Duration(seconds: 30));
        transport.last.serverDrop();
        settle(async);
        expect(client.state.nextRetryIn, const Duration(milliseconds: 500));
        expect(client.state.attempt, 1);
      });
    });
  });

  group('lỗi xác thực', () {
    test(
      'gateway đóng bằng 4401 lúc bắt tay: closed(unauthorized), không nối lại',
      () {
        fakeAsync((async) {
          final transport = FakeTransport()..autoWelcome = false;
          final client = buildClient(transport);
          client.connect();
          settle(async);

          transport.last.serverClose(4401);
          settle(async);
          async.elapse(const Duration(hours: 1));

          expect(
            client.state,
            const RealtimeState(
              RealtimeStatus.closed,
              cause: CloseCause.unauthorized,
            ),
          );
          expect(transport.attempts, 1);
          expect(async.pendingTimers, isEmpty);
        });
      },
    );

    test('token bị thu hồi giữa chừng (4401 khi đang connected) cũng không nối lại', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = buildClient(transport);
        client.connect();
        settle(async);
        expect(client.state.status, RealtimeStatus.connected);

        transport.last.serverClose(4401);
        settle(async);
        async.elapse(const Duration(hours: 1));

        expect(client.state.cause, CloseCause.unauthorized);
        expect(transport.attempts, 1);
        expect(async.pendingTimers, isEmpty);
      });
    });

    test('gateway từ chối ở bước nâng cấp bằng HTTP 401: closed(unauthorized), không nối lại', () {
      fakeAsync((async) {
        final transport = FakeTransport()
          ..failures = 10
          ..failWith = const RealtimeAuthRejected();
        final client = buildClient(transport);

        client.connect();
        settle(async);
        async.elapse(const Duration(hours: 1));

        expect(
          client.state,
          const RealtimeState(
            RealtimeStatus.closed,
            cause: CloseCause.unauthorized,
          ),
        );
        expect(transport.attempts, 1);
        expect(async.pendingTimers, isEmpty);
      });
    });

    test('token cũ khi resume: closed(unauthorized), resume() không mở lại, phải connect() sau khi đăng nhập lại', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        var token = 'het-han';
        final client = buildClient(transport, tokenProvider: () async => token);
        client.connect();
        settle(async);
        client.pause();
        settle(async);

        transport.autoWelcome = false;
        client.resume(); // tokenProvider vẫn trả token hết hạn
        settle(async);
        transport.last.serverClose(4401);
        settle(async);
        client.resume();
        settle(async);
        expect(client.state.cause, CloseCause.unauthorized);
        expect(transport.attempts, 2);

        token = 'moi';
        transport.autoWelcome = true;
        client.connect();
        settle(async);
        expect(client.state.status, RealtimeStatus.connected);
        expect(transport.tokens, ['het-han', 'het-han', 'moi']);
      });
    });

    test('chưa có token (provider trả null): closed(unauthorized), không gọi transport', () {
      fakeAsync((async) {
        final transport = FakeTransport();
        final client = buildClient(transport, tokenProvider: () async => null);

        client.connect();
        settle(async);

        expect(client.state.cause, CloseCause.unauthorized);
        expect(transport.attempts, 0);
      });
    });

    test('sau khi làm mới token, connect lại dùng token mới', () {
      fakeAsync((async) {
        final transport = FakeTransport()..autoWelcome = false;
        var token = 'cu';
        final client = buildClient(transport, tokenProvider: () async => token);
        client.connect();
        settle(async);
        transport.last.serverClose(4401);
        settle(async);

        token = 'moi';
        transport.autoWelcome = true;
        client.connect();
        settle(async);

        expect(transport.tokens, ['cu', 'moi']);
        expect(client.state.status, RealtimeStatus.connected);
      });
    });

    test(
      'tokenProvider ném lỗi (mất mạng khi làm mới): thử lại như lỗi tạm thời',
      () {
        fakeAsync((async) {
          final transport = FakeTransport();
          var calls = 0;
          final client = buildClient(
            transport,
            tokenProvider: () async {
              if (++calls == 1) throw StateError('không làm mới được token');
              return 'token-1';
            },
          );

          client.connect();
          settle(async);
          expect(client.state.status, RealtimeStatus.reconnecting);
          expect(transport.attempts, 0);

          async.elapse(client.state.nextRetryIn!);
          expect(client.state.status, RealtimeStatus.connected);
          expect(calls, 2);
        });
      },
    );

    test(
      'lỗi mạng KHÔNG bị nhầm với lỗi xác thực: nối lại mãi, token không bị bỏ',
      () {
        fakeAsync((async) {
          final transport = FakeTransport()..failures = 3;
          final client = buildClient(transport);
          client.connect();
          settle(async);
          for (var i = 0; i < 3; i++) {
            async.elapse(client.state.nextRetryIn!);
          }

          expect(client.state.status, RealtimeStatus.connected);
          expect(transport.tokens, [
            'token-1',
            'token-1',
            'token-1',
            'token-1',
          ]);
        });
      },
    );
  });
}
