import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shelf/shelf.dart';
import 'package:toi_noi_31/keys.dart';
import 'package:toi_noi_31/notification_permission.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import '../test_support/booking_flow.dart';
import '../test_support/fake_notification_permission.dart';
import '../test_support/harness.dart';

/// Đếm request theo `METHOD /đường-dẫn` đã tới "server".
List<String> _calls(Harness h) => [
  for (final r in h.adapter.requests) '${r.method} ${r.uri.path}',
];

int _count(Harness h, String call) => _calls(h).where((c) => c == call).length;

/// Bọc fake API: [fail] trả về `true` thì request nhận 503 thay vì tới fake.
Handler _failing(bool Function(Request request) fail) {
  var seq = 0;
  final inner = createFakeApiHandler(
    clock: () => DateTime.utc(2026, 11, 6, 3, 30),
    newId: () => 'T-${++seq}',
  );
  return (request) => fail(request)
      ? Response(
          503,
          headers: {'content-type': 'application/problem+json'},
          body: '{"title":"Service Unavailable","status":503}',
        )
      : inner(request);
}

void main() {
  group('runBookingFlow trên flutter test (adapter giả, đồng hồ giả)', () {
    testWidgets('đặt xe 5 km: báo giá, đặt, bật thông báo, tài xế nhận', (
      tester,
    ) async {
      final h = Harness();
      await tester.pumpWidget(h.app);

      final shots = <String>[];
      final outcome = await runBookingFlow(
        tester,
        driverAccepts: h.driver.accept,
        screenshot: (name) async => shots.add(name),
      );

      expect(outcome.tripId, 'T-1');
      expect(outcome.fare, const Money(25500));
      expect(outcome.driverId, 'D-01');
      expect(shots, ['1-bao-gia', '2-dang-tim-tai-xe', '3-tai-xe-da-nhan']);
      expect(h.permission.requests, 1);

      // Điều server đã nhận: đúng thứ tự, đúng khoá idempotency, đúng tài xế.
      final calls = _calls(h);
      expect(calls.first, 'POST /quotes');
      expect(calls[1], 'POST /trips');
      expect(
        calls.sublist(2).where((c) => c == 'GET /trips/T-1').length,
        greaterThanOrEqualTo(2),
        reason: 'app hỏi lại trạng thái cho tới khi có tài xế',
      );
      final accept = calls.indexOf('POST /trips/T-1/accept');
      expect(accept, greaterThan(2), reason: 'tài xế nhận sau vài lần hỏi');
      expect(calls.sublist(accept + 1), [
        'GET /trips/T-1',
      ], reason: 'đúng một lần hỏi nữa thì màn hình thấy tài xế và dừng');
      final acceptRequest = h.adapter.requests[accept];
      expect(acceptRequest.headers['x-fake-driver-id'], 'D-01');
      expect(acceptRequest.body, '{"expectedVersion":0}');
    });

    for (final (service, km, vnd, shown) in [
      (ServiceKind.ride, 2.001, 12005, '12.005 VND'),
      (ServiceKind.food, 100.0, 453000, '453.000 VND'),
      (ServiceKind.parcel, 5.0, 25500, '25.500 VND'),
    ]) {
      testWidgets('${service.name} $km km hiện đúng $shown', (tester) async {
        final h = Harness();
        await tester.pumpWidget(h.app);

        final outcome = await runBookingFlow(
          tester,
          driverAccepts: h.driver.accept,
          service: service,
          distanceKm: km,
        );

        expect(outcome.fare, Money(vnd));
        expect(outcome.fare, calculateFare(km));
        expect(find.text('Giá: $shown'), findsOneWidget);
        final create = h.adapter.requests.firstWhere(
          (r) => r.method == 'POST' && r.uri.path == '/trips',
        );
        expect(create.body, contains('"serviceKind":"${service.name}"'));
      });
    }

    testWidgets('người dùng từ chối thông báo: luồng vẫn đi tiếp', (
      tester,
    ) async {
      final h = Harness(
        permission: FakeNotificationPermission(
          answer: NotificationPermissionStatus.denied,
        ),
      );
      await tester.pumpWidget(h.app);

      await requestQuote(tester);
      final tripId = await placeBooking(tester);
      await askForNotifications(
        tester,
        expected: NotificationPermissionStatus.denied,
      );
      expect(
        find.byKey(AppKeys.enableNotificationsButton),
        findsOneWidget,
        reason: 'chưa bật thì nút vẫn còn để hỏi lại',
      );

      await actAsDriver(tester, () => h.driver.accept(tripId));
      expect(await waitForDriver(tester), 'D-01');
    });

    testWidgets('quyền đã có từ trước: không hiện nút bật thông báo', (
      tester,
    ) async {
      final h = Harness(
        permission: FakeNotificationPermission(
          current: NotificationPermissionStatus.granted,
        ),
      );
      await tester.pumpWidget(h.app);

      await requestQuote(tester);
      await placeBooking(tester);
      await pumpUntilFound(tester, find.text('Thông báo: đã bật'));
      expect(find.byKey(AppKeys.enableNotificationsButton), findsNothing);
      expect(h.permission.requests, 0);

      // Dọn: để app hết polling trước khi test kết thúc.
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.pump(h.pollInterval);
    });
  });

  group('lỗi mạng và lỗi server', () {
    testWidgets('quãng đường 150 km bị server từ chối, không có nút đặt', (
      tester,
    ) async {
      final h = Harness();
      await tester.pumpWidget(h.app);

      await tester.enterText(find.byKey(AppKeys.distanceField), '150');
      await tester.tap(find.byKey(AppKeys.quoteButton));
      await pumpUntilFound(tester, find.byKey(AppKeys.errorText));

      expect(
        find.text('Quãng đường phải nằm trong khoảng 0 đến 100 km.'),
        findsOneWidget,
      );
      expect(find.byKey(AppKeys.bookButton), findsNothing);
      expect(_count(h, 'POST /trips'), 0);
    });

    testWidgets('đặt chuyến gặp 503: giữ giá, bấm lại dùng cùng khoá', (
      tester,
    ) async {
      var failCreate = true;
      final h = Harness(
        handler: _failing(
          (r) => failCreate && r.method == 'POST' && r.url.path == 'trips',
        ),
      );
      await tester.pumpWidget(h.app);

      await requestQuote(tester);
      await tester.tap(find.byKey(AppKeys.bookButton));
      await pumpUntilFound(tester, find.byKey(AppKeys.errorText));
      expect(
        find.text('Máy chủ đang gặp sự cố. Hãy thử lại sau ít phút.'),
        findsOneWidget,
      );
      expect(find.byKey(AppKeys.bookButton), findsOneWidget);
      expect(find.byKey(AppKeys.tripIdText), findsNothing);

      failCreate = false;
      final tripId = await placeBooking(tester);
      expect(tripId, 'T-1');
      final keys = [
        for (final r in h.adapter.requests)
          if (r.method == 'POST' && r.uri.path == '/trips')
            r.headers['idempotency-key'],
      ];
      expect(keys, ['test-key-1', 'test-key-1']);

      await tester.pumpWidget(const SizedBox.shrink());
      await tester.pump(h.pollInterval);
    });

    testWidgets('mất kết nối khi hỏi trạng thái: hiện lỗi, Thử lại chạy tiếp', (
      tester,
    ) async {
      var failPolling = false;
      final h = Harness(
        handler: _failing(
          (r) => failPolling && r.method == 'GET' && r.url.path == 'trips/T-1',
        ),
      );
      await tester.pumpWidget(h.app);

      await requestQuote(tester);
      final tripId = await placeBooking(tester);
      failPolling = true;
      await pumpUntilFound(tester, find.byKey(AppKeys.errorText));
      expect(find.text('Thử lại'), findsOneWidget);
      expect(find.byKey(AppKeys.tripStatusText), findsNothing);

      failPolling = false;
      await actAsDriver(tester, () => h.driver.accept(tripId));
      await tester.tap(find.text('Thử lại'));
      expect(await waitForDriver(tester), 'D-01');
    });
  });

  group('chờ tài xế', () {
    testWidgets('pumpAndSettle không bao giờ yên khi còn thanh tiến trình', (
      tester,
    ) async {
      final h = Harness();
      await tester.pumpWidget(h.app);
      await requestQuote(tester);
      await placeBooking(tester);

      expect(find.byType(LinearProgressIndicator), findsOneWidget);
      Object? error;
      try {
        await tester.pumpAndSettle(
          const Duration(milliseconds: 100),
          EnginePhase.sendSemanticsUpdate,
          const Duration(seconds: 2),
        );
      } catch (e) {
        error = e;
      }
      expect(
        error,
        isA<FlutterError>().having(
          (e) => e.message,
          'message',
          'pumpAndSettle timed out',
        ),
      );

      await tester.pumpWidget(const SizedBox.shrink());
      await tester.pump(h.pollInterval);
    });

    testWidgets('rời màn hình thì ngừng hỏi trạng thái', (tester) async {
      final h = Harness();
      await tester.pumpWidget(h.app);
      await requestQuote(tester);
      await placeBooking(tester);
      await tester.pump(h.pollInterval * 3);
      expect(_count(h, 'GET /trips/T-1'), greaterThanOrEqualTo(3));

      await tester.pumpWidget(const SizedBox.shrink());
      await tester.pump(h.pollInterval); // xả Future.delayed còn treo
      final after = _count(h, 'GET /trips/T-1');
      await tester.pump(h.pollInterval * 5);
      expect(_count(h, 'GET /trips/T-1'), after);
    });

    testWidgets('pumpUntilFound báo lỗi rõ khi hết giờ', (tester) async {
      await tester.pumpWidget(const MaterialApp(home: SizedBox()));
      Object? error;
      try {
        await pumpUntilFound(
          tester,
          find.text('không bao giờ có'),
          timeout: const Duration(seconds: 1),
        );
      } catch (e) {
        error = e;
      }
      expect(
        error,
        isA<TestFailure>().having(
          (e) => e.message,
          'message',
          allOf(contains('sau 1 giây'), contains('không bao giờ có')),
        ),
      );
    });
  });
}
