import 'package:toi_noi_core/toi_noi_core.dart';

/// Dữ liệu một yêu cầu chuyến sau khi form đã kiểm và chuẩn hoá.
///
/// Địa chỉ đã gọn khoảng trắng; khoảng cách là `double` km (đã đổi dấu phẩy
/// thành dấu chấm). Số mét nguyên và cước do `toi_noi_core` tính khi cần.
final class TripRequestData {
  const TripRequestData({
    required this.pickup,
    required this.destination,
    required this.distanceKm,
  });

  final String pickup;
  final String destination;
  final double distanceKm;

  /// Cước ước tính theo bảng giá minh hoạ của `toi_noi_core`.
  Money get estimatedFare => calculateFare(distanceKm);

  @override
  bool operator ==(Object other) =>
      other is TripRequestData &&
      other.pickup == pickup &&
      other.destination == destination &&
      other.distanceKm == distanceKm;

  @override
  int get hashCode => Object.hash(pickup, destination, distanceKm);

  @override
  String toString() =>
      'TripRequestData($pickup -> $destination, $distanceKm km)';
}
