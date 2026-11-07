import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_36/realtime_client.dart';
import 'package:toi_noi_36/realtime_protocol.dart';

import 'support.dart';

/// Client đã connected, nghe T-1, hàng đợi vị trí cửa sổ 1 s, không heartbeat.
({RealtimeClient client, FakeTransport transport, List<DriverLocation> seen})
_setup(FakeAsync async, {Duration interval = const Duration(seconds: 1)}) {
  final transport = FakeTransport();
  final client = buildClient(
    transport,
    heartbeatInterval: const Duration(hours: 1),
    locationInterval: interval,
  );
  final seen = collect(client.locations);
  client.connect();
  client.subscribe('T-1');
  settle(async);
  return (client: client, transport: transport, seen: seen);
}

void main() {
  const ms = Duration(milliseconds: 1);

  group('gộp vị trí: giữ bản mới nhất, tối đa một bản mỗi cửa sổ', () {
    test('bản đầu ra ngay; các bản trong cửa sổ chỉ còn bản cuối, ra khi hết cửa sổ', () {
      fakeAsync((async) {
        final s = _setup(async);

        s.transport.last.serverSend(locationFrame('T-1', 10.1, second: 0));
        settle(async);
        expect(s.seen.map((l) => l.lat), [10.1]);

        async.elapse(const Duration(milliseconds: 200));
        s.transport.last.serverSend(locationFrame('T-1', 10.2, second: 1));
        async.elapse(const Duration(milliseconds: 200));
        s.transport.last.serverSend(locationFrame('T-1', 10.3, second: 2));
        async.elapse(const Duration(milliseconds: 599));
        expect(s.seen.map((l) => l.lat), [10.1]); // chưa hết 1 s

        async.elapse(ms);
        expect(s.seen.map((l) => l.lat), [10.1, 10.3]); // 10.2 bị gộp mất
      });
    });

    test(
      'tài xế gửi 5 vị trí mỗi giây trong 10 giây: client chỉ đưa ra 11 bản',
      () {
        fakeAsync((async) {
          final s = _setup(async);

          for (var i = 0; i < 50; i++) {
            s.transport.last.serverSend({
              ...locationFrame('T-1', 10 + i / 100),
              'at': DateTime.utc(
                2026,
                11,
                6,
                3,
                30,
              ).add(Duration(milliseconds: 200 * i)).toIso8601String(),
            });
            async.elapse(const Duration(milliseconds: 200));
          }
          async.elapse(const Duration(seconds: 2));

          // Một bản ngay lúc đầu, rồi mỗi giây một bản; bản nào ra cũng mới hơn bản trước.
          expect(s.seen, hasLength(11));
          expect(s.seen.first.lat, 10.0);
          expect(s.seen.last.lat, 10.49);
        });
      },
    );

    test('cửa sổ 2 giây: cùng luồng 5 vị trí mỗi giây chỉ ra 6 bản', () {
      fakeAsync((async) {
        final s = _setup(async, interval: const Duration(seconds: 2));

        for (var i = 0; i < 50; i++) {
          s.transport.last.serverSend({
            ...locationFrame('T-1', 10 + i / 100),
            'at': DateTime.utc(
              2026,
              11,
              6,
              3,
              30,
            ).add(Duration(milliseconds: 200 * i)).toIso8601String(),
          });
          async.elapse(const Duration(milliseconds: 200));
        }
        async.elapse(const Duration(seconds: 4));

        // Tại 0, 2, 4, 6, 8 và 10 giây.
        expect(s.seen, hasLength(6));
        expect(s.seen.last.lat, 10.49);
      });
    });

    test('hết cửa sổ mà không có gì chờ: cửa sổ đóng, bản kế tiếp ra ngay', () {
      fakeAsync((async) {
        final s = _setup(async);
        s.transport.last.serverSend(locationFrame('T-1', 10.1, second: 0));
        settle(async);

        async.elapse(
          const Duration(seconds: 1),
        ); // cửa sổ đóng, không có bản chờ
        s.transport.last.serverSend(locationFrame('T-1', 10.2, second: 5));
        settle(async);

        expect(s.seen.map((l) => l.lat), [10.1, 10.2]);
      });
    });

    test(
      'bản tới trong cửa sổ trailing cũng bị giữ tới hết cửa sổ kế tiếp',
      () {
        fakeAsync((async) {
          final s = _setup(async);
          s.transport.last.serverSend(locationFrame('T-1', 10.1, second: 0));
          async.elapse(const Duration(milliseconds: 500));
          s.transport.last.serverSend(locationFrame('T-1', 10.2, second: 1));
          async.elapse(
            const Duration(milliseconds: 500),
          ); // ra 10.2, mở cửa sổ mới
          expect(s.seen.map((l) => l.lat), [10.1, 10.2]);

          s.transport.last.serverSend(locationFrame('T-1', 10.3, second: 2));
          async.elapse(const Duration(milliseconds: 999));
          expect(s.seen.map((l) => l.lat), [10.1, 10.2]);
          async.elapse(ms);
          expect(s.seen.map((l) => l.lat), [10.1, 10.2, 10.3]);
        });
      },
    );

    test(
      'bản cũ hơn hoặc bằng bản đã nhận (theo at) bị bỏ, kể cả khi tới sau',
      () {
        fakeAsync((async) {
          final s = _setup(async, interval: Duration.zero);

          s.transport.last
            ..serverSend(locationFrame('T-1', 10.1, second: 10))
            ..serverSend(locationFrame('T-1', 10.0, second: 5)) // cũ hơn
            ..serverSend(locationFrame('T-1', 10.9, second: 10)) // bằng
            ..serverSend(locationFrame('T-1', 10.2, second: 11));
          settle(async);

          expect(s.seen.map((l) => l.lat), [10.1, 10.2]);
        });
      },
    );

    test('mỗi chuyến một cửa sổ riêng', () {
      fakeAsync((async) {
        final s = _setup(async);
        s.client.subscribe('T-2');
        settle(async);

        s.transport.last
          ..serverSend(locationFrame('T-1', 10.1, second: 0))
          ..serverSend(locationFrame('T-2', 20.1, second: 0));
        settle(async);

        expect(s.seen.map((l) => '${l.tripId}:${l.lat}'), [
          'T-1:10.1',
          'T-2:20.1',
        ]);
      });
    });

    test('unsubscribe rồi subscribe lại: bản gateway gửi lại (cùng at) không bị coi là cũ', () {
      fakeAsync((async) {
        final s = _setup(async);
        s.transport.last.serverSend(locationFrame('T-1', 10.1, second: 3));
        settle(async);

        s.client.unsubscribe('T-1');
        s.client.subscribe('T-1');
        settle(async);
        s.transport.last.serverSend(locationFrame('T-1', 10.1, second: 3));
        settle(async);

        expect(s.seen.map((l) => l.lat), [10.1, 10.1]);
      });
    });

    test('đi nền huỷ bản đang chờ: sau resume, bản gateway gửi lại vẫn ra (không bị mốc ảo chặn)', () {
      fakeAsync((async) {
        final s = _setup(async);
        s.transport.last
          ..serverSend(locationFrame('T-1', 10.1, second: 0))
          ..serverSend(locationFrame('T-1', 10.2, second: 1)); // đang chờ
        settle(async);
        expect(s.seen.map((l) => l.lat), [10.1]);

        s.client.pause();
        settle(async);
        s.client.resume();
        settle(async);
        s.transport.last.serverSend(locationFrame('T-1', 10.2, second: 1));
        settle(async);

        expect(s.seen.map((l) => l.lat), [10.1, 10.2]);
      });
    });

    test('chuyến không nghe thì vị trí bị bỏ', () {
      fakeAsync((async) {
        final s = _setup(async);

        s.transport.last.serverSend(locationFrame('T-9', 1.0));
        settle(async);

        expect(s.seen, isEmpty);
      });
    });

    test('unsubscribe bỏ bản đang chờ và huỷ Timer của cửa sổ', () {
      fakeAsync((async) {
        final s = _setup(async);
        s.transport.last
          ..serverSend(locationFrame('T-1', 10.1, second: 0))
          ..serverSend(locationFrame('T-1', 10.2, second: 1));
        settle(async);

        s.client.unsubscribe('T-1');
        async.elapse(const Duration(seconds: 5));

        expect(s.seen.map((l) => l.lat), [10.1]); // 10.2 không bao giờ ra
      });
    });

    test('sau khi nối lại, vị trí cuối gateway gửi lại (bằng bản đã có) bị bỏ, bản mới ra', () {
      fakeAsync((async) {
        final s = _setup(async);
        s.transport.last.serverSend(locationFrame('T-1', 10.1, second: 3));
        settle(async);

        s.transport.last.serverDrop();
        settle(async);
        async.elapse(s.client.state.nextRetryIn!);
        s.transport.last
          ..serverSend(
            locationFrame('T-1', 10.1, second: 3),
          ) // gateway gửi lại bản cuối
          ..serverSend(locationFrame('T-1', 10.5, second: 9));
        async.elapse(const Duration(seconds: 2));

        expect(s.seen.map((l) => l.lat), [10.1, 10.5]);
      });
    });

    test('dừng kết nối thì không còn Timer nào treo', () {
      fakeAsync((async) {
        final s = _setup(async);
        s.transport.last
          ..serverSend(locationFrame('T-1', 10.1, second: 0))
          ..serverSend(locationFrame('T-1', 10.2, second: 1));
        settle(async);

        s.client.close();
        settle(async);

        expect(async.pendingTimers, isEmpty);
      });
    });
  });
}
