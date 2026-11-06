import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_map/flutter_map.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:latlong2/latlong.dart';
import 'package:toi_noi_34/geo.dart';
import 'package:toi_noi_34/pickup_map.dart';

/// Tile provider giả: ghi lại URL mà flutter_map sẽ gọi, trả ảnh trong suốt có
/// sẵn trong bộ nhớ. Không đụng vào mạng hay đĩa.
class FakeTileProvider extends TileProvider {
  final requestedUrls = <String>[];

  @override
  ImageProvider getImage(TileCoordinates coordinates, TileLayer options) {
    requestedUrls.add(getTileUrl(coordinates, options));
    return MemoryImage(TileProvider.transparentImage);
  }
}

/// Đếm số lần có mã tạo `HttpClient` (đường đi của mọi request ra mạng qua
/// `dart:io`). Vẫn chuyển cho override chặn mạng của flutter_test, nên bản thân
/// bộ đếm không bao giờ tạo kết nối thật.
class CountingHttpOverrides extends HttpOverrides {
  CountingHttpOverrides(this._inner);

  final HttpOverrides _inner;
  int clientsCreated = 0;

  @override
  HttpClient createHttpClient(SecurityContext? context) {
    clientsCreated++;
    return _inner.createHttpClient(context);
  }
}

/// Cài bộ đếm cho đến hết test hiện tại.
CountingHttpOverrides installHttpCounter() {
  final previous = HttpOverrides.current;
  if (previous == null) {
    throw StateError('flutter_test phải đã cài override chặn mạng');
  }
  final counter = CountingHttpOverrides(previous);
  HttpOverrides.global = counter;
  addTearDown(() => HttpOverrides.global = previous);
  return counter;
}

/// Vùng phục vụ đủ rộng để tuyến vượt 100 km (từ TP. HCM ra Hà Nội).
const nationwideArea = ServiceArea(
  south: 8.0,
  west: 102.0,
  north: 23.5,
  east: 110.0,
);

/// Dựng [PickupMap] trần với provider giả; trả về để test đọc tiếp.
Future<void> pumpPickupMap(
  WidgetTester tester, {
  required MapController controller,
  required TileProvider tileProvider,
  LatLng center = const LatLng(10.7721, 106.6980),
  ServiceArea area = const ServiceArea(
    south: 10.60,
    west: 106.55,
    north: 10.90,
    east: 106.85,
  ),
  ValueChanged<LatLng>? onCenterChanged,
  ValueChanged<LatLng>? onSettled,
  VoidCallback? onAttributionTap,
}) async {
  await tester.pumpWidget(
    MaterialApp(
      home: Scaffold(
        body: PickupMap(
          controller: controller,
          initialCenter: center,
          serviceArea: area,
          tileProvider: tileProvider,
          onCenterChanged: onCenterChanged,
          onPickupSettled: onSettled ?? (_) {},
          onAttributionTap: onAttributionTap,
        ),
      ),
    ),
  );
  // Nhịp thứ hai cho sự kiện đổi kích thước của FlutterMap (postFrame).
  await tester.pump();
}
