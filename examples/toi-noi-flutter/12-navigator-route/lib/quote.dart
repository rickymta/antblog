import 'package:toi_noi_core/toi_noi_core.dart';

/// Báo giá mà màn hình báo giá giao cho màn hình xác nhận.
///
/// Chỉ giữ dữ liệu đầu vào; cước luôn tính lại bằng `calculateFare` của
/// `toi_noi_core`, nên hai màn hình không thể hiển thị hai con số khác nhau.
/// Số liệu và địa chỉ là minh hoạ.
final class Quote {
  const Quote({
    required this.id,
    required this.serviceKind,
    required this.distanceKm,
    required this.pickup,
    required this.dropoff,
  });

  final String id;
  final ServiceKind serviceKind;
  final double distanceKm;
  final String pickup;
  final String dropoff;

  /// Cước theo bảng giá minh hoạ (5 km thì 25.500 VND).
  Money get fare => calculateFare(distanceKm);
}

/// Báo giá mẫu dùng cho `main()` và test: xe máy 5 km.
const demoQuote = Quote(
  id: '0001',
  serviceKind: ServiceKind.ride,
  distanceKm: 5,
  pickup: 'Ngã tư Hoa Sữa',
  dropoff: 'Công viên Tre Xanh',
);
