import 'dart:math' as math;

import 'package:latlong2/latlong.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Bán kính trái đất trung bình (IUGG) tính bằng mét, dùng cho haversine.
const double earthRadiusMeters = 6371008.8;

void _checkCoordinate(LatLng point, String name) {
  // `isValid` viết dưới dạng điều kiện hợp lệ nên NaN cũng bị loại.
  if (!point.isValid) {
    throw ArgumentError.value(
      point,
      name,
      'toạ độ ngoài miền hoặc không hữu hạn',
    );
  }
}

/// Khoảng cách đường chim bay giữa [a] và [b] bằng công thức haversine, đơn vị
/// mét, chưa làm tròn.
///
/// Ném [ArgumentError] khi toạ độ không hợp lệ. `package:latlong2` thì trả 0
/// âm thầm cho NaN, nên một toạ độ hỏng sẽ thành "cách nhau 0 m".
double haversineMeters(
  LatLng a,
  LatLng b, {
  double radius = earthRadiusMeters,
}) {
  _checkCoordinate(a, 'a');
  _checkCoordinate(b, 'b');
  final sinDLat = math.sin((b.latitudeInRad - a.latitudeInRad) / 2);
  final sinDLng = math.sin((b.longitudeInRad - a.longitudeInRad) / 2);
  final h =
      sinDLat * sinDLat +
      sinDLng * sinDLng * math.cos(a.latitudeInRad) * math.cos(b.latitudeInRad);
  return 2 * radius * math.atan2(math.sqrt(h), math.sqrt(1 - h));
}

/// Khoảng cách chim bay làm tròn tới mét nguyên, đúng đơn vị của cước Tới Nơi.
int distanceMeters(LatLng a, LatLng b) => haversineMeters(a, b).round();

/// Độ dài của đường gấp khúc [points], mét nguyên.
///
/// Cộng các đoạn bằng `double` rồi làm tròn một lần ở cuối để sai số làm tròn
/// không tích luỹ theo số đoạn. Ít hơn hai điểm thì dài 0 m.
int routeMeters(List<LatLng> points) {
  var total = 0.0;
  for (var i = 1; i < points.length; i++) {
    total += haversineMeters(points[i - 1], points[i]);
  }
  return total.round();
}

/// Đường đi giả cho lab: đi theo kinh độ rồi theo vĩ độ, như trên bàn cờ phố.
///
/// Thay cho dịch vụ định tuyến của backend. Luôn dài hơn hoặc bằng đường chim
/// bay, và không bao giờ là cước chính thức.
List<LatLng> fakeRoute(LatLng from, LatLng to) => [
  from,
  LatLng(from.latitude, to.longitude),
  to,
];

/// Kết quả ước tính cước từ một quãng đường.
sealed class FareEstimate {
  const FareEstimate(this.meters);

  /// Quãng đường đã dùng để tính, mét nguyên.
  final int meters;
}

/// Quãng đường nằm trong miền hợp lệ của bảng giá.
final class FareQuoted extends FareEstimate {
  const FareQuoted(super.meters, this.fare);

  final Money fare;
}

/// Quãng đường vượt miền 0..100 km của bảng giá minh hoạ.
final class FareOutOfRange extends FareEstimate {
  const FareOutOfRange(super.meters);
}

/// Tính cước ước tính cho [meters] mét bằng `calculateFare` của `toi_noi_core`.
///
/// Không tự đặt công thức: chỉ đổi mét sang km rồi gọi hàm chung. Vượt
/// 100 km trả [FareOutOfRange] thay vì để `ArgumentError` rơi vào giao diện.
FareEstimate estimateFare(int meters) {
  if (meters < 0) {
    throw ArgumentError.value(meters, 'meters', 'không được âm');
  }
  if (meters > FarePolicy.maxKm * 1000) {
    return FareOutOfRange(meters);
  }
  return FareQuoted(meters, calculateFare(meters / 1000));
}

/// Hình chữ nhật toạ độ mà Tới Nơi nhận chuyến.
///
/// Lưu bốn số `double` thay vì hai `LatLng` để constructor vẫn là `const` và
/// `assert` đọc được các cạnh.
final class ServiceArea {
  const ServiceArea({
    required this.south,
    required this.west,
    required this.north,
    required this.east,
  }) : assert(south <= north, 'south phải nhỏ hơn hoặc bằng north'),
       assert(west <= east, 'west phải nhỏ hơn hoặc bằng east');

  final double south;
  final double west;
  final double north;
  final double east;

  LatLng get southWest => LatLng(south, west);
  LatLng get northEast => LatLng(north, east);

  /// Tâm hình chữ nhật.
  LatLng get center => LatLng((south + north) / 2, (west + east) / 2);

  /// `true` khi [point] nằm trong vùng, kể cả đúng trên biên.
  bool contains(LatLng point) {
    _checkCoordinate(point, 'point');
    return point.latitude >= south &&
        point.latitude <= north &&
        point.longitude >= west &&
        point.longitude <= east;
  }

  /// Điểm gần [point] nhất nằm trong vùng; điểm đã nằm trong thì giữ nguyên.
  LatLng clamp(LatLng point) {
    _checkCoordinate(point, 'point');
    return LatLng(
      point.latitude.clamp(south, north),
      point.longitude.clamp(west, east),
    );
  }
}
