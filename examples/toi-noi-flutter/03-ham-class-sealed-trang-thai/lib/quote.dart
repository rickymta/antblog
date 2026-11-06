import 'dart:math' as math;

const int _baseFareVnd = 12000;
const int _includedMeters = 2000;
const int _pricePerExtraKmVnd = 4500;
const double _maxDistanceKm = 100;

/// Báo giá minh hoạ: trả một record đặt tên thay vì tạo class riêng.
///
/// 12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt (số minh hoạ, không phải
/// bảng giá thật). Ném [ArgumentError] khi [distanceKm] ngoài miền 0 đến 100.
({int fareVnd, double distanceKm}) quoteFor(double distanceKm) {
  if (distanceKm < 0 || distanceKm > _maxDistanceKm) {
    throw ArgumentError.value(
      distanceKm,
      'distanceKm',
      'phải nằm trong khoảng 0 đến 100 km',
    );
  }
  // Tính bằng mét nguyên để tránh sai số số thực; cộng 500 rồi chia nguyên
  // cho 1000 là làm tròn nửa đồng lên.
  final meters = (distanceKm * 1000).round();
  final extraMeters = math.max(0, meters - _includedMeters);
  final fare = _baseFareVnd + (extraMeters * _pricePerExtraKmVnd + 500) ~/ 1000;
  return (fareVnd: fare, distanceKm: distanceKm);
}
