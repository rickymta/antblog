import 'package:toi_noi_core/toi_noi_core.dart';

import 'trip_failure.dart';

/// Điều khách muốn: dịch vụ nào, đi bao xa.
///
/// Đây là ngôn ngữ của domain. Hình dạng dây dẫn của cùng thông tin này là
/// `QuoteRequestDto` trong `toi_noi_api`, và lớp data lo việc đổi qua lại.
final class QuoteRequest {
  const QuoteRequest({required this.serviceKind, required this.distanceKm});

  final ServiceKind serviceKind;

  /// Quãng đường theo km như khách nhập.
  final double distanceKm;

  /// Quãng đường nằm trong miền bảng giá phục vụ (0 đến [FarePolicy.maxKm] km).
  ///
  /// Viết dạng "điều kiện hợp lệ" để NaN cũng bị loại.
  bool get isWithinRange => distanceKm >= 0 && distanceKm <= FarePolicy.maxKm;

  /// Ném [Rejected] (`invalidDistance`) khi ngoài miền. Quy tắc nằm ở đây để
  /// báo giá và đặt chuyến dùng chung, không phải ở một use case.
  void requireWithinRange() {
    if (!isWithinRange) {
      throw const Rejected(RejectionKind.invalidDistance);
    }
  }

  @override
  bool operator ==(Object other) =>
      other is QuoteRequest &&
      other.serviceKind == serviceKind &&
      other.distanceKm == distanceKm;

  @override
  int get hashCode => Object.hash(serviceKind, distanceKm);

  @override
  String toString() => 'QuoteRequest(${serviceKind.name}, $distanceKm km)';
}
