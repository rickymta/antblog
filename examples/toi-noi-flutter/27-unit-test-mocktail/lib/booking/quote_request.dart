import 'package:toi_noi_core/toi_noi_core.dart';

import 'trip_failure.dart';

/// Điều khách muốn: dịch vụ nào, đi bao xa.
final class QuoteRequest {
  const QuoteRequest({required this.serviceKind, required this.distanceKm});

  final ServiceKind serviceKind;
  final double distanceKm;

  /// Nằm trong miền 0 đến [FarePolicy.maxKm] km. Viết dạng "điều kiện hợp lệ"
  /// để NaN cũng bị loại.
  bool get isWithinRange => distanceKm >= 0 && distanceKm <= FarePolicy.maxKm;

  /// Ném [Rejected] (`invalidDistance`) khi ngoài miền.
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
