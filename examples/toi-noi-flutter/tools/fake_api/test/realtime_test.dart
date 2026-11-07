import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:shelf/shelf.dart';
import 'package:shelf/shelf_io.dart' as shelf_io;
import 'package:test/test.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';
import 'package:web_socket_channel/io.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

import 'support.dart';

typedef Frame = Map<String, Object?>;

/// Một client WebSocket tối giản của test: gom frame JSON theo thứ tự đến.
final class Probe {
  Probe._(this.channel)
    : _frames = StreamIterator<Frame>(
        channel.stream.map(
          (Object? data) => jsonDecode(data! as String) as Frame,
        ),
      );

  final WebSocketChannel channel;
  final StreamIterator<Frame> _frames;

  static Future<Probe> connect(Uri url, {String? token}) async {
    final channel = IOWebSocketChannel.connect(
      url,
      headers: {'Authorization': ?(token == null ? null : 'Bearer $token')},
    );
    await channel.ready;
    return Probe._(channel);
  }

  Future<Frame> next() async {
    final hasNext = await _frames.moveNext().timeout(
      const Duration(seconds: 5),
      onTimeout: () => throw TimeoutException('Không có frame nào trong 5 s'),
    );
    if (!hasNext) throw StateError('Kết nối đã đóng');
    return _frames.current;
  }

  void send(Frame frame) => channel.sink.add(jsonEncode(frame));

  /// Gửi `ping` rồi đọc tới `pong`: mọi frame gửi trước đó đã được xử lý.
  /// Trả các frame đã đọc trước `pong`.
  Future<List<Frame>> fence() async {
    send({'type': 'ping', 'id': 'fence'});
    final seen = <Frame>[];
    while (true) {
      final frame = await next();
      if (frame['type'] == 'pong' && frame['id'] == 'fence') return seen;
      seen.add(frame);
    }
  }

  /// Đọc cho tới khi server đóng; trả mã đóng.
  Future<int?> closed() async {
    while (await _frames.moveNext().timeout(const Duration(seconds: 5))) {}
    return channel.closeCode;
  }

  Future<void> close() => channel.sink.close();
}

final class Gateway {
  Gateway._(this.server, this.handler, this.auth, this.realtime, this.store);

  final HttpServer server;
  final Handler handler;
  final AuthModule auth;
  final RealtimeModule realtime;
  final FakeApiStore store;

  Uri get url => Uri.parse('ws://127.0.0.1:${server.port}$realtimePath');

  static Future<Gateway> start({
    RealtimeSubscribeAuthorizer? authorize,
    DateTime Function()? clock,
    int? maxLogPerTrip,
    bool rejectBeforeUpgrade = false,
  }) async {
    final auth = AuthModule();
    final realtime = RealtimeModule(
      verifyToken: auth.verifyAccessToken,
      authorize: authorize,
      logEpoch: 'epoch-test',
      maxLogPerTrip: maxLogPerTrip,
      rejectBeforeUpgrade: rejectBeforeUpgrade,
    );
    final store = FakeApiStore();
    var tripSeq = 0;
    final handler = createFakeApiHandler(
      clock: clock ?? () => DateTime.utc(2026, 11, 6, 3, 30),
      newId: () => 'T-${++tripSeq}',
      store: store,
      modules: [auth, realtime],
    );
    final server = await shelf_io.serve(
      handler,
      InternetAddress.loopbackIPv4,
      0,
    );
    return Gateway._(server, handler, auth, realtime, store);
  }

  /// Đăng nhập OTP giả và trả access token.
  Future<String> login({
    String phone = '+84000000001',
    String app = 'customer',
  }) async {
    final challenge = asMap(
      await call(
        handler,
        'POST',
        '/auth/otp/request',
        body: {'phone': phone, 'app': app},
      ),
    );
    final session = asMap(
      await call(
        handler,
        'POST',
        '/auth/otp/verify',
        body: {'requestId': challenge['requestId'], 'code': fakeOtpCode},
      ),
    );
    return session['accessToken']! as String;
  }

  /// Đặt một chuyến 5 km qua REST và trả mã chuyến.
  Future<String> createTrip() async {
    final reply = await call(
      handler,
      'POST',
      '/trips',
      body: {'serviceKind': 'ride', 'distanceKm': 5},
      headers: {'idempotency-key': 'k-${store.trips.length}'},
    );
    return asMap(reply)['id']! as String;
  }

  Future<Probe> connect({String? token}) async =>
      Probe.connect(url, token: token ?? await login());

  Future<void> close() => server.close(force: true);
}

void main() {
  late Gateway gateway;

  setUp(() async => gateway = await Gateway.start());
  tearDown(() => gateway.close());

  group('bắt tay và xác thực', () {
    test('có token hợp lệ: nhận welcome kèm userId', () async {
      final probe = await gateway.connect();
      final welcome = await probe.next();

      expect(welcome['type'], 'welcome');
      expect(welcome['userId'], 'U-1001');
      expect(welcome['connectionId'], 'c-0001');
      expect(gateway.realtime.connectionCount, 1);
      await probe.close();
    });

    test('không có token: nâng cấp thành công rồi đóng bằng mã 4401', () async {
      final probe = await Probe.connect(gateway.url);

      expect(await probe.closed(), realtimeCloseUnauthorized);
      expect(probe.channel.closeReason, 'unauthorized');
      expect(gateway.realtime.connectionCount, 0);
    });

    test('token lạ hoặc sai dạng: 4401', () async {
      for (final token in ['rt_khong-ton-tai', 'a.b.']) {
        final probe = await Probe.connect(gateway.url, token: token);
        expect(await probe.closed(), 4401, reason: token);
      }
    });

    test('token hết hạn: 4401', () async {
      var now = DateTime.utc(2026, 11, 6, 3, 30);
      await gateway.close();
      gateway = await Gateway.start(clock: () => now);
      final token = await gateway.login();
      now = now.add(const Duration(minutes: 16)); // access token sống 15 phút

      final probe = await Probe.connect(gateway.url, token: token);
      expect(await probe.closed(), 4401);
    });

    test('đường dẫn không phải WebSocket: 404, không phải 500', () async {
      final response = await gateway.handler(
        Request('GET', Uri.parse('http://fake.test$realtimePath')),
      );
      expect(response.statusCode, 404);
    });
  });

  group('subscribe, sự kiện bền và resume', () {
    test('subscribe chuyến mới: latestSequence 0, không phát lại gì', () async {
      final tripId = await gateway.createTrip();
      final probe = await gateway.connect();
      await probe.next(); // welcome

      probe.send({'type': 'subscribe', 'tripId': tripId});
      expect(await probe.next(), {
        'type': 'subscribed',
        'tripId': tripId,
        'latestSequence': 0,
        'replayed': 0,
        'logEpoch': 'epoch-test',
      });
      expect(gateway.realtime.subscriberCount(tripId), 1);
      await probe.close();
    });

    test(
      'sự kiện trực tiếp có sequence 1, 2, 3 và trạng thái sau sự kiện',
      () async {
        final tripId = await gateway.createTrip();
        final probe = await gateway.connect();
        await probe.next();
        probe.send({'type': 'subscribe', 'tripId': tripId});
        await probe.next();

        gateway.realtime
          ..publish(tripId, const SearchStarted())
          ..publish(tripId, const DriverAccepted('D-7'))
          ..publish(tripId, const DriverArrived());

        final frames = [
          await probe.next(),
          await probe.next(),
          await probe.next(),
        ];
        expect(frames.map((f) => f['sequence']), [1, 2, 3]);
        expect(frames.map((f) => f['status']), [
          'searching',
          'accepted',
          'arriving',
        ]);
        expect(frames[1]['event'], {
          'type': 'driverAccepted',
          'driverId': 'D-7',
        });
        expect(frames[0]['occurredAt'], '2026-11-06T03:30:00.000Z');
        await probe.close();
      },
    );

    test('lastSequence: phát lại phần còn thiếu theo thứ tự, rồi mới tới sự kiện mới', () async {
      final tripId = await gateway.createTrip();
      gateway.realtime
        ..publish(tripId, const SearchStarted())
        ..publish(tripId, const DriverAccepted('D-7'))
        ..publish(tripId, const DriverArrived());

      final probe = await gateway.connect();
      await probe.next();
      probe.send({'type': 'subscribe', 'tripId': tripId, 'lastSequence': 1});

      expect((await probe.next())['replayed'], 2);
      expect((await probe.next())['sequence'], 2);
      expect((await probe.next())['sequence'], 3);

      gateway.realtime.publish(tripId, const PickedUp());
      expect((await probe.next())['sequence'], 4);
      await probe.close();
    });

    test('không có lastSequence: phát lại từ đầu; lastSequence bằng mới nhất: không phát lại', () async {
      final tripId = await gateway.createTrip();
      gateway.realtime
        ..publish(tripId, const SearchStarted())
        ..publish(tripId, const DriverAccepted('D-7'));
      final probe = await gateway.connect();
      await probe.next();

      probe.send({'type': 'subscribe', 'tripId': tripId});
      expect((await probe.next())['replayed'], 2);
      await probe.next();
      await probe.next();

      probe.send({'type': 'subscribe', 'tripId': tripId, 'lastSequence': 2});
      final ack = await probe.next();
      expect(ack['replayed'], 0);
      expect(ack['latestSequence'], 2);
      expect(await probe.fence(), isEmpty);
      await probe.close();
    });

    test(
      'subscribe lại cùng chuyến không nhân đôi sự kiện trực tiếp',
      () async {
        final tripId = await gateway.createTrip();
        final probe = await gateway.connect();
        await probe.next();
        for (var i = 0; i < 2; i++) {
          probe.send({'type': 'subscribe', 'tripId': tripId});
          await probe.next();
        }

        gateway.realtime.publish(tripId, const SearchStarted());
        expect((await probe.next())['sequence'], 1);
        expect(await probe.fence(), isEmpty);
        expect(gateway.realtime.subscriberCount(tripId), 1);
        await probe.close();
      },
    );

    test('hai kết nối cùng nghe một chuyến đều nhận sự kiện', () async {
      final tripId = await gateway.createTrip();
      final a = await gateway.connect();
      final b = await gateway.connect(
        token: await gateway.login(phone: '+84000000002', app: 'driver'),
      );
      for (final probe in [a, b]) {
        await probe.next();
        probe.send({'type': 'subscribe', 'tripId': tripId});
        await probe.next();
      }

      gateway.realtime.publish(tripId, const SearchStarted());

      expect((await a.next())['sequence'], 1);
      expect((await b.next())['sequence'], 1);
      await a.close();
      await b.close();
    });

    test('unsubscribe: không nhận sự kiện mới nữa', () async {
      final tripId = await gateway.createTrip();
      final probe = await gateway.connect();
      await probe.next();
      probe.send({'type': 'subscribe', 'tripId': tripId});
      await probe.next();

      probe.send({'type': 'unsubscribe', 'tripId': tripId});
      expect((await probe.next())['type'], 'unsubscribed');
      gateway.realtime.publish(tripId, const SearchStarted());

      expect(await probe.fence(), isEmpty);
      expect(gateway.realtime.subscriberCount(tripId), 0);
      await probe.close();
    });

    test('đóng kết nối thì rời mọi kênh', () async {
      final tripId = await gateway.createTrip();
      final probe = await gateway.connect();
      await probe.next();
      probe.send({'type': 'subscribe', 'tripId': tripId});
      await probe.next();

      await probe.close();
      // Chờ server thấy kết nối đóng.
      for (var i = 0; i < 50 && gateway.realtime.connectionCount > 0; i++) {
        await Future<void>.delayed(const Duration(milliseconds: 10));
      }

      expect(gateway.realtime.connectionCount, 0);
      expect(gateway.realtime.subscriberCount(tripId), 0);
    });
  });

  group('vị trí (tạm thời)', () {
    test('frame location không có sequence và không nằm trong log', () async {
      final tripId = await gateway.createTrip();
      final probe = await gateway.connect();
      await probe.next();
      probe.send({'type': 'subscribe', 'tripId': tripId});
      await probe.next();

      final delivered = gateway.realtime.publishLocation(
        tripId,
        const DriverLocationUpdate(
          driverId: 'D-7',
          lat: 10.7769,
          lng: 106.7009,
          headingDegrees: 90,
        ),
      );

      final frame = await probe.next();
      expect(delivered, 1);
      expect(frame['type'], 'location');
      expect(frame.containsKey('sequence'), isFalse);
      expect(frame['lat'], 10.7769);
      expect(frame['at'], '2026-11-06T03:30:00.000Z');
      expect(gateway.realtime.latestSequence(tripId), 0);
      await probe.close();
    });

    test(
      'người nghe đến sau nhận vị trí mới nhất, sau phần phát lại',
      () async {
        final tripId = await gateway.createTrip();
        gateway.realtime.publish(tripId, const SearchStarted());
        for (final lat in [10.1, 10.2, 10.3]) {
          gateway.realtime.publishLocation(
            tripId,
            DriverLocationUpdate(driverId: 'D-7', lat: lat, lng: 106),
          );
        }

        final probe = await gateway.connect();
        await probe.next();
        probe.send({'type': 'subscribe', 'tripId': tripId});

        expect((await probe.next())['type'], 'subscribed');
        expect((await probe.next())['type'], 'event');
        final location = await probe.next();
        expect(location['type'], 'location');
        expect(location['lat'], 10.3); // chỉ bản mới nhất, không có 10.1, 10.2
        expect(await probe.fence(), isEmpty);
        await probe.close();
      },
    );
  });

  group('lỗi giao thức không làm rớt kết nối', () {
    late Probe probe;

    setUp(() async {
      probe = await gateway.connect();
      await probe.next();
    });

    tearDown(() => probe.close());

    test('chuyến không tồn tại: trip_not_found', () async {
      probe.send({'type': 'subscribe', 'tripId': 'T-404'});
      expect(await probe.next(), {
        'type': 'error',
        'code': 'trip_not_found',
        'message': 'Trip not found',
        'tripId': 'T-404',
      });
      expect(await probe.fence(), isEmpty); // vẫn dùng được
    });

    test('frame hỏng, thiếu tripId, lastSequence sai, type lạ', () async {
      probe.channel.sink.add('không phải json');
      expect((await probe.next())['code'], 'bad_frame');

      probe.send({'type': 'subscribe'});
      expect((await probe.next())['code'], 'bad_frame');

      probe.send({'type': 'subscribe', 'tripId': 'T-1', 'lastSequence': -1});
      expect((await probe.next())['code'], 'bad_frame');

      probe.send({'type': 'subscribe', 'tripId': 'T-1', 'lastSequence': '3'});
      expect((await probe.next())['code'], 'bad_frame');

      probe.send({'type': 'dance'});
      expect((await probe.next())['code'], 'unknown_type');
      expect(await probe.fence(), isEmpty);
    });

    test('ping được trả pong kèm id', () async {
      probe.send({'type': 'ping', 'id': 42});
      expect(await probe.next(), {'type': 'pong', 'id': 42});
    });
  });

  group('quyền nghe kênh', () {
    test('authorizer từ chối: forbidden và không được đăng ký', () async {
      await gateway.close();
      gateway = await Gateway.start(
        authorize: (who, trip) => who.role == 'driver',
      );
      final tripId = await gateway.createTrip();
      final customer = await gateway.connect();
      await customer.next();

      customer.send({'type': 'subscribe', 'tripId': tripId});
      final error = await customer.next();
      expect(error['code'], 'forbidden');
      expect(gateway.realtime.subscriberCount(tripId), 0);

      final driver = await gateway.connect(
        token: await gateway.login(phone: '+84000000002', app: 'driver'),
      );
      await driver.next();
      driver.send({'type': 'subscribe', 'tripId': tripId});
      expect((await driver.next())['type'], 'subscribed');
      await customer.close();
      await driver.close();
    });
  });

  group('simulator HTTP', () {
    test('POST events: 201 kèm sequence, trạng thái; chuyến lạ 404', () async {
      final tripId = await gateway.createTrip();

      final first = await call(
        gateway.handler,
        'POST',
        '/realtime/trips/$tripId/events',
        body: {'type': 'searchStarted'},
      );
      final second = await call(
        gateway.handler,
        'POST',
        '/realtime/trips/$tripId/events',
        body: {'type': 'driverAccepted', 'driverId': 'D-7'},
      );
      final missing = await call(
        gateway.handler,
        'POST',
        '/realtime/trips/T-404/events',
        body: {'type': 'searchStarted'},
      );

      expect(first.status, 201);
      expect(asMap(first), {
        'tripId': tripId,
        'sequence': 1,
        'version': 1,
        'status': 'searching',
      });
      expect(asMap(second)['sequence'], 2);
      expect(missing.status, 404);
    });

    test('sự kiện sai thứ tự: 409 và sequence không nhảy', () async {
      final tripId = await gateway.createTrip();

      final reply = await call(
        gateway.handler,
        'POST',
        '/realtime/trips/$tripId/events',
        body: {'type': 'completed'}, // requested chưa thể completed
      );

      expect(reply.status, 409);
      expect(gateway.realtime.latestSequence(tripId), 0);
      final ok = await call(
        gateway.handler,
        'POST',
        '/realtime/trips/$tripId/events',
        body: {'type': 'searchStarted'},
      );
      expect(asMap(ok)['sequence'], 1);
    });

    test('body sai: 422 theo từng trường', () async {
      final tripId = await gateway.createTrip();
      Future<Reply> post(String path, Object body) => call(
        gateway.handler,
        'POST',
        '/realtime/trips/$tripId/$path',
        body: body,
      );

      expect(
        errorsOf(await post('events', {'type': 'nhay-mua'})),
        contains('type'),
      );
      expect(
        errorsOf(
          await post('events', {'type': 'driverAccepted', 'driverId': ' '}),
        ),
        contains('driverId'),
      );
      expect(
        errorsOf(await post('events', {'type': 'cancelled', 'by': 'bot'})),
        contains('by'),
      );
      final location = await post('location', {
        'driverId': 'D-7',
        'lat': 91,
        'lng': 'x',
      });
      expect(location.status, 422);
      expect(errorsOf(location), allOf(contains('lat'), contains('lng')));
    });

    test('POST location: 202 kèm số kết nối đã nhận', () async {
      final tripId = await gateway.createTrip();
      final probe = await gateway.connect();
      await probe.next();
      probe.send({'type': 'subscribe', 'tripId': tripId});
      await probe.next();

      final reply = await call(
        gateway.handler,
        'POST',
        '/realtime/trips/$tripId/location',
        body: {'driverId': 'D-7', 'lat': 10.78, 'lng': 106.7},
      );

      expect(reply.status, 202);
      expect(asMap(reply)['delivered'], 1);
      expect((await probe.next())['type'], 'location');
      await probe.close();
    });
  });

  group('server khởi động lại', () {
    test('dropConnections: client thấy mã 4001; log giữ nguyên; kết nối mới resume được', () async {
      final tripId = await gateway.createTrip();
      final first = await gateway.connect();
      await first.next();
      first.send({'type': 'subscribe', 'tripId': tripId});
      await first.next();
      gateway.realtime.publish(tripId, const SearchStarted());
      expect((await first.next())['sequence'], 1);

      await gateway.realtime.dropConnections();
      expect(await first.closed(), realtimeCloseRestart);
      expect(gateway.realtime.connectionCount, 0);

      // Sự kiện xảy ra trong lúc client mất kết nối.
      gateway.realtime.publish(tripId, const DriverAccepted('D-7'));
      final second = await gateway.connect();
      await second.next();
      second.send({'type': 'subscribe', 'tripId': tripId, 'lastSequence': 1});
      expect((await second.next())['replayed'], 1);
      expect((await second.next())['sequence'], 2);
      await second.close();
    });
  });

  group('REST và realtime cùng một chuyến', () {
    test('publish ghi vào kho: GET /trips thấy trạng thái, version và lastSequence', () async {
      final tripId = await gateway.createTrip();
      final before = asMap(
        await call(gateway.handler, 'GET', '/trips/$tripId'),
      );
      expect(before['version'], 0);
      expect(before['lastSequence'], 0);
      expect(before['logEpoch'], 'epoch-test');

      gateway.realtime
        ..publish(tripId, const SearchStarted())
        ..publish(tripId, const DriverAccepted('D-7'));

      final after = asMap(await call(gateway.handler, 'GET', '/trips/$tripId'));
      expect(after['status'], 'accepted');
      expect(after['driverId'], 'D-7');
      expect(after['version'], 2);
      expect(after['lastSequence'], 2);
    });

    test(
      'event mang version của chuyến sau sự kiện; version trùng với REST',
      () async {
        final tripId = await gateway.createTrip();
        final probe = await gateway.connect();
        await probe.next();
        probe.send({'type': 'subscribe', 'tripId': tripId});
        await probe.next();

        gateway.realtime
          ..publish(tripId, const SearchStarted())
          ..publish(tripId, const DriverAccepted('D-7'));
        final first = await probe.next();
        final second = await probe.next();

        expect([first['version'], second['version']], [1, 2]);
        final rest = asMap(
          await call(gateway.handler, 'GET', '/trips/$tripId'),
        );
        expect(rest['version'], second['version']);
        await probe.close();
      },
    );

    test('snapshot REST + lastSequence: subscribe tiếp từ snapshot không phát lại gì', () async {
      final tripId = await gateway.createTrip();
      gateway.realtime
        ..publish(tripId, const SearchStarted())
        ..publish(tripId, const DriverAccepted('D-7'));
      final snapshot = asMap(
        await call(gateway.handler, 'GET', '/trips/$tripId'),
      );

      final probe = await gateway.connect();
      await probe.next();
      probe.send({
        'type': 'subscribe',
        'tripId': tripId,
        'lastSequence': snapshot['lastSequence'],
        'logEpoch': snapshot['logEpoch'],
      });
      expect((await probe.next())['replayed'], 0);
      gateway.realtime.publish(tripId, const DriverArrived());
      final next = await probe.next();
      expect(next['sequence'], 3);
      expect(next['version'], (snapshot['version']! as int) + 1);
      await probe.close();
    });

    test('REST accept sau khi realtime đã đổi trạng thái: 409 như mọi chuyến không còn requested', () async {
      final tripId = await gateway.createTrip();
      gateway.realtime.publish(tripId, const SearchStarted());

      final reply = await call(
        gateway.handler,
        'POST',
        '/trips/$tripId/accept',
        body: {'expectedVersion': 1},
        headers: {'x-fake-driver-id': 'D-7'},
      );

      expect(reply.status, 409);
    });

    test(
      'không mount RealtimeModule thì response chuyến không có trường mới',
      () async {
        final handler = createFakeApiHandler(newId: () => 'T-1');
        final created = await call(
          handler,
          'POST',
          '/trips',
          body: {'serviceKind': 'ride', 'distanceKm': 5},
          headers: {'idempotency-key': 'k1'},
        );
        expect(asMap(created).keys, isNot(contains('lastSequence')));
        expect(asMap(created).keys, isNot(contains('logEpoch')));
      },
    );
  });

  group('log_reset và replay_unavailable', () {
    Future<Probe> open() async {
      final probe = await gateway.connect();
      await probe.next();
      return probe;
    }

    test('logEpoch khác (gateway đã khởi động lại): log_reset, không đăng ký, không phát nhầm', () async {
      final tripId = await gateway.createTrip();
      gateway.realtime
        ..publish(tripId, const SearchStarted())
        ..publish(tripId, const DriverAccepted('D-7'))
        ..publish(tripId, const DriverArrived());
      final probe = await open();

      // Client từng nhận tới #2 của một log khác; log này đã tới #3.
      probe.send({
        'type': 'subscribe',
        'tripId': tripId,
        'lastSequence': 2,
        'logEpoch': 'epoch-cu',
      });

      final error = await probe.next();
      expect(error['code'], 'log_reset');
      expect(error['tripId'], tripId);
      expect(gateway.realtime.subscriberCount(tripId), 0);
      gateway.realtime.publish(tripId, const PickedUp());
      expect(await probe.fence(), isEmpty);
      await probe.close();
    });

    test(
      'lastSequence lớn hơn log hiện có (không kèm epoch): log_reset',
      () async {
        final tripId = await gateway.createTrip();
        gateway.realtime.publish(tripId, const SearchStarted());
        final probe = await open();

        probe.send({'type': 'subscribe', 'tripId': tripId, 'lastSequence': 5});

        expect((await probe.next())['code'], 'log_reset');
        await probe.close();
      },
    );

    test(
      'log_reset cho kết nối đang nghe chuyến đó thì gỡ luôn đăng ký',
      () async {
        final tripId = await gateway.createTrip();
        final probe = await open();
        probe.send({'type': 'subscribe', 'tripId': tripId});
        await probe.next();
        expect(gateway.realtime.subscriberCount(tripId), 1);

        probe.send({
          'type': 'subscribe',
          'tripId': tripId,
          'logEpoch': 'epoch-cu',
        });
        await probe.next();

        expect(gateway.realtime.subscriberCount(tripId), 0);
        await probe.close();
      },
    );

    test('log bị cắt: xin phần đã cắt nhận replay_unavailable, phần còn lại phát lại bình thường', () async {
      await gateway.close();
      gateway = await Gateway.start(maxLogPerTrip: 2);
      final tripId = await gateway.createTrip();
      gateway.realtime
        ..publish(tripId, const SearchStarted())
        ..publish(tripId, const DriverAccepted('D-7'))
        ..publish(tripId, const DriverArrived())
        ..publish(tripId, const PickedUp());
      final probe = await open();

      for (final last in [0, 1]) {
        probe.send({
          'type': 'subscribe',
          'tripId': tripId,
          'lastSequence': last,
        });
        expect(
          (await probe.next())['code'],
          'replay_unavailable',
          reason: 'last=$last',
        );
      }
      expect(gateway.realtime.subscriberCount(tripId), 0);

      probe.send({'type': 'subscribe', 'tripId': tripId, 'lastSequence': 2});
      final ack = await probe.next();
      expect(ack['replayed'], 2);
      expect(ack['latestSequence'], 4);
      expect((await probe.next())['sequence'], 3);
      expect((await probe.next())['sequence'], 4);
      await probe.close();
    });
  });

  group('restart: gateway mất log', () {
    test('đóng kết nối, sang logEpoch mới, sequence bắt đầu lại; trạng thái chuyến giữ nguyên', () async {
      final tripId = await gateway.createTrip();
      final probe = await gateway.connect();
      await probe.next();
      probe.send({'type': 'subscribe', 'tripId': tripId});
      await probe.next();
      gateway.realtime
        ..publish(tripId, const SearchStarted())
        ..publish(tripId, const DriverAccepted('D-7'));
      await probe.next();
      await probe.next();

      await gateway.realtime.restart(logEpoch: 'epoch-moi');

      expect(await probe.closed(), realtimeCloseRestart);
      final snapshot = asMap(
        await call(gateway.handler, 'GET', '/trips/$tripId'),
      );
      expect(snapshot['logEpoch'], 'epoch-moi');
      expect(snapshot['lastSequence'], 0);
      expect(snapshot['version'], 2); // kho không mất
      expect(snapshot['status'], 'accepted');
      final event = gateway.realtime.publish(tripId, const DriverArrived());
      expect(event.sequence, 1);
      expect(event.version, 3);

      // Client cũ quay lại với log cũ: bị từ chối thay vì nhận nhầm "#1" mới.
      final again = await gateway.connect();
      await again.next();
      again.send({
        'type': 'subscribe',
        'tripId': tripId,
        'lastSequence': 2,
        'logEpoch': 'epoch-test',
      });
      expect((await again.next())['code'], 'log_reset');
      await again.close();
    });
  });

  group('lời mời cuốc cho tài xế (kênh offers)', () {
    Future<Probe> driver({String phone = '+84000000002'}) async {
      final probe = await gateway.connect(
        token: await gateway.login(phone: phone, app: 'driver'),
      );
      await probe.next();
      return probe;
    }

    test('tài xế subscribe offers, nhận lời mời có chuyến, cước, hạn; chỉ đúng người đó', () async {
      final tripId = await gateway.createTrip();
      final mine = await driver();
      final other = await driver(phone: '+84000000003');
      for (final probe in [mine, other]) {
        probe.send({'type': 'subscribe', 'channel': 'offers'});
        expect(await probe.next(), {'type': 'subscribed', 'channel': 'offers'});
      }

      final reply = await call(
        gateway.handler,
        'POST',
        '/realtime/drivers/U-1001/offers', // người dùng đầu tiên đăng nhập là U-1001
        body: {'tripId': tripId, 'ttlSeconds': 20},
      );

      expect(reply.status, 202);
      expect(asMap(reply)['delivered'], 1);
      final offer = await mine.next();
      expect(offer, {
        'type': 'offer',
        'offerId': 'O-0001',
        'tripId': tripId,
        'serviceKind': 'ride',
        'distanceMeters': 5000,
        'fare': 25500,
        'currency': 'VND',
        'expiresAt': '2026-11-06T03:30:20.000Z',
      });
      expect(await other.fence(), isEmpty);
      await mine.close();
      await other.close();
    });

    test(
      'khách (không phải tài xế) subscribe offers: forbidden, kênh offers',
      () async {
        final customer = await gateway.connect();
        await customer.next();

        customer.send({'type': 'subscribe', 'channel': 'offers'});

        final error = await customer.next();
        expect(error['code'], 'forbidden');
        expect(error['channel'], 'offers');
        await customer.close();
      },
    );

    test('lời mời không phát lại cho người subscribe sau; unsubscribe thì ngừng nhận', () async {
      final tripId = await gateway.createTrip();
      final probe = await driver();
      await call(
        gateway.handler,
        'POST',
        '/realtime/drivers/U-1001/offers',
        body: {'tripId': tripId},
      );

      probe.send({'type': 'subscribe', 'channel': 'offers'});
      await probe.next();
      expect(await probe.fence(), isEmpty); // lời mời trước đó không có ở đây

      probe.send({'type': 'unsubscribe', 'channel': 'offers'});
      expect(await probe.next(), {'type': 'unsubscribed', 'channel': 'offers'});
      gateway.realtime.publishOffer('U-1001', tripId);
      expect(await probe.fence(), isEmpty);
      await probe.close();
    });

    test(
      'simulator: chuyến lạ 404, chuyến đã nhận 409, body sai 422',
      () async {
        final tripId = await gateway.createTrip();
        gateway.realtime
          ..publish(tripId, const SearchStarted())
          ..publish(tripId, const DriverAccepted('D-7'));
        Future<Reply> post(Object body) => call(
          gateway.handler,
          'POST',
          '/realtime/drivers/U-1001/offers',
          body: body,
        );

        expect((await post({'tripId': 'T-404'})).status, 404);
        expect((await post({'tripId': tripId})).status, 409);
        expect(errorsOf(await post({})), contains('tripId'));
        expect(
          errorsOf(await post({'tripId': tripId, 'ttlSeconds': 0})),
          contains('ttlSeconds'),
        );
      },
    );
  });

  group('token hết hạn và nâng cấp bị từ chối', () {
    test('kết nối đang mở bị đóng bằng 4401 khi token quá hạn', () async {
      var now = DateTime.utc(2026, 11, 6, 3, 30);
      await gateway.close();
      gateway = await Gateway.start(clock: () => now);
      final probe = await gateway.connect();
      await probe.next();
      expect(await gateway.realtime.closeExpired(), 0); // chưa tới hạn

      now = now.add(const Duration(minutes: 15)); // access token sống 15 phút
      expect(await gateway.realtime.closeExpired(), 1);

      expect(await probe.closed(), realtimeCloseUnauthorized);
      expect(probe.channel.closeReason, 'token expired');
      expect(gateway.realtime.connectionCount, 0);
    });

    test('rejectBeforeUpgrade: HTTP 401 ở bước nâng cấp, client dart:io thấy httpStatusCode 401', () async {
      await gateway.close();
      gateway = await Gateway.start(rejectBeforeUpgrade: true);

      Object? failure;
      try {
        await Probe.connect(gateway.url, token: 'rt_khong-ton-tai');
      } on Object catch (e) {
        failure = e;
      }

      expect(failure, isA<WebSocketChannelException>());
      final inner = (failure! as WebSocketChannelException).inner;
      expect(inner, isA<WebSocketException>());
      expect((inner! as WebSocketException).httpStatusCode, 401);
      expect(gateway.realtime.connectionsOpened, 0);
    });
  });

  group('tuỳ chọn dòng lệnh', () {
    test('--with-realtime mặc định tắt, độc lập với --with-auth', () {
      expect(parseServerOptions(const []).withRealtime, isFalse);
      final options = parseServerOptions(const ['--with-realtime']);
      expect(options.withRealtime, isTrue);
      expect(options.withAuth, isFalse);
    });
  });
}
