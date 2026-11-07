import 'package:flutter/material.dart';
import 'package:flutter_map/flutter_map.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:latlong2/latlong.dart';
import 'package:toi_noi_34/demo_data.dart';
import 'package:toi_noi_34/geo.dart';
import 'package:toi_noi_34/pickup_screen.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

Future<void> pumpScreen(
  WidgetTester tester, {
  required FakeTileProvider tiles,
  LatLng initialPickup = demoPickup,
  LatLng dropoff = demoDropoff,
  LatLng driver = demoDriver,
  ServiceArea area = demoServiceArea,
  ValueChanged<LatLng>? onConfirm,
}) async {
  await tester.pumpWidget(
    MaterialApp(
      home: PickupScreen(
        initialPickup: initialPickup,
        dropoff: dropoff,
        driver: driver,
        serviceArea: area,
        tileProvider: tiles,
        onConfirm: onConfirm,
      ),
    ),
  );
  await tester.pump();
}

MapCamera cameraOf(WidgetTester tester) =>
    MapCamera.of(tester.element(find.byType(MarkerLayer)));

String fareLineText(WidgetTester tester) =>
    tester.widget<Text>(find.byKey(const ValueKey('fare-line'))).data!;

String coordinatesText(WidgetTester tester) =>
    tester.widget<Text>(find.byKey(const ValueKey('live-coordinates'))).data!;

/// Kéo bản đồ bằng ngón tay giữ nguyên trên màn hình rồi nhả và chờ đứng yên.
Future<void> dragMapAndSettle(WidgetTester tester, Offset by) async {
  await tester.dragFrom(tester.getCenter(find.byType(FlutterMap)), by);
  await tester.pump(const Duration(milliseconds: 300));
}

void main() {
  late FakeTileProvider tiles;

  setUp(() => tiles = FakeTileProvider());

  group('marker và polyline', () {
    testWidgets('marker tài xế đặt đúng toạ độ (alignment mặc định là tâm)', (
      tester,
    ) async {
      // Tài xế gần để còn nằm trong khung ở zoom 16.
      const near = LatLng(10.7735, 106.6995);
      await pumpScreen(tester, tiles: tiles, driver: near);

      final camera = cameraOf(tester);
      final mapOrigin = tester.getTopLeft(find.byType(FlutterMap));
      final expected = mapOrigin + camera.latLngToScreenOffset(near);
      final actual = tester.getCenter(find.byKey(driverMarkerKey));
      expect(actual.dx, closeTo(expected.dx, 0.5));
      expect(actual.dy, closeTo(expected.dy, 0.5));
    });

    testWidgets('marker ngoài khung nhìn không được dựng', (tester) async {
      await pumpScreen(tester, tiles: tiles);
      // Điểm đến cách điểm đón hơn 9 km, xa khỏi khung 800 px ở zoom 16.
      expect(find.byKey(dropoffMarkerKey), findsNothing);
    });

    testWidgets('marker điểm đến dùng topCenter nên đáy chạm đúng toạ độ', (
      tester,
    ) async {
      // Điểm đến gần để còn nằm trong khung ở zoom 16.
      const near = LatLng(10.7752, 106.7010);
      await pumpScreen(tester, tiles: tiles, dropoff: near);

      final camera = cameraOf(tester);
      final mapOrigin = tester.getTopLeft(find.byType(FlutterMap));
      final expected = mapOrigin + camera.latLngToScreenOffset(near);
      final rect = tester.getRect(find.byKey(dropoffMarkerKey));
      expect(rect.bottomCenter.dx, closeTo(expected.dx, 0.5));
      expect(rect.bottomCenter.dy, closeTo(expected.dy, 0.5));
    });

    testWidgets('hai polyline: tài xế tới điểm đón và điểm đón tới điểm đến', (
      tester,
    ) async {
      await pumpScreen(tester, tiles: tiles);

      final layer = tester.widget<PolylineLayer>(find.byType(PolylineLayer));
      expect(layer.polylines, hasLength(2));
      expect(layer.polylines[0].points, fakeRoute(demoDriver, demoPickup));
      expect(layer.polylines[1].points, fakeRoute(demoPickup, demoDropoff));
    });

    testWidgets('kéo xong và đứng yên thì polyline đi từ điểm đón mới', (
      tester,
    ) async {
      await pumpScreen(tester, tiles: tiles);

      await dragMapAndSettle(tester, const Offset(-100, 40));

      final picked = cameraOf(tester).center;
      expect(picked, isNot(demoPickup));
      final layer = tester.widget<PolylineLayer>(find.byType(PolylineLayer));
      expect(layer.polylines[0].points.last, picked);
      expect(layer.polylines[1].points.first, picked);
    });
  });

  group('thẻ cước', () {
    testWidgets('cước hiển thị khớp calculateFare của toi_noi_core', (
      tester,
    ) async {
      await pumpScreen(tester, tiles: tiles);

      final meters = routeMeters(fakeRoute(demoPickup, demoDropoff));
      final fare = calculateFare(meters / 1000);
      expect(fareLineText(tester), contains(fare.toString()));
      expect(
        fareLineText(tester),
        contains('${(meters / 1000).toStringAsFixed(1)} km'),
      );
    });

    testWidgets(
      'toạ độ cập nhật ngay khi kéo, cước và nút xác nhận chờ đứng yên',
      (tester) async {
        LatLng? confirmed;
        await pumpScreen(tester, tiles: tiles, onConfirm: (p) => confirmed = p);
        final fareBefore = fareLineText(tester);
        final coordinatesBefore = coordinatesText(tester);
        FilledButton confirm() =>
            tester.widget<FilledButton>(find.byKey(confirmButtonKey));
        expect(confirm().onPressed, isNotNull);

        final gesture = await tester.startGesture(
          tester.getCenter(find.byType(FlutterMap)),
        );
        await gesture.moveBy(const Offset(-80, 0));
        await tester.pump(const Duration(milliseconds: 16));
        await gesture.moveBy(const Offset(-80, 0));
        await tester.pump(const Duration(milliseconds: 16));

        expect(coordinatesText(tester), isNot(coordinatesBefore));
        expect(fareLineText(tester), fareBefore, reason: 'chưa settle');
        expect(
          confirm().onPressed,
          isNull,
          reason: 'đang kéo thì chưa xác nhận',
        );

        await gesture.up();
        await tester.pump(const Duration(milliseconds: 300));

        expect(fareLineText(tester), isNot(fareBefore));
        expect(confirm().onPressed, isNotNull);
        await tester.tap(find.byKey(confirmButtonKey));
        expect(confirmed, cameraOf(tester).center);
      },
    );

    testWidgets('điểm đến ngoài vùng phục vụ có thông báo riêng', (
      tester,
    ) async {
      await pumpScreen(
        tester,
        tiles: tiles,
        dropoff: const LatLng(21.0285, 105.8542),
      );
      expect(fareLineText(tester), 'Điểm đến ngoài vùng phục vụ');
    });

    testWidgets('quãng đường vượt 100 km báo chưa báo giá được', (
      tester,
    ) async {
      await pumpScreen(
        tester,
        tiles: tiles,
        area: nationwideArea,
        dropoff: const LatLng(21.0285, 105.8542),
      );
      expect(fareLineText(tester), contains('vượt 100 km'));
      expect(fareLineText(tester), isNot(contains('VND')));
    });
  });

  group('vùng phục vụ và nút về điểm ban đầu', () {
    testWidgets('điểm đón ban đầu ngoài vùng được kẹp trước khi dựng bản đồ', (
      tester,
    ) async {
      await pumpScreen(
        tester,
        tiles: tiles,
        initialPickup: const LatLng(12, 108),
      );

      expect(coordinatesText(tester), 'Điểm đón: 10.90000, 106.85000');
      expect(
        demoServiceArea.contains(cameraOf(tester).center),
        isTrue,
        reason: 'camera cũng phải bắt đầu trong vùng',
      );
    });

    testWidgets('nút "về điểm đón ban đầu" đưa camera và điểm đón trở lại', (
      tester,
    ) async {
      await pumpScreen(tester, tiles: tiles);
      await dragMapAndSettle(tester, const Offset(-150, 0));
      expect(cameraOf(tester).center, isNot(demoPickup));

      await tester.tap(find.byKey(myLocationButtonKey));
      await tester.pump(const Duration(milliseconds: 300));

      expect(cameraOf(tester).center, demoPickup);
      expect(coordinatesText(tester), 'Điểm đón: 10.77210, 106.69800');
    });
  });
}
