import 'package:toi_noi_core/toi_noi_core.dart';

/// Báo giá do server trả về cho một yêu cầu (dịch vụ, quãng đường).
///
/// Bất biến và có `==` theo giá trị: Riverpod 3 dùng `==` để quyết định có
/// báo cho listener hay không. Số liệu là minh hoạ.
final class Quote {
  const Quote({
    required this.serviceKind,
    required this.distanceKm,
    required this.fare,
  });

  final ServiceKind serviceKind;
  final double distanceKm;
  final Money fare;

  @override
  bool operator ==(Object other) =>
      other is Quote &&
      other.serviceKind == serviceKind &&
      other.distanceKm == distanceKm &&
      other.fare == fare;

  @override
  int get hashCode => Object.hash(serviceKind, distanceKm, fare);

  @override
  String toString() => 'Quote(${serviceKind.name}, $distanceKm km, $fare)';
}
