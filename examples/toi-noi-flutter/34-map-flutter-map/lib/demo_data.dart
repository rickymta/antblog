import 'package:latlong2/latlong.dart';

import 'geo.dart';

/// Vùng phục vụ minh hoạ quanh trung tâm TP. Hồ Chí Minh (toạ độ gần đúng,
/// chọn cho dễ nhìn trên bản đồ, không phải ranh giới dịch vụ thực).
const ServiceArea demoServiceArea = ServiceArea(
  south: 10.60,
  west: 106.55,
  north: 10.90,
  east: 106.85,
);

/// Điểm đón mặc định khi mở màn hình.
const LatLng demoPickup = LatLng(10.7721, 106.6980);

/// Điểm đến của chuyến minh hoạ.
const LatLng demoDropoff = LatLng(10.8231, 106.6297);

/// Vị trí tài xế gần điểm đón.
const LatLng demoDriver = LatLng(10.7790, 106.7050);
