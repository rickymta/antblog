import 'package:flutter/material.dart';
import 'package:flutter_map/flutter_map.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/io_client.dart';
import 'package:latlong2/latlong.dart';
import 'package:toi_noi_34/demo_data.dart';
import 'package:toi_noi_34/pickup_map.dart';

import 'support.dart';

void main() {
  late MapController controller;
  late FakeTileProvider tiles;

  setUp(() {
    controller = MapController();
    tiles = FakeTileProvider();
  });

  tearDown(() => controller.dispose());

  group('không ra mạng', () {
    testWidgets('provider giả nhận đủ yêu cầu tile và không tạo HttpClient', (
      tester,
    ) async {
      final http = installHttpCounter();

      await pumpPickupMap(tester, controller: controller, tileProvider: tiles);
      await tester.pump(const Duration(seconds: 1));

      expect(tiles.requestedUrls, isNotEmpty);
      for (final url in tiles.requestedUrls) {
        expect(url, startsWith('https://tile.openstreetmap.org/16/'));
        expect(url, endsWith('.png'));
      }
      expect(http.clientsCreated, 0);
    });

    testWidgets('đối chứng: provider mặc định có đường ra HttpClient', (
      tester,
    ) async {
      final http = installHttpCounter();
      final real = NetworkTileProvider(
        httpClient: IOClient(),
        cachingProvider: const DisabledMapCachingProvider(),
        silenceExceptions: true,
      );
      addTearDown(real.dispose);

      await tester.runAsync(() async {
        await pumpPickupMap(tester, controller: controller, tileProvider: real);
        // Cho request chạy trong zone thật; override của flutter_test trả 400.
        await Future<void>.delayed(const Duration(milliseconds: 200));
      });

      expect(http.clientsCreated, greaterThan(0));
    });

    testWidgets('URL dùng HTTPS, không subdomain, User-Agent nhận diện app', (
      tester,
    ) async {
      await pumpPickupMap(tester, controller: controller, tileProvider: tiles);

      final layer = tester.widget<TileLayer>(find.byType(TileLayer));
      expect(layer.urlTemplate, osmTileUrl);
      expect(layer.urlTemplate, startsWith('https://'));
      expect(layer.urlTemplate, isNot(contains('{s}')));
      expect(
        layer.tileProvider.headers['User-Agent'],
        'flutter_map ($tileUserAgentPackageName)',
      );
      expect(
        layer.tileProvider.headers['User-Agent'],
        isNot(contains('unknown')),
      );
    });
  });

  group('attribution', () {
    testWidgets(
      'dòng attribution OpenStreetMap hiện sẵn, không cần chạm vào nút',
      (tester) async {
        await pumpPickupMap(
          tester,
          controller: controller,
          tileProvider: tiles,
        );

        expect(find.byType(SimpleAttributionWidget), findsOneWidget);
        final logo = find.text('OpenStreetMap contributors');
        expect(logo, findsOneWidget);
        // Phải nằm trong khung nhìn, không bị đẩy ra ngoài màn hình.
        final rect = tester.getRect(logo);
        final screen = tester.view.physicalSize / tester.view.devicePixelRatio;
        final window = Offset.zero & screen;
        expect(window.contains(rect.topLeft), isTrue);
        expect(window.contains(rect.bottomRight), isTrue);
      },
    );

    testWidgets('chạm vào dòng attribution gọi callback mở giấy phép', (
      tester,
    ) async {
      var taps = 0;
      await pumpPickupMap(
        tester,
        controller: controller,
        tileProvider: tiles,
        onAttributionTap: () => taps++,
      );

      await tester.tap(find.text('OpenStreetMap contributors'));
      expect(taps, 1);
    });
  });

  group('chọn điểm bằng kéo bản đồ', () {
    testWidgets('đáy pin trùng đúng tâm bản đồ', (tester) async {
      await pumpPickupMap(tester, controller: controller, tileProvider: tiles);

      final pin = tester
          .getRect(find.byKey(const ValueKey('center-pin')))
          .bottomCenter;
      final map = tester.getCenter(find.byType(FlutterMap));
      expect(pin.dx, closeTo(map.dx, 0.01));
      expect(pin.dy, closeTo(map.dy, 0.01));
    });

    testWidgets('kéo từ ngay trên pin vẫn làm bản đồ di chuyển', (
      tester,
    ) async {
      await pumpPickupMap(tester, controller: controller, tileProvider: tiles);
      final before = controller.camera.center;

      final pinCenter = tester.getCenter(
        find.byKey(const ValueKey('center-pin')),
      );
      await tester.dragFrom(pinCenter, const Offset(-120, 0));
      await tester.pump(const Duration(seconds: 1));

      // Kéo bản đồ sang trái thì tâm dịch về phía đông.
      expect(controller.camera.center.longitude, greaterThan(before.longitude));
      expect(controller.camera.center.latitude, closeTo(before.latitude, 1e-6));
    });

    testWidgets(
      'onCenterChanged báo từng bước, onPickupSettled chỉ sau khi đứng yên',
      (tester) async {
        final live = <LatLng>[];
        final settled = <LatLng>[];
        await pumpPickupMap(
          tester,
          controller: controller,
          tileProvider: tiles,
          onCenterChanged: live.add,
          onSettled: settled.add,
        );

        final gesture = await tester.startGesture(
          tester.getCenter(find.byType(FlutterMap)),
        );
        for (var i = 0; i < 5; i++) {
          await gesture.moveBy(const Offset(-20, 10));
          await tester.pump(const Duration(milliseconds: 16));
        }
        expect(live.length, greaterThanOrEqualTo(5));
        expect(settled, isEmpty, reason: 'ngón tay còn trên màn hình');

        await gesture.up();
        await tester.pump(const Duration(milliseconds: 100));
        expect(settled, isEmpty, reason: 'chưa đủ settleDelay 250 ms');

        await tester.pump(const Duration(milliseconds: 200));
        expect(settled, hasLength(1));
        expect(settled.single, controller.camera.center);
        expect(settled.single, live.last);
      },
    );

    testWidgets('vuốt nhanh (fling) vẫn chỉ có một lần settled, ở điểm cuối', (
      tester,
    ) async {
      var liveCount = 0;
      final settled = <LatLng>[];
      await pumpPickupMap(
        tester,
        controller: controller,
        tileProvider: tiles,
        onCenterChanged: (_) => liveCount++,
        onSettled: settled.add,
      );

      await tester.fling(find.byType(FlutterMap), const Offset(-150, 0), 1500);
      await tester.pumpAndSettle();
      await tester.pump(const Duration(milliseconds: 300));

      expect(
        liveCount,
        greaterThan(5),
        reason: 'camera đi tiếp sau khi nhả tay',
      );
      expect(settled, hasLength(1));
      expect(settled.single, controller.camera.center);
    });

    testWidgets(
      'move() tới điểm ngoài vùng bị kẹp đúng như ServiceArea.clamp',
      (tester) async {
        await pumpPickupMap(
          tester,
          controller: controller,
          tileProvider: tiles,
        );

        const outside = LatLng(11.5, 107.5);
        controller.move(outside, 16);
        await tester.pump(const Duration(milliseconds: 300));

        final expected = demoServiceArea.clamp(outside);
        expect(
          controller.camera.center.latitude,
          closeTo(expected.latitude, 1e-9),
        );
        expect(
          controller.camera.center.longitude,
          closeTo(expected.longitude, 1e-9),
        );
        expect(demoServiceArea.contains(controller.camera.center), isTrue);
      },
    );

    testWidgets(
      'initialCenter ngoài vùng làm flutter_map assert, nên phải kẹp trước',
      (tester) async {
        const outside = LatLng(12, 108);
        final errors = <FlutterErrorDetails>[];
        final previous = FlutterError.onError;
        FlutterError.onError = errors.add;
        try {
          await pumpPickupMap(
            tester,
            controller: controller,
            tileProvider: tiles,
            center: outside,
          );
        } finally {
          FlutterError.onError = previous;
        }

        expect(
          errors.map((e) => e.exception.toString()),
          contains(contains('no longer within the cameraConstraint')),
        );
        // Cùng điểm đó sau khi kẹp thì dựng bình thường.
        await pumpPickupMap(
          tester,
          controller: MapController(),
          tileProvider: tiles,
          center: demoServiceArea.clamp(outside),
        );
        expect(tester.takeException(), isNull);
      },
    );

    testWidgets('xoay bản đồ bị tắt: rotation vẫn bằng 0 sau cử chỉ hai ngón', (
      tester,
    ) async {
      await pumpPickupMap(tester, controller: controller, tileProvider: tiles);
      final center = tester.getCenter(find.byType(FlutterMap));

      final a = await tester.startGesture(
        center + const Offset(-60, 0),
        pointer: 1,
      );
      final b = await tester.startGesture(
        center + const Offset(60, 0),
        pointer: 2,
      );
      for (var i = 1; i <= 10; i++) {
        await a.moveTo(center + Offset(-60.0 + i * 3, i * 9.0));
        await b.moveTo(center + Offset(60.0 - i * 3, -i * 9.0));
        await tester.pump(const Duration(milliseconds: 16));
      }
      await a.up();
      await b.up();
      await tester.pump(const Duration(seconds: 1));

      expect(controller.camera.rotation, 0);
    });
  });
}
