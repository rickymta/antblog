import 'dart:math' show max;

// Bảng giá minh hoạ của Tới Nơi, không phải bảng giá thực tế.
const int baseFareVnd = 12000; // đã gồm 2 km đầu
const int includedMeters = 2000;
const int pricePerExtraKmVnd = 4500;
const int maxDistanceKm = 100;
const int metersPerKm = 1000;

/// Tính cước (đồng VND) cho quãng đường [distanceKm], miền hợp lệ 0..100 km.
///
/// Ném [ArgumentError] khi ngoài miền, kể cả NaN. `round()` chỉ dùng một lần
/// ở biên km -> mét (độ phân giải 1 m); sau đó mọi phép tính là `int`.
int calculateFareVnd(double distanceKm) {
  // NaN so sánh với số nào cũng cho false, nên guard viết dưới dạng
  // "điều kiện hợp lệ" rồi phủ định. Guard phải đứng trước `round()` vì
  // NaN.round() và Infinity.round() ném UnsupportedError.
  final isValid = distanceKm >= 0 && distanceKm <= maxDistanceKm;
  if (!isValid) {
    throw ArgumentError.value(
      distanceKm,
      'distanceKm',
      'phải nằm trong khoảng 0..$maxDistanceKm km',
    );
  }

  final meters = (distanceKm * metersPerKm).round();
  final extraMeters = max(0, meters - includedMeters);
  // (+ 500) ~/ 1000 là làm tròn nửa lên cho số không âm: 4,5 đồng -> 5 đồng.
  return baseFareVnd + (extraMeters * pricePerExtraKmVnd + 500) ~/ metersPerKm;
}

/// Định dạng số đồng với dấu chấm nghìn: 25500 -> "25.500 VND".
///
/// Chèn tay để chưa phải dùng package `intl` (bài về localization sẽ thay thế).
String formatVnd(int vnd) {
  final digits = vnd.abs().toString();
  final buffer = StringBuffer();
  for (var i = 0; i < digits.length; i++) {
    final remaining = digits.length - i;
    if (i > 0 && remaining % 3 == 0) {
      buffer.write('.');
    }
    buffer.write(digits[i]);
  }
  final sign = vnd < 0 ? '-' : '';
  return '$sign$buffer VND';
}
