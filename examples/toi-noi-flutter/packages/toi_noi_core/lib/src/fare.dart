import 'dart:math' show max;

import 'money.dart';

/// Bảng giá minh hoạ của Tới Nơi (không phải bảng giá thực tế).
///
/// Dùng `abstract final class` làm "không gian tên": không tạo được đối tượng,
/// không bị kế thừa, và `FarePolicy.maxKm` không chen vào tên toàn cục.
abstract final class FarePolicy {
  /// Giá mở cửa, đã gồm [includedMeters] đầu.
  static const int baseFareVnd = 12000;

  /// Số mét đã nằm trong giá mở cửa.
  static const int includedMeters = 2000;

  /// Giá mỗi km vượt.
  static const int pricePerKmVnd = 4500;

  /// Quãng đường tối đa hợp lệ.
  static const int maxKm = 100;
}

/// Tính cước bằng số đồng cho quãng đường [distanceKm], miền hợp lệ 0..100 km.
///
/// Quy đổi sang mét nguyên một lần, rồi chỉ tính bằng số nguyên để kết quả
/// không phụ thuộc sai số `double`. Phần vượt làm tròn nửa đồng lên.
/// Ném [ArgumentError] khi ngoài miền, kể cả NaN và vô cực.
int calculateFareVnd(double distanceKm) {
  // NaN so sánh với số nào cũng cho false, nên guard viết dưới dạng
  // "điều kiện hợp lệ" rồi phủ định.
  final isValid = distanceKm >= 0 && distanceKm <= FarePolicy.maxKm;
  if (!isValid) {
    throw ArgumentError.value(
      distanceKm,
      'distanceKm',
      'phải nằm trong khoảng 0..${FarePolicy.maxKm} km',
    );
  }

  final meters = (distanceKm * 1000).round();
  final extraMeters = max(0, meters - FarePolicy.includedMeters);
  return FarePolicy.baseFareVnd +
      (extraMeters * FarePolicy.pricePerKmVnd + 500) ~/ 1000;
}

/// Giống [calculateFareVnd] nhưng trả [Money] thay vì `int` trần.
Money calculateFare(double distanceKm) => Money(calculateFareVnd(distanceKm));
