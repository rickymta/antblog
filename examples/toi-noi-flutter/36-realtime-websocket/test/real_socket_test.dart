// Test Dart thuần: không khởi tạo TestWidgetsFlutterBinding, nên `HttpClient`
// không bị chặn và socket tới localhost là thật.
import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:shelf/shelf.dart';
import 'package:shelf/shelf_io.dart' as shelf_io;
import 'package:toi_noi_36/realtime_client.dart';
import 'package:toi_noi_36/realtime_protocol.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_36/realtime_transport.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'gateway_harness.dart';

void main() {
  late GatewayHarness gateway;
  late CountingTransport transport;
  late RealtimeClient client;
  late String tripId;

  Future<RealtimeClient> startClient({
    Future<String?> Function()? tokenProvider,
    Duration locationInterval = Duration.zero,
  }) async {
    final token = await gateway.login();
    client = RealtimeClient(
      transport: transport,
      url: gateway.url,
      tokenProvider: tokenProvider ?? () async => token,
      backoff: fastBackoff(),
      locationInterval: locationInterval,
    );
    return client;
  }

  setUp(() async {
    gateway = await GatewayHarness.start();
    transport = CountingTransport();
    tripId = await gateway.createTrip();
  });

  tearDown(() async {
    await client.dispose();
    await gateway.close();
  });

  test(
    'kết nối thật: states, welcome, subscribe, sự kiện đúng thứ tự',
    () async {
      await startClient();
      final states = <RealtimeState>[];
      final events = <TripEventEnvelope>[];
      client.states.listen(states.add);
      client.events.listen(events.add);

      client
        ..connect()
        ..subscribe(tripId);
      await waitForState(client, (s) => s.status == RealtimeStatus.connected);
      await eventually(() => gateway.realtime.subscriberCount(tripId) == 1);
      gateway.realtime
        ..publish(tripId, const SearchStarted())
        ..publish(tripId, const DriverAccepted('D-7'))
        ..publish(tripId, const DriverArrived());
      await eventually(() => events.length == 3);

      expect(states.map((s) => s.status), [
        RealtimeStatus.connecting,
        RealtimeStatus.connected,
      ]);
      expect(events.map((e) => e.sequence), [1, 2, 3]);
      expect(events.map((e) => e.status), [
        TripStatus.searching,
        TripStatus.accepted,
        TripStatus.arriving,
      ]);
      expect(events[1].event, const DriverAccepted('D-7'));
    },
  );

  test(
    'gateway khởi động lại: nối lại, không mất và không trùng sự kiện nào',
    () async {
      await startClient();
      final events = <TripEventEnvelope>[];
      client.events.listen(events.add);
      client
        ..connect()
        ..subscribe(tripId);
      await eventually(() => gateway.realtime.subscriberCount(tripId) == 1);
      gateway.realtime
        ..publish(tripId, const SearchStarted())
        ..publish(tripId, const DriverAccepted('D-7'));
      await eventually(() => events.length == 2);

      await gateway.realtime.dropConnections();
      // Sự kiện xảy ra trong lúc client chưa kịp nối lại.
      gateway.realtime
        ..publish(tripId, const DriverArrived())
        ..publish(tripId, const PickedUp());
      await waitForState(client, (s) => s.status == RealtimeStatus.connected);
      await eventually(() => events.length == 4);
      gateway.realtime.publish(
        tripId,
        const TripStarted(),
      ); // sự kiện trực tiếp sau resume
      await eventually(() => events.length == 5);

      expect(events.map((e) => e.sequence), [1, 2, 3, 4, 5]);
      expect(client.duplicatesDropped, 0); // gateway chỉ phát lại phần thiếu
      expect(transport.connects, 2);
      expect(gateway.realtime.subscriberCount(tripId), 1);
    },
  );

  test('subscribe với lastSequence đã lưu: chỉ nhận phần sau đó', () async {
    await startClient();
    gateway.realtime
      ..publish(tripId, const SearchStarted())
      ..publish(tripId, const DriverAccepted('D-7'))
      ..publish(tripId, const DriverArrived());
    final events = <TripEventEnvelope>[];
    client.events.listen(events.add);

    client
      ..connect()
      ..subscribe(tripId, lastSequence: 1);
    await eventually(() => events.length == 2);

    expect(events.map((e) => e.sequence), [2, 3]);
  });

  test(
    'token xấu: đúng một lần thử, closed(unauthorized), không nối lại',
    () async {
      await startClient(tokenProvider: () async => 'rt_khong-ton-tai');

      client.connect();
      final state = await waitForState(
        client,
        (s) => s.status == RealtimeStatus.closed,
      );

      expect(state.cause, CloseCause.unauthorized);
      expect(transport.connects, 1);
      expect(gateway.realtime.connectionCount, 0);
    },
  );

  test('token mới sau lỗi xác thực: connect lại thành công', () async {
    var token = 'rt_khong-ton-tai';
    await startClient(tokenProvider: () async => token);
    client.connect();
    await waitForState(client, (s) => s.cause == CloseCause.unauthorized);

    token = await gateway.login();
    client.connect();
    await waitForState(client, (s) => s.status == RealtimeStatus.connected);

    expect(transport.connects, 2);
  });

  test('pause giải phóng kết nối phía gateway; resume nối lại', () async {
    await startClient();
    client
      ..connect()
      ..subscribe(tripId);
    await eventually(() => gateway.realtime.subscriberCount(tripId) == 1);

    client.pause();
    await eventually(
      () => gateway.realtime.connectionCount == 0,
      reason: 'gateway phải thấy kết nối đóng',
    );
    expect(client.state.cause, CloseCause.background);

    client.resume();
    await eventually(() => gateway.realtime.subscriberCount(tripId) == 1);
    expect(client.state.status, RealtimeStatus.connected);
  });

  test(
    'vị trí tài xế qua socket thật: không có sequence, bản cuối luôn tới',
    () async {
      await startClient(locationInterval: const Duration(milliseconds: 30));
      final locations = <DriverLocation>[];
      client.locations.listen(locations.add);
      client
        ..connect()
        ..subscribe(tripId);
      await eventually(() => gateway.realtime.subscriberCount(tripId) == 1);

      for (final lat in [10.1, 10.2, 10.3, 10.4]) {
        gateway.realtime.publishLocation(
          tripId,
          DriverLocationUpdate(driverId: 'D-7', lat: lat, lng: 106.7),
        );
      }
      await eventually(
        () => locations.isNotEmpty && locations.last.lat == 10.4,
      );

      final lats = locations.map((l) => l.lat).toList();
      expect(lats.first, 10.1);
      expect(lats, orderedEquals([...lats]..sort())); // không bao giờ lùi
      expect(client.lastSequence(tripId), 0); // vị trí không làm tăng sequence
    },
  );

  test(
    'chuyến không có thật: issues nhận trip_not_found, kết nối vẫn sống',
    () async {
      await startClient();
      final issues = <RealtimeIssue>[];
      client.issues.listen(issues.add);

      client
        ..connect()
        ..subscribe('T-404');
      await eventually(() => issues.isNotEmpty);

      expect(issues.single.code, 'trip_not_found');
      expect(client.state.status, RealtimeStatus.connected);
    },
  );

  test(
    'không có gateway nào nghe: SocketException (refused), không phải HTTP 400',
    () async {
      await startClient();
      // Cổng 1 không ai nghe.
      final failure = await const WebSocketTransport()
          .connect(Uri.parse('ws://127.0.0.1:1/realtime'), token: 'x')
          .then<Object?>((_) => null, onError: (Object e) => e);

      expect(failure, isA<SocketException>());
    },
  );

  test('gateway trả HTTP 401 ở bước nâng cấp: transport ném RealtimeAuthRejected, client dừng closed(unauthorized)', () async {
    await gateway.close();
    gateway = await GatewayHarness.start(rejectBeforeUpgrade: true);
    await startClient(tokenProvider: () async => 'rt_khong-ton-tai');

    final failure = await const WebSocketTransport()
        .connect(gateway.url, token: 'rt_khong-ton-tai')
        .then<Object?>((_) => null, onError: (Object e) => e);
    expect(failure, isA<RealtimeAuthRejected>());

    client.connect();
    final state = await waitForState(
      client,
      (s) => s.status == RealtimeStatus.closed,
    );

    expect(state.cause, CloseCause.unauthorized);
    expect(transport.connects, 1);
    expect(gateway.realtime.connectionsOpened, 0);
  });

  test('dart:io cho httpStatusCode của nâng cấp bị từ chối (đây là chỗ transport đọc 401)', () async {
    await startClient();
    final refusing = await shelf_io.serve(
      (Request request) => Response(401, body: 'no'),
      InternetAddress.loopbackIPv4,
      0,
    );
    addTearDown(() => refusing.close(force: true));

    Object? failure;
    try {
      await WebSocket.connect('ws://127.0.0.1:${refusing.port}/realtime');
    } on Object catch (e) {
      failure = e;
    }

    expect(failure, isA<WebSocketException>());
    expect((failure! as WebSocketException).httpStatusCode, 401);
  });

  test('bắt tay quá connectTimeout: TimeoutException, socket nâng cấp tới muộn bị đóng', () async {
    await gateway.close();
    gateway = await GatewayHarness.start(
      latency: const Duration(milliseconds: 300),
    );
    await startClient();
    final token = await gateway.login();

    final failure =
        await const WebSocketTransport(
              connectTimeout: Duration(milliseconds: 50),
            )
            .connect(gateway.url, token: token)
            .then<Object?>((_) => null, onError: (Object e) => e);
    expect(failure, isA<TimeoutException>());

    await eventually(
      () => gateway.realtime.connectionsOpened == 1,
      reason: 'nâng cấp tới muộn',
    );
    await eventually(
      () => gateway.realtime.connectionCount == 0,
      reason: 'socket muộn phải bị đóng',
    );
  });

  test('gateway mất log: client thôi nghe chuyến và báo log_reset; subscribe lại nhận log mới', () async {
    await startClient();
    final events = <TripEventEnvelope>[];
    final issues = <RealtimeIssue>[];
    client.events.listen(events.add);
    client.issues.listen(issues.add);
    client
      ..connect()
      ..subscribe(tripId);
    await eventually(() => gateway.realtime.subscriberCount(tripId) == 1);
    gateway.realtime
      ..publish(tripId, const SearchStarted())
      ..publish(tripId, const DriverAccepted('D-7'));
    await eventually(() => events.length == 2);

    await gateway.realtime.restart();
    // Log mới: sự kiện này là #1 của logEpoch mới, version 3 của chuyến.
    gateway.realtime.publish(tripId, const DriverArrived());
    await eventually(() => issues.any((i) => i.code == 'log_reset'));
    await waitForState(client, (s) => s.status == RealtimeStatus.connected);

    expect(events.map((e) => e.sequence), [1, 2]); // không có "#1" của log mới
    expect(client.lastSequence(tripId), isNull);
    expect(gateway.realtime.subscriberCount(tripId), 0);

    client.subscribe(tripId); // app tải lại bản chụp rồi nghe lại
    await eventually(() => events.length == 3);
    expect(events.last.sequence, 1);
    expect(events.last.version, 3);
  });

  test(
    'tài xế nhận lời mời cuốc qua kênh offers; khách bị forbidden',
    () async {
      await gateway.close();
      gateway = await GatewayHarness.start();
      tripId = await gateway.createTrip();
      final driverToken = await gateway.login(app: 'driver');
      await startClient(tokenProvider: () async => driverToken);
      final offers = <DriverOffer>[];
      client.offers.listen(offers.add);
      client
        ..connect()
        ..subscribeOffers();
      await waitForState(client, (s) => s.status == RealtimeStatus.connected);
      await eventually(
        () => gateway.realtime.publishOffer('U-1001', tripId).delivered == 1,
      );
      await eventually(() => offers.isNotEmpty);

      expect(offers.first.tripId, tripId);
      expect(offers.first.fare, const Money(25500));

      final issues = <RealtimeIssue>[];
      final customer = RealtimeClient(
        transport: transport,
        url: gateway.url,
        tokenProvider: () async => gateway.login(),
        backoff: fastBackoff(),
      );
      addTearDown(customer.dispose);
      customer.issues.listen(issues.add);
      customer
        ..connect()
        ..subscribeOffers();
      await eventually(() => issues.isNotEmpty);
      expect(issues.single.code, 'forbidden');
    },
  );
}
