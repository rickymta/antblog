import 'package:toi_noi_core/toi_noi_core.dart';

/// Tài xế hiển thị trên màn hình tìm tài xế. Dữ liệu hư cấu.
class Driver {
  const Driver({
    required this.id,
    required this.name,
    required this.vehicle,
    required this.plate,
    required this.rating,
    required this.trips,
    required this.etaMinutes,
  });

  final String id;
  final String name;
  final String vehicle;
  final String plate;

  /// Điểm đánh giá 1,0 đến 5,0 (số minh hoạ).
  final double rating;

  /// Số chuyến đã chạy (số minh hoạ).
  final int trips;

  /// Số phút ước tính tới điểm đón (số minh hoạ).
  final int etaMinutes;

  /// Chữ cái đầu của tên, hiển thị trong avatar.
  String get initial => name.trim().isEmpty ? '?' : name.trim()[0];
}

/// Ba tài xế hư cấu của lab.
const List<Driver> kDrivers = [
  Driver(
    id: 'drv-001',
    name: 'Trần Minh Khoa',
    vehicle: 'Honda Vision',
    plate: '59X1-000.01',
    rating: 4.9,
    trips: 1280,
    etaMinutes: 3,
  ),
  Driver(
    id: 'drv-002',
    name: 'Lê Thu Hà',
    vehicle: 'Yamaha Janus',
    plate: '59X1-000.02',
    rating: 4.8,
    trips: 640,
    etaMinutes: 5,
  ),
  Driver(
    id: 'drv-003',
    name: 'Phạm Quốc Bảo',
    vehicle: 'Honda Lead',
    plate: '59X1-000.03',
    rating: 4.7,
    trips: 312,
    etaMinutes: 4,
  ),
];

/// Tìm tài xế theo [id]; `null` khi chưa có tài xế hoặc id lạ.
Driver? findDriver(String? id) {
  for (final driver in kDrivers) {
    if (driver.id == id) return driver;
  }
  return null;
}

/// Tag của `Hero` thẻ tài xế. Mỗi tài xế một tag: hai thẻ của hai tài xế
/// khác nhau cùng nằm trên một màn hình không bao giờ trùng tag.
String driverHeroTag(Driver driver) => 'driver-card-${driver.id}';

/// Tiến độ chuyến từ 0 đến 1 để vẽ vòng tiến độ: sáu bước của đường đi bình
/// thường, huỷ thì về 0.
extension TripProgress on TripStatus {
  double get progress => switch (this) {
    TripStatus.requested => 0,
    TripStatus.searching => 1 / 6,
    TripStatus.accepted => 2 / 6,
    TripStatus.arriving => 3 / 6,
    TripStatus.pickedUp => 4 / 6,
    TripStatus.inTransit => 5 / 6,
    TripStatus.completed => 1,
    TripStatus.cancelled => 0,
  };
}
