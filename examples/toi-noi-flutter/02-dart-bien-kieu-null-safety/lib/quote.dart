import 'fare.dart';

/// Một báo giá: khoảng cách, cước và (có thể chưa có) tài xế.
class Quote {
  Quote({required this.distanceKm, required this.fareVnd, this.driverName}) {
    // Public final field không được promote, nên chép ra biến cục bộ trước.
    final name = driverName;
    if (name != null && name.trim().isEmpty) {
      throw ArgumentError.value(
        name,
        'driverName',
        'dùng null khi chưa có tài xế, không dùng chuỗi rỗng',
      );
    }
  }

  /// Tính cước từ khoảng cách rồi tạo báo giá.
  factory Quote.fromDistance(double distanceKm, {String? driverName}) {
    return Quote(
      distanceKm: distanceKm,
      fareVnd: calculateFareVnd(distanceKm),
      driverName: driverName,
    );
  }

  final double distanceKm;
  final int fareVnd; // không nullable: báo giá luôn có giá
  final String? driverName; // nullable: null nghĩa là chưa có tài xế

  /// Chuỗi tiền chỉ được tạo khi cần, đúng một lần; sau đó là hằng của đối tượng.
  late final String fareText = formatVnd(fareVnd);

  /// Trả bản sao đã có tài xế; đối tượng cũ giữ nguyên.
  Quote withDriver(String name) =>
      Quote(distanceKm: distanceKm, fareVnd: fareVnd, driverName: name);

  String describe() {
    // ?. : nếu driverName là null thì cả biểu thức là null, không ném lỗi.
    final givenName = driverName?.split(' ').last;
    // ?? : thay null bằng giá trị mặc định.
    final driver = givenName ?? 'chưa có tài xế';
    return '${distanceKm.toStringAsFixed(1)} km | $fareText | $driver';
  }
}
