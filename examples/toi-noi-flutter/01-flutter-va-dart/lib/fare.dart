/// Bảng giá minh hoạ của Tới Nơi, không phải bảng giá thực tế.
const int _baseFareVnd = 12000;
const double _includedKm = 2;
const int _pricePerExtraKmVnd = 4500;
const double _maxDistanceKm = 100;

/// Tính cước (đồng VND) cho quãng đường [distanceKm].
///
/// 12.000 VND cho 2 km đầu, 4.500 VND cho mỗi km vượt, làm tròn bằng
/// `round()`. Ném [ArgumentError] khi khoảng cách ngoài miền 0 đến 100 km.
int calculateFareVnd(double distanceKm) {
  if (distanceKm < 0 || distanceKm > _maxDistanceKm) {
    throw ArgumentError.value(
      distanceKm,
      'distanceKm',
      'phải nằm trong khoảng 0 đến 100 km',
    );
  }
  final extraKm = distanceKm > _includedKm ? distanceKm - _includedKm : 0.0;
  return (_baseFareVnd + extraKm * _pricePerExtraKmVnd).round();
}
